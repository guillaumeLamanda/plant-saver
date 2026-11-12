#![recursion_limit = "256"]

#[cfg(not(esp32c6))]
compile_error!("plant-saver only supports the ESP32-C6 (MCU=esp32c6)");

mod config;
mod matter_clusters;
mod matter_device;
mod sensors;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use esp_idf_svc::hal::adc::oneshot::config::{AdcChannelConfig, Calibration};
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::hal::task::thread::ThreadSpawnConfiguration;

use soil_moisture::{SoilMoisture, SoilMoistureConfig};

use crate::config::SOIL_MOISTURE_ATTENUATION;
use crate::matter_clusters::humidity::SharedSoilMoistureLevel;
use crate::sensors::EspAdc;

/// Time to let the logs flush after the Matter stack dies, and to keep a
/// persistent failure from becoming a tight reboot loop.
const MATTER_RESTART_DELAY: Duration = Duration::from_secs(10);

fn main() {
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Booting plant-saver");

    let peripherals = esp_idf_svc::hal::peripherals::Peripherals::take().unwrap();

    let adc = AdcDriver::new(peripherals.adc1).unwrap();

    // Curve fitting only feeds `raw_to_mv` (the `mv=` log); the level is
    // computed from raw counts, which `read_raw` returns uncalibrated.
    let adc_channel_config = AdcChannelConfig {
        attenuation: SOIL_MOISTURE_ATTENUATION,
        calibration: Calibration::Curve,
        ..Default::default()
    };
    let channel = AdcChannelDriver::new(adc, peripherals.pins.gpio1, &adc_channel_config).unwrap();
    let mut soil_moisture = SoilMoisture::new(SoilMoistureConfig::default(), EspAdc::new(channel));

    // A failed or implausible boot reading leaves the level `None`.
    if let Err(e) = soil_moisture.read() {
        log::error!("ADC read error at boot: {e:?}");
    }
    let level = Arc::new(SharedSoilMoistureLevel::new(soil_moisture.level()));
    let modem = peripherals.modem;

    let level_for_matter = level.clone();
    ThreadSpawnConfiguration::set(&ThreadSpawnConfiguration {
        name: Some(c"matter"),
        ..Default::default()
    })
    .unwrap();
    thread::Builder::new()
        .stack_size(20 * 1024)
        .spawn(move || {
            // The stack is not supposed to return; whether it errors or ends,
            // the device is off the network, so restart rather than keep
            // running the sensor loop alone.
            match run_matter(modem, level_for_matter) {
                Err(e) => log::error!("Matter aborted: {e:?}"),
                Ok(()) => log::error!("Matter stack returned unexpectedly"),
            }
            log::error!("Restarting in {}s", MATTER_RESTART_DELAY.as_secs());
            thread::sleep(MATTER_RESTART_DELAY);
            esp_idf_svc::hal::reset::restart();
        })
        .unwrap();

    let poll_interval = if cfg!(debug_assertions) {
        Duration::from_secs(10)
    } else {
        Duration::from_secs(600)
    };

    loop {
        let result = soil_moisture.read();
        // Publish in both arms: a fault keeps the last level until the fault
        // limit, then clears it.
        level.set(soil_moisture.level());
        match result {
            Ok(reading) => {
                // `raw` is this poll's value, unsmoothed; `mv` is the same
                // value calibrated. Re-measure the dry/wet bounds from them.
                let (wet, dry) = soil_moisture.bounds();
                let mv = soil_moisture.adc().raw_to_mv(reading.raw);
                let moisture = reading.level.map(|l| l.value());
                if !reading.plausible {
                    log::warn!("soil-moisture raw={} outside plausible range", reading.raw);
                }
                log::info!(
                    "moisture={moisture:?}% raw={} mv={mv:?} smoothed={:?} bounds(wet={wet}, dry={dry})",
                    reading.raw,
                    reading.smoothed_raw
                );
            }
            Err(e) => {
                log::error!(
                    "ADC read error: {e:?} (moisture={:?}%)",
                    soil_moisture.level().map(|l| l.value())
                );
            }
        }
        std::thread::sleep(poll_interval);
    }
}

pub fn run_matter(modem: Modem, level: Arc<SharedSoilMoistureLevel>) -> Result<(), anyhow::Error> {
    block_on(matter_device::matter(modem, level))
}

/// Stack-tuning diagnostic; call it from the loop when sizing task stacks.
#[allow(dead_code)]
fn log_task_stacks() {
    use esp_idf_svc::sys;

    for name in [
        c"matter",
        c"ThreadDriver Ru",
        c"BTC_TASK",
        c"BTU_TASK",
        c"ble_ll_task",
        c"hciT",
        c"sys_evt",
    ] {
        let name_str = name.to_str().unwrap();

        // These tasks remain alive for the firmware lifetime. Looking them up
        // individually avoids FreeRTOS trace metadata and snapshot buffers.
        let handle = unsafe { sys::xTaskGetHandle(name.as_ptr()) };

        if handle.is_null() {
            log::warn!("task-stack ({name_str}): not found");
            continue;
        }

        // Convert the reported StackType_t units to bytes for this target.
        let slack_words = unsafe { sys::uxTaskGetStackHighWaterMark2(handle) };
        let slack_bytes = slack_words * core::mem::size_of::<sys::StackType_t>() as u32;

        log::info!("task-stack ({name_str}): slack_bytes={slack_bytes}");
    }
}

pub(crate) fn log_heap(tag: &str) {
    let free = unsafe { esp_idf_svc::sys::esp_get_free_heap_size() };
    let min = unsafe { esp_idf_svc::sys::esp_get_minimum_free_heap_size() };
    let largest = unsafe {
        esp_idf_svc::sys::heap_caps_get_largest_free_block(esp_idf_svc::sys::MALLOC_CAP_DEFAULT)
    };
    log::info!("heap ({tag}): free={free}, largest_block={largest}, min_free_ever={min}");
}
