mod config;
mod sensors;

use std::{thread, time::Duration};

use crate::config::SoilMoistureConfig;
use crate::sensors::{EspAdc, SoilMoisture};

fn main() {
    use esp_idf_svc::hal::{adc::oneshot::AdcDriver, gpio::PinDriver, peripherals::Peripherals};

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();

    let adc = AdcDriver::new(peripherals.adc1).unwrap();

    let mut moisture_led = PinDriver::output(peripherals.pins.gpio12).unwrap();

    let soil_moisture_config = SoilMoistureConfig::default();
    let mut esp_adc = EspAdc::new(&adc, peripherals.pins.gpio36, &soil_moisture_config).unwrap();
    let mut soil_moisture = SoilMoisture::new(soil_moisture_config);

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
    }
}
