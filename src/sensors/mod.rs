use esp_idf_svc::{
    hal::{
        adc::{
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

use crate::config::SoilMoistureConfig;

pub trait Adc {
    fn read(&mut self) -> Result<u16, EspError>;
}

pub struct EspAdc<'a> {
    driver: &'a AdcDriver<'a, ADCU2>,
    channel: AdcChannelDriver<'a, ADCCH4<ADCU2>, &'a AdcDriver<'a, ADCU2>>,
}

impl<'a> EspAdc<'a> {
    pub fn new(
        driver: &'a AdcDriver<'a, ADCU2>,
        pin: Gpio13<'a>,
        config: &SoilMoistureConfig,
    ) -> Result<Self, EspError> {
        let adc_channel_config = AdcChannelConfig {
            attenuation: config.attenuation,
            calibration: Calibration::Line,
            ..Default::default()
        };
        let channel = AdcChannelDriver::new(driver, pin, &adc_channel_config)?;
        Ok(Self { driver, channel })
    }
}

impl<'a> Adc for EspAdc<'a> {
    fn read(&mut self) -> Result<u16, EspError> {
        self.driver.read(&mut self.channel)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoilMoistureLevel(u8);

impl SoilMoistureLevel {
    pub fn new(value: u8) -> Self {
        Self(value.clamp(0, 100))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

pub struct SoilMoisture {
    max_dry: u16,
    max_wet: u16,
}

impl SoilMoisture {
    pub fn new(config: SoilMoistureConfig) -> Self {
        Self {
            max_dry: config.max_dry,
            max_wet: config.max_wet,
        }
    }

    pub fn read<A: Adc>(&mut self, adc: &mut A) -> Result<SoilMoistureLevel, EspError> {
        let raw = adc.read()?;
        if raw > self.max_dry {
            self.max_dry = raw;
        }
        if raw < self.max_wet {
            self.max_wet = raw;
        }
        Ok(SoilMoistureLevel::new(self.to_percentage(raw)))
    }

    fn to_percentage(&self, raw: u16) -> u8 {
        let range = self.max_dry as f32 - self.max_wet as f32;
        if range == 0.0 {
            return 0;
        }
        let diff = self.max_dry as f32 - raw as f32;
        let percentage = (diff / range) * 100.0;
        percentage.clamp(0.0, 100.0) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeAdc {
        values: Vec<u16>,
        index: usize,
    }

    impl FakeAdc {
        fn new(values: Vec<u16>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl Adc for FakeAdc {
        fn read(&mut self) -> Result<u16, EspError> {
            let value = self.values[self.index % self.values.len()];
            self.index += 1;
            Ok(value)
        }
    }

    fn default_config() -> SoilMoistureConfig {
        SoilMoistureConfig::default()
    }

    #[test]
    fn dry_reading_returns_zero() {
        let mut soil = SoilMoisture::new(default_config());
        let mut adc = FakeAdc::new(vec![4095]);
        let level = soil.read(&mut adc).unwrap();
        assert_eq!(level.value(), 0);
    }

    #[test]
    fn wet_reading_returns_one_hundred() {
        let mut soil = SoilMoisture::new(default_config());
        let mut adc = FakeAdc::new(vec![1500]);
        let level = soil.read(&mut adc).unwrap();
        assert_eq!(level.value(), 100);
    }

    #[test]
    fn mid_reading_returns_fifty() {
        let mut soil = SoilMoisture::new(default_config());
        let midpoint = (4095 + 1500) / 2;
        let mut adc = FakeAdc::new(vec![midpoint]);
        let level = soil.read(&mut adc).unwrap();
        assert_eq!(level.value(), 50);
    }

    #[test]
    fn adaptive_bounds_widen() {
        let mut config = default_config();
        config.max_dry = 3000;
        config.max_wet = 2000;
        let mut soil = SoilMoisture::new(config);
        let mut adc = FakeAdc::new(vec![3500, 1000]);
        let _ = soil.read(&mut adc).unwrap();
        let level = soil.read(&mut adc).unwrap();
        assert_eq!(level.value(), 100);
    }
}
