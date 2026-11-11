use esp_idf_svc::{
    hal::adc::{
        ADCCH1, ADCU1,
        oneshot::{AdcChannelDriver, AdcDriver},
    },
    sys::EspError,
};

use crate::config::{SAMPLES_PER_READ, SoilMoistureConfig};

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
    min_plausible_raw: u16,
    max_plausible_raw: u16,
    channel: AdcChannelDriver<'a, ADCCH1<ADCU1>, AdcDriver<'a, ADCU1>>,
}

impl<'a> SoilMoisture<'a> {
    pub fn new(
        config: SoilMoistureConfig,
        channel: AdcChannelDriver<'a, ADCCH1<ADCU1>, AdcDriver<'a, ADCU1>>,
    ) -> Self {
        Self {
            max_dry: config.max_dry,
            max_wet: config.max_wet,
            min_plausible_raw: config.min_plausible_raw,
            max_plausible_raw: config.max_plausible_raw,
            channel,
        }
    }

    /// Current calibration bounds as `(wet, dry)` raw values.
    pub fn bounds(&self) -> (u16, u16) {
        (self.max_wet, self.max_dry)
    }

    /// Median of `SAMPLES_PER_READ` raw samples. One-shot reads take
    /// microseconds, so the burst still reflects a single instant.
    fn read_median_raw(&mut self) -> Result<u16, EspError> {
        let mut samples = [0u16; SAMPLES_PER_READ];
        for sample in &mut samples {
            *sample = self.channel.read_raw()?;
        }
        samples.sort_unstable();
        Ok(samples[SAMPLES_PER_READ / 2])
    }

    pub fn read_with_raw(&mut self) -> Result<SoilMoistureReading, EspError> {
        let raw = self.read_median_raw()?;

        // Adapt only on plausible readings: a faulty probe must not widen the
        // bounds for good and squash every later reading.
        if (self.min_plausible_raw..=self.max_plausible_raw).contains(&raw) {
            if raw > self.max_dry {
                self.max_dry = raw;
            }
            if raw < self.max_wet {
                self.max_wet = raw;
            }
        } else {
            log::warn!(
                "soil-moisture raw={raw} outside plausible range {}..={}; calibration unchanged",
                self.min_plausible_raw,
                self.max_plausible_raw
            );
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
