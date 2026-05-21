mod sensors;
use std::{thread, time::Duration};

use crate::sensors::MoistureSensor;

fn main() {
    use esp_idf_svc::hal::{adc::oneshot::AdcDriver, gpio::PinDriver, peripherals::Peripherals};

    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    // esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();

    let adc = AdcDriver::new(peripherals.adc2).unwrap();

    let mut moisture_led = PinDriver::output(peripherals.pins.gpio12).unwrap();
    let mut luminosity_led = PinDriver::output(peripherals.pins.gpio4).unwrap();
    luminosity_led.set_high().unwrap();
    thread::sleep(Duration::from_secs(1));
    luminosity_led.set_low().unwrap();

    let mut moisture_sensor = MoistureSensor::new(&adc, peripherals.pins.gpio13);

    loop {
        #[cfg(debug_assertions)]
        let wait_time = 1000;
        #[cfg(not(debug_assertions))]
        let wait_time = 1000 * 60 * 10;

        thread::sleep(Duration::from_millis(wait_time));
        let value = moisture_sensor.read_humidity_level().unwrap();
        println!("humidity: {}", value);

        match value {
            value if value < 65 => moisture_led.set_high().unwrap(),
            _ => moisture_led.set_low().unwrap(),
        }
    }
}
