use std::{time::Duration, thread};

use esp_idf_svc::hal::{prelude::Peripherals, adc::{AdcDriver, config::Config, AdcChannelDriver, attenuation}};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();

    let adc_config = Config::new();
    #[cfg(not(esp32))]
    let mut adc = AdcDriver::new(peripherals.adc1, &adc_config).unwrap();

    #[cfg(esp32)]
    let mut adc = AdcDriver::new(peripherals.adc2, &adc_config).unwrap();

    #[cfg(not(esp32))]
    let mut adc_pin: AdcChannelDriver<{ attenuation::DB_11 }, _> =
    AdcChannelDriver::new(peripherals.pins.gpio4)?;

    #[cfg(esp32)]
    let mut adc_pin: AdcChannelDriver<{ attenuation::DB_11 }, _> =
    AdcChannelDriver::new(peripherals.pins.gpio13)?;

    loop {
        // you can change the sleep duration depending on how often you want to sample
        thread::sleep(Duration::from_millis(10));
        println!("ADC value: {}", adc.read(&mut adc_pin)?);
    }

    log::info!("Hello, world!");
}
