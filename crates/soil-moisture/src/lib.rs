//! Soil moisture reading logic, independent of any hardware.
//!
//! [`SoilMoisture`] turns raw ADC counts into a [`SoilMoistureLevel`]. It sits
//! behind the [`Adc`] seam (the "ADC adapter"): the firmware implements it for
//! the ESP32 ADC, tests implement it with scripted values.
//!
//! The level is `None` until the first plausible reading, and again after
//! [`MAX_CONSECUTIVE_FAULTS`] faults in a row (see [`SoilMoisture::read`]), so
//! a faulty probe is never published as a real level.

/// Raw ADC samples taken per reading; their median is the reading, so a single
/// glitched sample does not show up in the level. Odd, so the median is one
/// sample.
pub const SAMPLES_PER_READ: usize = 9;

/// Weight of a new reading in the exponential moving average over readings
/// (0 < α ≤ 1). 0.5 halves the effect of a one-off odd reading while a real
/// change (watering) shows up within two or three polls.
pub const SMOOTHING_ALPHA: f32 = 0.5;

/// Consecutive faults (implausible raw value or ADC error) after which the
/// level becomes `None` and the smoothing state is reset.
pub const MAX_CONSECUTIVE_FAULTS: u8 = 3;

/// A percentage (0–100) of soil moisture: 0 is dry, 100 is freshly watered.
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

/// The ADC adapter: supplies raw ADC counts.
pub trait Adc {
    type Error;

