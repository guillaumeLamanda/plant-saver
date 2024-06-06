use std::{
    ops::{Div, Mul},
    thread,
    time::Duration,
    u16,
};

#[cfg(not(esp_idf_version_major = "4"))]
fn main() {
    use esp_idf_svc::hal::{
        adc::{
            attenuation,
            oneshot::{config::AdcChannelConfig, AdcChannelDriver, AdcDriver},
        },
        gpio::PinDriver,
        peripherals::Peripherals,
    };

    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    // esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();

    let adc_channel_config = AdcChannelConfig {
        attenuation: attenuation::DB_11,
        calibration: true,
        ..Default::default()
    };

    let adc = AdcDriver::new(peripherals.adc2).unwrap();

    let mut moisture_led = PinDriver::output(peripherals.pins.gpio12).unwrap();
    let mut luminosity_led = PinDriver::output(peripherals.pins.gpio4).unwrap();
    luminosity_led.set_high().unwrap();
    thread::sleep(Duration::from_secs(1));
    luminosity_led.set_low().unwrap();

    let mut moisture_adc_pin =
        AdcChannelDriver::new(&adc, peripherals.pins.gpio13, &adc_channel_config).unwrap();

    loop {
        #[cfg(debug_assertions)]
        let wait_time = 1000;
        #[cfg(not(debug_assertions))]
        let wait_time = 1000 * 60 * 10;

        thread::sleep(Duration::from_millis(wait_time));
        let adc_value = adc.read(&mut moisture_adc_pin).unwrap();
        println!("adc_value: {}", adc_value);

        // luminosity_led.set_high().unwrap();
        // value in water : 709
        // value when the humidity is 60%: 2752
        println!("row value: {}", adc_value);
        let value = moisture_voltage_to_humidity_level(adc_value);
        println!("humidity: {}", value);

        match value {
            value if value < 65 => {
                println!("Arose moi ! (humidity:{})", value);
                moisture_led.set_high().unwrap()
            }
            _ => {
                println!("I'm fine ! (humidity:{})", value);
                moisture_led.set_low().unwrap()
            }
        }
    }
}

fn moisture_voltage_to_humidity_level(voltage: u16) -> u16 {
    const MAX_DRY: f32 = 4095.0;
    const MAX_WET: f32 = 700.0;

    // linear conversion
    let value_diff: f32 = MAX_DRY - voltage as f32;
    let x = value_diff.div(MAX_DRY - MAX_WET).mul(100.0);
    x as u16
}
