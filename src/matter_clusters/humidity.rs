//! Matter `RelativeHumidityMeasurement` cluster (id 0x0405) on endpoint 1.
//!
//! Pull-based: `measured_value` reads the latest `SoilMoistureLevel` (0–100)
//! from a shared atomic cell that the main sensor loop updates. No sensor
//! logic lives here — the value originates in the `SoilMoisture` module and is
//! only wired to the cluster by this handler.

use esp_idf_matter::matter::dm::{Cluster, Dataver, ReadContext};
use esp_idf_matter::matter::error::Error;
use esp_idf_matter::matter::tlv::Nullable;

use rs_matter::with;

pub use esp_idf_matter::matter::dm::clusters::decl::relative_humidity_measurement::*;

/// Endpoint 1 — the humidity sensor. Endpoint 0 is the root node; endpoint 2+
/// is reserved for a future watering actuator, so this number stays stable per
/// the PRD endpoint layout.
pub const HUMIDITY_ENDPOINT_ID: u16 = 1;

/// The cluster metadata this handler serves: the three mandatory attributes
/// (`measured_value`, `min_measured_value`, `max_measured_value`). The optional
/// `tolerance` attribute is excluded; reading it yields `AttributeNotFound`.
pub const HUMIDITY_CLUSTER: Cluster<'static> =
    FULL_CLUSTER.with_attrs(with!(required)).with_cmds(with!());

/// Pull-based handler for the `RelativeHumidityMeasurement` cluster.
///
/// `measured_value` returns the latest `SoilMoistureLevel` (0–100) from the
/// shared `level` cell; `min_measured_value` is 0; `max_measured_value` is 100.
/// Each read fetches the current value — nothing is cached in the handler.
///
/// The Matter cluster encodes these as `u16` in **hundredths of a percent**
/// (0 = 0.00%, 10000 = 100.00%), so the 0–100 `SoilMoistureLevel` is scaled
/// ×100 at the handler boundary. The sensor domain stays 0–100.
pub struct RelativeHumidityHandler {
    dataver: Dataver,
    level: u8,
}

impl RelativeHumidityHandler {
    pub fn new(dataver: Dataver, level: u8) -> Self {
        Self { dataver, level }
    }

    /// Wrap in the codegen `HandlerAdaptor` that implements the generic
    /// `rs-matter` `Handler` trait.
    pub fn adapt(self) -> HandlerAdaptor<Self> {
        HandlerAdaptor(self)
    }
}

/// Hundredths of a percent per the `RelativeHumidityMeasurement` spec: the
/// `SoilMoistureLevel` (0–100) is scaled ×100 (0–10000). `u8(0..=100) * 100`
/// → `0..=10000`, which fits in `u16`.
const HUNDREDTHS: u16 = 100;

impl ClusterHandler for RelativeHumidityHandler {
    const CLUSTER: Cluster<'static> = HUMIDITY_CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }

    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn measured_value(&self, _ctx: impl ReadContext) -> Result<Nullable<u16>, Error> {
        Ok(Nullable::some(self.level as u16 * HUNDREDTHS))
    }

    fn min_measured_value(&self, _ctx: impl ReadContext) -> Result<Nullable<u16>, Error> {
        Ok(Nullable::some(0))
    }

    fn max_measured_value(&self, _ctx: impl ReadContext) -> Result<Nullable<u16>, Error> {
        Ok(Nullable::some(100 * HUNDREDTHS))
    }
}
