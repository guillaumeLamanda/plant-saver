#![allow(unexpected_cfgs)]
#![recursion_limit = "256"]

mod config;
mod matter_device;
mod sensors;

use std::{thread, time::Duration};

use esp_idf_svc::hal::task::thread::ThreadSpawnConfiguration;

use crate::config::SoilMoistureConfig;
use crate::sensors::{EspAdc, SoilMoisture};

fn main() {
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Booting plant-saver");

    let peripherals = esp_idf_svc::hal::peripherals::Peripherals::take().unwrap();

    let adc = esp_idf_svc::hal::adc::oneshot::AdcDriver::new(peripherals.adc1).unwrap();
    let mut moisture_led =
        esp_idf_svc::hal::gpio::PinDriver::output(peripherals.pins.gpio12).unwrap();

    let soil_moisture_config = SoilMoistureConfig::default();
    let mut esp_adc = EspAdc::new(&adc, peripherals.pins.gpio36, &soil_moisture_config).unwrap();
    let mut soil_moisture = SoilMoisture::new(soil_moisture_config);

    let modem = peripherals.modem;

    ThreadSpawnConfiguration::set(&ThreadSpawnConfiguration {
        name: Some(c"matter"),
        ..Default::default()
    })
    .unwrap();

    thread::Builder::new()
        .stack_size(20 * 1024)
        .spawn(move || {
            if let Err(e) = matter_device::run(modem) {
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

        match soil_moisture.read(&mut esp_adc) {
            Ok(level) => {
                let needs_water = soil_moisture.needs_water(level);
                log::info!(
                    "soil moisture: {}, needs water: {}",
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
