#![allow(unexpected_cfgs)]
#![allow(unknown_lints)]
#![recursion_limit = "256"]

mod config;
#[cfg(esp32h2)]
mod h2_wireless;
mod matter_clusters;
mod matter_device;
mod sensors;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

use esp_idf_svc::hal::task::thread::ThreadSpawnConfiguration;

use crate::config::SoilMoistureConfig;
use crate::sensors::{EspAdc, SoilMoisture};

fn main() {
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Booting plant-saver");

    let peripherals = esp_idf_svc::hal::peripherals::Peripherals::take().unwrap();

    let adc = esp_idf_svc::hal::adc::oneshot::AdcDriver::new(peripherals.adc1).unwrap();

    #[cfg(not(esp32h2))]
    let mut moisture_led =
        esp_idf_svc::hal::gpio::PinDriver::output(peripherals.pins.gpio12).unwrap();

    let soil_moisture_config = SoilMoistureConfig::default();

    #[cfg(esp32)]
    let mut esp_adc = EspAdc::new(&adc, peripherals.pins.gpio36, &soil_moisture_config).unwrap();

    #[cfg(esp32h2)]
    let mut esp_adc = EspAdc::new(&adc, peripherals.pins.gpio1, &soil_moisture_config).unwrap();
    let mut soil_moisture = SoilMoisture::new(soil_moisture_config);

    let modem = peripherals.modem;

    // Latest SoilMoistureLevel (0–100), shared with the Matter stack: the read
    // loop publishes here, the RelativeHumidityMeasurement handler reads it
    // pull-based on each controller attribute read.
    let soil_moisture_level = Arc::new(AtomicU8::new(0));
    let level_for_matter = soil_moisture_level.clone();

    ThreadSpawnConfiguration::set(&ThreadSpawnConfiguration {
        name: Some(c"matter"),
        ..Default::default()
    })
    .unwrap();

    thread::Builder::new()
        .stack_size(20 * 1024)
        .spawn(move || {
            if let Err(e) = matter_device::run(modem, level_for_matter) {
                log::error!("Matter stack aborted: {e:?}");
            }
        })
        .unwrap();
}

#[cfg(esp32h2)]
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
