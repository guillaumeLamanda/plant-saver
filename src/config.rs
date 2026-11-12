use esp_idf_svc::hal::adc::attenuation;

/// Raw ADC samples taken per reading; their median is the reading, so a single
/// glitched sample does not show up in the level. Odd, so the median is one
/// sample.
pub const SAMPLES_PER_READ: usize = 9;

/// Weight of a new reading in the exponential moving average over readings
/// (0 < α ≤ 1). 0.5 halves the effect of a one-off odd reading while a real
/// change (watering) shows up within two or three polls.
pub const SMOOTHING_ALPHA: f32 = 0.5;

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
    /// Readings outside this raw range are treated as faults (probe
    /// disconnected, shorted or unpowered): they are logged and ignored, and
    /// the level keeps its last value.
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
            dry_raw: 2780,
            wet_raw: 2100,
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
