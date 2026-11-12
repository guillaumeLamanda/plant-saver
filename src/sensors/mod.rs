//! The production ADC adapter: reads the soil-moisture probe on the ESP32 ADC.
//! The reading logic lives in the host-testable `soil-moisture` crate.

use esp_idf_svc::{
    hal::adc::{
        ADCCH1, ADCU1,
        oneshot::{AdcChannelDriver, AdcDriver},
    },
    sys::EspError,
};
use soil_moisture::Adc;

pub struct EspAdc<'a> {
    channel: AdcChannelDriver<'a, ADCCH1<ADCU1>, AdcDriver<'a, ADCU1>>,
}

impl<'a> EspAdc<'a> {
    pub fn new(channel: AdcChannelDriver<'a, ADCCH1<ADCU1>, AdcDriver<'a, ADCU1>>) -> Self {
        Self { channel }
    }

    /// Calibrated millivolts for a raw count, using the channel's calibration
    /// scheme. Diagnostic only: the level is computed from raw counts.
    pub fn raw_to_mv(&self, raw: u16) -> Result<u16, EspError> {
        self.channel.raw_to_mv(raw)
    }
}

impl Adc for EspAdc<'_> {
    type Error = EspError;

    /// Uncalibrated raw count, whatever calibration the channel has.
    fn read_raw(&mut self) -> Result<u16, EspError> {
        self.channel.read_raw()
    }
}
