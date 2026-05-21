use esp_idf_svc::{
    hal::{
        adc::{
            attenuation,
            oneshot::{
                config::{AdcChannelConfig, Calibration},
                AdcChannelDriver, AdcDriver,
            },
            ADCCH4, ADCU2,
        },
        gpio::Gpio13,
    },
    sys::EspError,
};
use std::ops::{Div, Mul};

pub struct MoistureSensor<'a> {
    adc: &'a AdcDriver<'a, ADCU2>,
    pin: AdcChannelDriver<'a, ADCCH4<ADCU2>, &'a AdcDriver<'a, ADCU2>>,
    max_dry: u16,
    max_wet: u16,
}

impl<'a> MoistureSensor<'a> {
    pub fn new(adc: &'a AdcDriver<'a, ADCU2>, pin: Gpio13<'a>) -> Self {
        let adc_channel_config = AdcChannelConfig {
            attenuation: attenuation::DB_12,
            calibration: Calibration::Line,
            ..Default::default()
        };
        let pin = AdcChannelDriver::new(adc, pin, &adc_channel_config).unwrap();
        Self {
            adc,
            pin,
            max_dry: 4095,
            max_wet: 1500,
        }
    }

    pub fn read_humidity_level(&mut self) -> Result<u16, EspError> {
        let voltage_value = self.adc.read(&mut self.pin)?;
        if voltage_value > self.max_dry {
            self.max_dry = voltage_value;
        }
        if voltage_value < self.max_wet {
            self.max_wet = voltage_value;
        }

        Ok(self.moisture_voltage_to_humidity_level(voltage_value))
    }

    fn moisture_voltage_to_humidity_level(&self, voltage: u16) -> u16 {
        let value_diff: f32 = self.max_dry as f32 - voltage as f32;
        value_diff
            .div(self.max_dry as f32 - self.max_wet as f32)
            .mul(100.0) as u16
    }
}
