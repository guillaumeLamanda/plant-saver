use esp_idf_svc::hal::adc::attenuation;

pub struct SoilMoistureConfig {
    pub max_dry: u16,
    pub max_wet: u16,
    pub attenuation: attenuation::adc_atten_t,
    /// Reserved for the read-only "needs water" On/Off cluster (slice 04).
    #[allow(dead_code)]
    pub needs_water_threshold: u8,
}

impl Default for SoilMoistureConfig {
    fn default() -> Self {
        Self {
            max_dry: 4095,
            max_wet: 1500,
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
