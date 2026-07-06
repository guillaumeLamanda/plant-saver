use esp_idf_svc::hal::adc::attenuation;

pub struct SoilMoistureConfig {
    pub max_dry: u16,
    pub max_wet: u16,
    pub attenuation: attenuation::adc_atten_t,
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
