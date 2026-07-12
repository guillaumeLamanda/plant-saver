#![allow(unexpected_cfgs)]
#![allow(unknown_lints)]
#![recursion_limit = "256"]

mod config;
mod h2_wireless;
mod matter_clusters;
mod matter_device;
mod sensors;

use esp_idf_svc::hal::adc::oneshot::config::{AdcChannelConfig, Calibration};
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::task::block_on;

use crate::config::SoilMoistureConfig;
use crate::sensors::SoilMoisture;

#[cfg(not(esp32h2))]
fn main() {
    panic!("device not supported")
}

fn main() {
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Booting plant-saver");

    let peripherals = esp_idf_svc::hal::peripherals::Peripherals::take().unwrap();

    let adc = AdcDriver::new(peripherals.adc1).unwrap();

    let soil_moisture_config = SoilMoistureConfig::default();

    let adc_channel_config = AdcChannelConfig {
        attenuation: soil_moisture_config.attenuation,
        calibration: Calibration::None,
        ..Default::default()
    };
    let channel = AdcChannelDriver::new(adc, peripherals.pins.gpio1, &adc_channel_config).unwrap();
    let mut soil_moisture = SoilMoisture::new(soil_moisture_config, channel);

    let modem = peripherals.modem;

    log_heap("pre-block-on");
    block_on(matter_device::matter(modem, || {
        soil_moisture.read().map(|level| level.value()).unwrap()
    }))
    .unwrap();
}

pub fn run_matter(modem: Modem, get_value: fn() -> u8) -> Result<(), anyhow::Error> {
    let result = block_on(matter_device::matter(modem, get_value));

    if let Err(e) = &result {
        log::error!("Matter aborted execution with error: {e:?}");
    }

    result
}

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
