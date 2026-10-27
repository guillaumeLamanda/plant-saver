use esp_idf_svc::{
    hal::adc::{
        oneshot::{AdcChannelDriver, AdcDriver},
        ADCCH0, ADCU1,
    },
    sys::EspError,
};

use crate::config::SoilMoistureConfig;

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
    pub raw: u16,
    pub level: SoilMoistureLevel,
}

pub struct SoilMoisture<'a> {
    max_dry: u16,
    max_wet: u16,
    channel: AdcChannelDriver<'a, ADCCH0<ADCU1>, AdcDriver<'a, ADCU1>>,
}

impl<'a> SoilMoisture<'a> {
    pub fn new(
        config: SoilMoistureConfig,
        channel: AdcChannelDriver<'a, ADCCH0<ADCU1>, AdcDriver<'a, ADCU1>>,
    ) -> Self {
        Self {
            max_dry: config.max_dry,
            max_wet: config.max_wet,
            channel,
        }
    }

    pub fn read_with_raw(&mut self) -> Result<SoilMoistureReading, EspError> {
        let raw = self.channel.read_raw()?;
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

    pub fn read(&mut self) -> Result<SoilMoistureLevel, EspError> {
        Ok(self.read_with_raw()?.level)
    }

    pub fn to_percentage(&self, raw: u16) -> u8 {
        let range = self.max_dry as f32 - self.max_wet as f32;
        if range == 0.0 {
            return 0;
        }
        let diff = self.max_dry as f32 - raw as f32;
        let percentage = (diff / range) * 100.0;
        percentage.clamp(0.0, 100.0) as u8
    }
}
