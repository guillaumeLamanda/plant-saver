use esp_idf_svc::{
    hal::adc::{
        ADCCH1, ADCU1,
        oneshot::{AdcChannelDriver, AdcDriver},
    },
    sys::EspError,
};

use crate::config::{SAMPLES_PER_READ, SMOOTHING_ALPHA, SoilMoistureConfig};

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
    /// This poll's raw value (median of one burst).
    pub raw: u16,
    /// Moving average of plausible raw values; the level is computed from it.
    pub smoothed_raw: u16,
    pub level: SoilMoistureLevel,
}

pub struct SoilMoisture<'a> {
    dry_raw: u16,
    wet_raw: u16,
    min_plausible_raw: u16,
    max_plausible_raw: u16,
    /// `None` until the first plausible reading.
    smoothed_raw: Option<f32>,
    channel: AdcChannelDriver<'a, ADCCH1<ADCU1>, AdcDriver<'a, ADCU1>>,
}

impl<'a> SoilMoisture<'a> {
    pub fn new(
        config: SoilMoistureConfig,
        channel: AdcChannelDriver<'a, ADCCH1<ADCU1>, AdcDriver<'a, ADCU1>>,
    ) -> Self {
        Self {
            dry_raw: config.dry_raw,
            wet_raw: config.wet_raw,
            min_plausible_raw: config.min_plausible_raw,
            max_plausible_raw: config.max_plausible_raw,
            smoothed_raw: None,
            channel,
        }
    }

    /// Calibration bounds as `(wet, dry)` raw values.
    pub fn bounds(&self) -> (u16, u16) {
        (self.wet_raw, self.dry_raw)
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

        // A faulty probe (disconnected, shorted, unpowered) must not drag the
        // average: skip it and keep the last level.
        if (self.min_plausible_raw..=self.max_plausible_raw).contains(&raw) {
            let raw = f32::from(raw);
            self.smoothed_raw = Some(match self.smoothed_raw {
                Some(prev) => prev + SMOOTHING_ALPHA * (raw - prev),
                None => raw,
            });
        } else {
            log::warn!(
                "soil-moisture raw={raw} outside plausible range {}..={}; reading ignored",
                self.min_plausible_raw,
                self.max_plausible_raw
            );
        }

        // Before any plausible reading there is nothing to average, so the
        // faulty value itself gives the level.
        let basis = self.smoothed_raw.unwrap_or(f32::from(raw));

        Ok(SoilMoistureReading {
            raw,
            smoothed_raw: basis.round() as u16,
            level: SoilMoistureLevel::new(self.to_percentage(basis)),
        })
    }

    pub fn read(&mut self) -> Result<SoilMoistureLevel, EspError> {
        Ok(self.read_with_raw()?.level)
    }

    /// Linear map from `dry_raw` (0%) to `wet_raw` (100%), clamped.
    /// Capacitive probes read lower when wetter.
    fn to_percentage(&self, raw: f32) -> u8 {
        let range = f32::from(self.dry_raw) - f32::from(self.wet_raw);
        if range <= 0.0 {
            return 0;
        }
        let percentage = (f32::from(self.dry_raw) - raw) / range * 100.0;
        percentage.clamp(0.0, 100.0) as u8
    }
}
