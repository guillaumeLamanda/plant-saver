use std::{time::Duration, thread};

use esp_idf_svc::hal::{prelude::Peripherals, adc::{AdcDriver, config::Config, AdcChannelDriver, attenuation}, gpio::{ADCPin, PinDriver}};

const MAX_DRY: u16 = 2491;
const MAX_WET: u16 = 741;

const MOISTURE_RANGE: u16 = MAX_DRY - MAX_WET;
const FULL_PRECENTAGE: f32 = 100.0;
const NO_PRECENTAGE: f32 = 0.0;

fn main() {
    log::info!("Hello, world!");
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
    AdcChannelDriver::new(peripherals.pins.gpio4).unwrap();

    #[cfg(not(esp32))]
    let mut led = PinDriver::output(peripherals.pins.gpio5).unwrap();

    #[cfg(esp32)]
    let mut led = PinDriver::output(peripherals.pins.gpio12).unwrap();

    #[cfg(esp32)]
    let mut adc_pin: AdcChannelDriver<{ attenuation::DB_11 }, _> =
    AdcChannelDriver::new(peripherals.pins.gpio13).unwrap();

    loop {
        #[cfg(debug_assertions)]
        let wait_time = 1000;
        #[cfg(not(debug_assertions))]
        let wait_time = 1000 * 60 * 10;

        // you can change the sleep duration depending on how often you want to sample
        thread::sleep(Duration::from_millis(wait_time));
        let adc_value = adc.read(&mut adc_pin).unwrap();

        let value_diff = MAX_DRY - adc_value;
        let value = ( (value_diff as f32 / MOISTURE_RANGE as f32) * FULL_PRECENTAGE ).trunc();
        match value {
            value if value < 60.0 => { 
                println!("Arose moi ! (humidity:{})", value);
                led.set_high().unwrap() 
            },
            _ => { 
                println!("I'm fine ! (humidity:{})", value);
                led.set_low().unwrap() 
            },
        }
    }

}
