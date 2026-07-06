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

    let adc = AdcDriver::new(peripherals.adc2).unwrap();

    let mut moisture_led = PinDriver::output(peripherals.pins.gpio12).unwrap();
    let mut luminosity_led = PinDriver::output(peripherals.pins.gpio4).unwrap();
    luminosity_led.set_high().unwrap();
    thread::sleep(Duration::from_secs(1));
    luminosity_led.set_low().unwrap();

    let soil_moisture_config = SoilMoistureConfig::default();
    let mut esp_adc = EspAdc::new(&adc, peripherals.pins.gpio13, &soil_moisture_config).unwrap();
    let mut soil_moisture = SoilMoisture::new(soil_moisture_config);

    loop {
        #[cfg(debug_assertions)]
        let wait_time = 1000;
        #[cfg(not(debug_assertions))]
        let wait_time = 1000 * 60 * 10;

        thread::sleep(Duration::from_millis(wait_time));
        let level = soil_moisture.read(&mut esp_adc).unwrap();
        println!("soil moisture: {}", level.value());

        match level.value() {
            value if value < 65 => moisture_led.set_high().unwrap(),
            _ => moisture_led.set_low().unwrap(),
        }
    }
}
