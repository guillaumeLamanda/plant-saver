use esp_idf_svc::{
    hal::{
        adc::{
            oneshot::{
                config::{AdcChannelConfig, Calibration},
                AdcChannelDriver, AdcDriver,
            },
            ADCCH0, ADCU1,
        },
        gpio::Gpio36,
    },
    sys::EspError,
};

use crate::config::SoilMoistureConfig;

pub trait Adc {
    fn read(&mut self) -> Result<u16, EspError>;
}

pub struct EspAdc<'a> {
    driver: &'a AdcDriver<'a, ADCU1>,
    channel: AdcChannelDriver<'a, ADCCH0<ADCU1>, &'a AdcDriver<'a, ADCU1>>,
}

impl<'a> EspAdc<'a> {
    pub fn new(
        driver: &'a AdcDriver<'a, ADCU1>,
        pin: Gpio36<'a>,
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
        self.driver.read_raw(&mut self.channel)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoilMoistureReading {
    raw: u16,
    level: SoilMoistureLevel,
}

impl SoilMoistureReading {
    pub fn raw(&self) -> u16 {
        self.raw
    }

    pub fn level(&self) -> SoilMoistureLevel {
        self.level
    }
}

pub struct SoilMoisture {
    max_dry: u16,
    max_wet: u16,
    needs_water_threshold: u8,
}

impl SoilMoisture {
    pub fn new(config: SoilMoistureConfig) -> Self {
        Self {
            max_dry: config.max_dry,
            max_wet: config.max_wet,
            needs_water_threshold: config.needs_water_threshold.clamp(0, 100),
        }
    }

    pub fn read_with_raw<A: Adc>(&mut self, adc: &mut A) -> Result<SoilMoistureReading, EspError> {
        let raw = adc.read()?;
        if raw > self.max_dry {
            self.max_dry = raw;
        }
        if raw < self.max_wet {
            self.max_wet = raw;
        }
        Ok(SoilMoistureReading {
            raw,
            level: SoilMoistureLevel::new(self.to_percentage(raw)),
        })
    }

    pub fn needs_water(&self, level: SoilMoistureLevel) -> bool {
        level.value() < self.needs_water_threshold
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
        let level = soil.read_with_raw(&mut adc).unwrap().level();
        assert_eq!(level.value(), 0);
    }

    #[test]
    fn wet_reading_returns_one_hundred() {
        let mut soil = SoilMoisture::new(default_config());
        let mut adc = FakeAdc::new(vec![1500]);
        let level = soil.read_with_raw(&mut adc).unwrap().level();
        assert_eq!(level.value(), 100);
    }

    #[test]
    fn mid_reading_returns_fifty() {
        let mut soil = SoilMoisture::new(default_config());
        let midpoint = (4095 + 1500) / 2;
        let mut adc = FakeAdc::new(vec![midpoint]);
        let level = soil.read_with_raw(&mut adc).unwrap().level();
        assert_eq!(level.value(), 50);
    }

    #[test]
    fn reading_exposes_raw_adc_value() {
        let mut soil = SoilMoisture::new(default_config());
        let mut adc = FakeAdc::new(vec![2345]);

        let reading = soil.read_with_raw(&mut adc).unwrap();

        assert_eq!(reading.raw(), 2345);
        assert_eq!(reading.level().value(), 67);
    }

    #[test]
    fn adaptive_bounds_widen() {
        let mut config = default_config();
        config.max_dry = 3000;
        config.max_wet = 2000;
        let mut soil = SoilMoisture::new(config);
        let mut adc = FakeAdc::new(vec![3500, 1000]);
        let _ = soil.read_with_raw(&mut adc).unwrap();
        let level = soil.read_with_raw(&mut adc).unwrap().level();
        assert_eq!(level.value(), 100);
    }

    #[test]
    fn needs_water_dry_reading_returns_true() {
        let mut soil = SoilMoisture::new(default_config());
        let mut adc = FakeAdc::new(vec![4095]);
        let level = soil.read_with_raw(&mut adc).unwrap().level();
        assert!(soil.needs_water(level));
    }

    #[test]
    fn needs_water_wet_reading_returns_false() {
        let mut soil = SoilMoisture::new(default_config());
        let mut adc = FakeAdc::new(vec![1500]);
        let level = soil.read_with_raw(&mut adc).unwrap().level();
        assert!(!soil.needs_water(level));
    }

    #[test]
    fn needs_water_at_threshold_returns_false() {
        let soil = SoilMoisture::new(default_config());
        assert!(!soil.needs_water(SoilMoistureLevel::new(
            SoilMoistureConfig::default().needs_water_threshold
        )));
    }

    #[test]
    fn needs_water_just_below_threshold_returns_true() {
        let soil = SoilMoisture::new(default_config());
        assert!(soil.needs_water(SoilMoistureLevel::new(
            SoilMoistureConfig::default().needs_water_threshold - 1
        )));
    }

    struct FailingAdc;

    impl Adc for FailingAdc {
        fn read(&mut self) -> Result<u16, EspError> {
            Err(EspError::from_non_zero(
                core::num::NonZeroI32::new(1).unwrap(),
            ))
        }
    }

    #[test]
    fn read_propagates_adc_error_so_loop_can_skip() {
        let mut soil = SoilMoisture::new(default_config());
        let mut adc = FailingAdc;
        let result = soil.read_with_raw(&mut adc);
        assert!(result.is_err());
    }
}
