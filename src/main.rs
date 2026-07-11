#![allow(unexpected_cfgs)]
#![allow(unknown_lints)]
#![recursion_limit = "256"]

mod config;
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

    #[cfg(esp32)]
    let mut moisture_led =
        esp_idf_svc::hal::gpio::PinDriver::output(peripherals.pins.gpio12).unwrap();

    #[cfg(esp32h2)]
    let mut moisture_led =
        esp_idf_svc::hal::gpio::PinDriver::output(peripherals.pins.gpio1).unwrap();

    let soil_moisture_config = SoilMoistureConfig::default();
    let mut esp_adc = EspAdc::new(&adc, peripherals.pins.gpio36, &soil_moisture_config).unwrap();
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

    log_heap("baseline");
    let mut cycle: u32 = 0;
    loop {
        #[cfg(debug_assertions)]
        let wait_time = 1000;
        #[cfg(not(debug_assertions))]
        let wait_time = 1000 * 60 * 10;

        thread::sleep(Duration::from_millis(wait_time));

        match soil_moisture.read_with_raw(&mut esp_adc) {
            Ok(reading) => {
                let level = reading.level();
                let needs_water = soil_moisture.needs_water(level);
                soil_moisture_level.store(level.value(), Ordering::Relaxed);
                log::info!(
                    "soil moisture raw: {}, level: {}, needs water: {}",
                    reading.raw(),
                    level.value(),
                    needs_water
                );
                if needs_water {
                    moisture_led.set_high().unwrap();
                } else {
                    moisture_led.set_low().unwrap();
                }
            }
            Err(e) => log::warn!("soil moisture read failed, skipping cycle: {}", e),
        }

        cycle = cycle.wrapping_add(1);
        if cycle < 10 || cycle % 10 == 0 {
            log_heap("runtime");
        }
    }
}

fn log_heap(tag: &str) {
    let free = unsafe { esp_idf_svc::sys::esp_get_free_heap_size() };
    let min = unsafe { esp_idf_svc::sys::esp_get_minimum_free_heap_size() };
    log::info!("heap ({tag}): free={free} bytes, min={min} bytes");
}
