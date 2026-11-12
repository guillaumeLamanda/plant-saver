use esp_idf_svc::hal::adc::attenuation;

/// ADC input attenuation for the soil-moisture channel. The calibration
/// bounds in `soil_moisture::SoilMoistureConfig` are raw counts measured at
/// this attenuation: changing it invalidates them.
pub const SOIL_MOISTURE_ATTENUATION: attenuation::adc_atten_t = attenuation::DB_12;

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