    fn read_raw(&mut self) -> Result<u16, Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoilMoistureConfig {
    /// Raw value for 0%: the probe pushed to its line in dry soil (the plant
    /// needs water). Measured on the real probe: dry potting soil ≈ 2786,
    /// air ≈ 2794.
    pub dry_raw: u16,
    /// Raw value for 100%: the probe in freshly watered soil, ~15 min after
    /// watering. Not water itself (≈ 1326): soil never gets that wet, so a
    /// water bound squashes every soil reading into the bottom of the scale.
    /// 2100 is an estimate from soil watered 2 days earlier (≈ 2164); replace
    /// it with a fresh-watering measurement.
    pub wet_raw: u16,
    /// Readings outside this raw range are faults (probe disconnected,
    /// shorted or unpowered): they do not move the smoothed value, and after
    /// [`MAX_CONSECUTIVE_FAULTS`] in a row the level becomes `None`.
    pub min_plausible_raw: u16,
    pub max_plausible_raw: u16,
    /// Reserved for the read-only "needs water" On/Off cluster (slice 04).
    pub needs_water_threshold: u8,
}

impl Default for SoilMoistureConfig {
    fn default() -> Self {
        Self {
            dry_raw: 2780,
            wet_raw: 2100,
            min_plausible_raw: 500,
            max_plausible_raw: 4000,
            needs_water_threshold: 65,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoilMoistureReading {
    /// This poll's raw value (median of one burst).
    pub raw: u16,
    /// Whether `raw` is inside the plausible range. When `false` the reading
    /// is a fault and did not move the smoothed value.
    pub plausible: bool,
    /// Moving average of plausible raw values; the level is computed from it.
    /// `None` before the first plausible reading and after a reset.
    pub smoothed_raw: Option<u16>,
    /// Same as [`SoilMoisture::level`] after this reading.
    pub level: Option<SoilMoistureLevel>,
}

pub struct SoilMoisture<A: Adc> {
    config: SoilMoistureConfig,
    adc: A,
    /// `None` until the first plausible reading, and after a fault reset.
    smoothed_raw: Option<f32>,
    consecutive_faults: u8,
}

impl<A: Adc> SoilMoisture<A> {
    pub fn new(config: SoilMoistureConfig, adc: A) -> Self {
        Self {
            config,
            adc,
            smoothed_raw: None,
            consecutive_faults: 0,
        }
    }

    pub fn adc(&self) -> &A {
        &self.adc
    }

    /// Calibration bounds as `(wet, dry)` raw values.
    pub fn bounds(&self) -> (u16, u16) {
        (self.config.wet_raw, self.config.dry_raw)
    }

    /// The current level; `None` until the first plausible reading and after
    /// [`MAX_CONSECUTIVE_FAULTS`] consecutive faults.
    pub fn level(&self) -> Option<SoilMoistureLevel> {
        self.smoothed_raw
            .map(|raw| SoilMoistureLevel::new(self.to_percentage(raw)))
    }

    /// Take one reading.
    ///
    /// An implausible raw value or an ADC error is a fault: it does not move
    /// the smoothed value. After [`MAX_CONSECUTIVE_FAULTS`] consecutive faults
    /// the level becomes `None` and the smoothing state is reset, so recovery
    /// starts fresh. A plausible reading resets the fault count. An `Err`
    /// counts as a fault; read [`Self::level`] afterwards to publish it.
    pub fn read(&mut self) -> Result<SoilMoistureReading, A::Error> {
        let raw = match self.read_median_raw() {
            Ok(raw) => raw,
            Err(e) => {
                self.record_fault();
                return Err(e);
            }
        };

        let plausible =
            (self.config.min_plausible_raw..=self.config.max_plausible_raw).contains(&raw);
        if plausible {
            self.consecutive_faults = 0;
            let raw = f32::from(raw);
            self.smoothed_raw = Some(match self.smoothed_raw {
                Some(prev) => prev + SMOOTHING_ALPHA * (raw - prev),
                None => raw,
            });
        } else {
            self.record_fault();
        }

        Ok(SoilMoistureReading {
            raw,
            plausible,
            smoothed_raw: self.smoothed_raw.map(|s| s.round() as u16),
            level: self.level(),
        })
    }

    fn record_fault(&mut self) {
        self.consecutive_faults = self.consecutive_faults.saturating_add(1);
        if self.consecutive_faults >= MAX_CONSECUTIVE_FAULTS {
            self.smoothed_raw = None;
        }
    }

    /// Median of `SAMPLES_PER_READ` raw samples. One-shot reads take
    /// microseconds, so the burst still reflects a single instant.
    fn read_median_raw(&mut self) -> Result<u16, A::Error> {
        let mut samples = [0u16; SAMPLES_PER_READ];
        for sample in &mut samples {
            *sample = self.adc.read_raw()?;
        }
        samples.sort_unstable();
        Ok(samples[SAMPLES_PER_READ / 2])
    }

    /// Linear map from `dry_raw` (0%) to `wet_raw` (100%), clamped.
    /// Capacitive probes read lower when wetter.
    fn to_percentage(&self, raw: f32) -> u8 {
        let dry = f32::from(self.config.dry_raw);
        let range = dry - f32::from(self.config.wet_raw);
        if range <= 0.0 {
            return 0;
        }
        ((dry - raw) / range * 100.0).clamp(0.0, 100.0) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug, PartialEq, Eq)]
    struct Fault;

    /// Scripted ADC: pops one scripted result per `read_raw`; panics when the
    /// script runs out, so a test cannot silently read more than it planned.
    #[derive(Default)]
    struct FakeAdc {
        script: VecDeque<Result<u16, Fault>>,
    }

    impl FakeAdc {
        /// One full burst where every sample is `raw`.
        fn push_burst(&mut self, raw: u16) {
            for _ in 0..SAMPLES_PER_READ {
                self.script.push_back(Ok(raw));
            }
        }
    }

    impl Adc for FakeAdc {
        type Error = Fault;

        fn read_raw(&mut self) -> Result<u16, Fault> {
            self.script.pop_front().expect("script exhausted")
        }
    }

    fn sensor() -> SoilMoisture<FakeAdc> {
        SoilMoisture::new(SoilMoistureConfig::default(), FakeAdc::default())
    }

    fn read_burst(s: &mut SoilMoisture<FakeAdc>, raw: u16) -> SoilMoistureReading {
        s.adc.push_burst(raw);
        s.read().unwrap()
    }

    fn level_value(s: &SoilMoisture<FakeAdc>) -> Option<u8> {
        s.level().map(|l| l.value())
    }

    #[test]
    fn level_is_none_before_first_reading() {
        assert_eq!(sensor().level(), None);
    }

    #[test]
    fn maps_bounds_and_midpoint() {
        // dry 2780 -> 0%, wet 2100 -> 100%, midpoint 2440 -> 50%.
        for (raw, expected) in [(2780, 0), (2440, 50), (2100, 100)] {
            let mut s = sensor();
            let r = read_burst(&mut s, raw);
            assert_eq!(r.level.map(|l| l.value()), Some(expected), "raw={raw}");
            assert_eq!(level_value(&s), Some(expected));
        }
    }

    #[test]
    fn clamps_beyond_bounds() {
        // Plausible but beyond the calibration bounds: clamped, not a fault.
        let mut s = sensor();
        assert_eq!(read_burst(&mut s, 3500).level.map(|l| l.value()), Some(0));
        let mut s = sensor();
        assert_eq!(read_burst(&mut s, 1000).level.map(|l| l.value()), Some(100));
    }

    #[test]
    fn median_rejects_a_glitched_sample() {
        let mut s = sensor();
        s.adc.push_burst(2440);
        // Replace one sample with a glitch.
        s.adc.script[3] = Ok(4095);
        s.adc.script[7] = Ok(0);
        let r = s.read().unwrap();
        assert_eq!(r.raw, 2440);
        assert_eq!(r.level.map(|l| l.value()), Some(50));
    }

    #[test]
    fn smooths_successive_readings() {
        let mut s = sensor();
        // First reading seeds the average.
        assert_eq!(read_burst(&mut s, 2780).smoothed_raw, Some(2780));
        // alpha 0.5: halfway to 2440.
        let r = read_burst(&mut s, 2440);
        assert_eq!(r.smoothed_raw, Some(2610));
        assert_eq!(r.level.map(|l| l.value()), Some(25));
        let r = read_burst(&mut s, 2440);
        assert_eq!(r.smoothed_raw, Some(2525));
        assert_eq!(r.level.map(|l| l.value()), Some(37));
    }

    #[test]
    fn implausible_first_reading_gives_no_level() {
        for raw in [0, 499, 4001, 4095] {
            let mut s = sensor();
            let r = read_burst(&mut s, raw);
            assert!(!r.plausible, "raw={raw}");
            assert_eq!(r.raw, raw);
            assert_eq!(r.level, None);
            assert_eq!(s.level(), None);
        }
    }

    #[test]
    fn plausible_range_edges_are_plausible() {
        for raw in [500, 4000] {
            let mut s = sensor();
            assert!(read_burst(&mut s, raw).plausible, "raw={raw}");
            assert!(s.level().is_some());
        }
    }

    #[test]
    fn faults_keep_the_level_until_the_limit() {
        let mut s = sensor();
        read_burst(&mut s, 2440);
        for _ in 0..MAX_CONSECUTIVE_FAULTS - 1 {
            let r = read_burst(&mut s, 4095);
            assert!(!r.plausible);
            // Smoothed value did not move.
            assert_eq!(r.smoothed_raw, Some(2440));
            assert_eq!(r.level.map(|l| l.value()), Some(50));
        }
        assert_eq!(level_value(&s), Some(50));
    }

    #[test]
    fn consecutive_faults_clear_the_level() {
        let mut s = sensor();
        read_burst(&mut s, 2440);
        for _ in 0..MAX_CONSECUTIVE_FAULTS {
            read_burst(&mut s, 0);
        }
        assert_eq!(s.level(), None);
        // Still None while the fault persists.
        let r = read_burst(&mut s, 0);
        assert_eq!(r.level, None);
        assert_eq!(r.smoothed_raw, None);
    }

    #[test]
    fn recovery_after_reset_starts_fresh() {
        let mut s = sensor();
        read_burst(&mut s, 2780); // 0%
        for _ in 0..MAX_CONSECUTIVE_FAULTS {
            read_burst(&mut s, 0);
        }
        // Not smoothed against the old 2780: 2100 gives 100% directly.
        let r = read_burst(&mut s, 2100);
        assert_eq!(r.smoothed_raw, Some(2100));
        assert_eq!(r.level.map(|l| l.value()), Some(100));
    }

    #[test]
    fn plausible_reading_resets_the_fault_count() {
        let mut s = sensor();
        read_burst(&mut s, 2440);
        for _ in 0..MAX_CONSECUTIVE_FAULTS - 1 {
            read_burst(&mut s, 0);
        }
        read_burst(&mut s, 2440);
        // The count restarted, so this many faults again is still not enough.
        for _ in 0..MAX_CONSECUTIVE_FAULTS - 1 {
            read_burst(&mut s, 0);
        }
        assert_eq!(level_value(&s), Some(50));
    }

    #[test]
    fn adc_error_is_returned_and_counts_as_a_fault() {
        let mut s = sensor();
        read_burst(&mut s, 2440);

        // Error mid-burst: Err returned, level kept, smoothed value untouched.
        s.adc.script.extend([Ok(2440), Err(Fault)]);
        assert_eq!(s.read(), Err(Fault));
        assert_eq!(level_value(&s), Some(50));

        // Drain what is left of the aborted script before the next burst.
        s.adc.script.clear();
        // The aborted burst was fault 1; the rest bring it to the limit.
        for _ in 0..MAX_CONSECUTIVE_FAULTS - 1 {
            s.adc.script.push_back(Err(Fault));
            assert_eq!(s.read(), Err(Fault));
        }
        assert_eq!(s.level(), None);
    }

    #[test]
    fn adc_errors_and_implausible_values_share_the_fault_count() {
        let mut s = sensor();
        read_burst(&mut s, 2440);
        read_burst(&mut s, 0);
        s.adc.script.push_back(Err(Fault));
        assert!(s.read().is_err());
        assert_eq!(level_value(&s), Some(50));
        read_burst(&mut s, 4095);
        assert_eq!(s.level(), None);
    }

    #[test]
    fn adc_error_before_first_reading_leaves_no_level() {
        let mut s = sensor();
        s.adc.script.push_back(Err(Fault));
        assert!(s.read().is_err());
        assert_eq!(s.level(), None);
    }

    #[test]
    fn degenerate_calibration_range_maps_to_zero() {
        let config = SoilMoistureConfig {
            dry_raw: 2000,
            wet_raw: 2000,
            ..Default::default()
        };
        let mut s = SoilMoisture::new(config, FakeAdc::default());
        assert_eq!(read_burst(&mut s, 2000).level.map(|l| l.value()), Some(0));
    }
}
