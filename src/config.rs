use esp_idf_svc::hal::adc::attenuation;

/// Raw ADC samples taken per reading; their median is the reading, so a single
/// glitched sample neither shows up in the level nor moves the bounds. Odd, so
/// the median is one sample.
pub const SAMPLES_PER_READ: usize = 9;

pub struct SoilMoistureConfig {
    /// Starting raw value for 0% (dry). Must sit below the ADC ceiling (4095):
    /// the bounds only widen, so a ceiling value could never adapt and would
    /// squash the scale. Placeholder until measured on the real sensor in dry
    /// soil (the main loop logs `raw`).
    pub max_dry: u16,
    /// Starting raw value for 100% (wet). Measure with the probe in water.
    pub max_wet: u16,
    /// Readings outside this raw range are treated as faults (probe
    /// disconnected, shorted or unpowered): they still produce a level but
    /// never move the calibration bounds.
    pub min_plausible_raw: u16,
    pub max_plausible_raw: u16,
    pub attenuation: attenuation::adc_atten_t,
    /// Reserved for the read-only "needs water" On/Off cluster (slice 04).
    #[allow(dead_code)]
    pub needs_water_threshold: u8,
}

impl Default for SoilMoistureConfig {
    fn default() -> Self {
        Self {
            max_dry: 2700,
            max_wet: 1500,
            min_plausible_raw: 500,
            max_plausible_raw: 4000,
            attenuation: attenuation::DB_12,
            needs_water_threshold: 65,
        }
    }
}

/// Reserved for real-DAC provisioning; must match the DAC's VID/PID when
/// test DACs are removed (PRD out-of-scope for this phase).
#[allow(dead_code)]
pub struct MatterConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_name: &'static str,
}

impl Default for MatterConfig {
    fn default() -> Self {
        Self {
            vendor_id: 0xFFF1,
            product_id: 0x8001,
            device_name: "plant-saver",
        }
    }
}
