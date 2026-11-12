//! Matter `RelativeHumidityMeasurement` cluster (id 0x0405) on endpoint 1.
//!
//! `measured_value` reads the latest `SoilMoistureLevel` (0–100) from a
//! [`SharedSoilMoistureLevel`] that the main sensor loop updates, and is null
//! while there is no level (faulty or not-yet-read probe). When the
//! level changes, the handler's background task bumps the cluster's data
//! version and notifies the data model, so subscriptions get a report and
//! data-version-filtered reads return the new value instead of "unchanged".
//! No sensor logic lives here — the value originates in the `SoilMoisture`
//! module and is only wired to the cluster by this handler.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use esp_idf_matter::matter::dm::{Cluster, Dataver, HandlerContext, ReadContext};
use esp_idf_matter::matter::error::Error;
use esp_idf_matter::matter::tlv::Nullable;
use esp_idf_matter::matter::utils::sync::Notification;
use esp_idf_matter::matter::utils::sync::blocking::raw::StdRawMutex;

use rs_matter::with;
use soil_moisture::SoilMoistureLevel;

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

/// The latest `SoilMoistureLevel` (0–100), or `None` when there is no
/// trustworthy reading (before the first plausible one, or after the probe
/// faulted), shared between the sensor loop (writer, plain thread) and the
/// Matter handler (reader, async task on the Matter thread).
///
/// Stored in a single atomic: `NONE` (`u8::MAX`, outside 0–100) stands for
/// `None`. `set` wakes the handler only when the value actually changes. The
/// notification uses a `std` mutex because the writer and the reader live on
/// different threads (rs-matter's default raw mutex is not thread-safe here).
pub struct SharedSoilMoistureLevel {
    value: AtomicU8,
    changed: Notification<StdRawMutex>,
}

/// Sentinel for "no level"; a valid level is at most 100.
const NONE: u8 = u8::MAX;

fn encode(level: Option<SoilMoistureLevel>) -> u8 {
    level.map_or(NONE, |l| l.value())
}

impl SharedSoilMoistureLevel {
    pub fn new(level: Option<SoilMoistureLevel>) -> Self {
        Self {
            value: AtomicU8::new(encode(level)),
            changed: Notification::new(),
        }
    }

    pub fn get(&self) -> Option<SoilMoistureLevel> {
        match self.value.load(Ordering::Relaxed) {
            NONE => None,
            value => Some(SoilMoistureLevel::new(value)),
        }
    }

    /// Store `level`; wake the Matter handler if it differs from the last one.
    pub fn set(&self, level: Option<SoilMoistureLevel>) {
        let value = encode(level);
        if self.value.swap(value, Ordering::Relaxed) != value {
            self.changed.notify();
        }
    }
}

/// Handler for the `RelativeHumidityMeasurement` cluster.
///
/// `measured_value` returns the latest `SoilMoistureLevel` (0–100) from the
/// shared `level` cell, or null when there is none (the probe is faulty or has
/// not produced a plausible reading yet); `min_measured_value` is 0;
/// `max_measured_value` is 100.
/// Each read fetches the current value — nothing is cached in the handler.
/// The background `run` task reports changes (see the module docs).
///
/// The Matter cluster encodes these as `u16` in **hundredths of a percent**
/// (0 = 0.00%, 10000 = 100.00%), so the 0–100 `SoilMoistureLevel` is scaled
/// ×100 at the handler boundary. The sensor domain stays 0–100.
pub struct RelativeHumidityHandler {
    dataver: Dataver,
    level: Arc<SharedSoilMoistureLevel>,
}

impl RelativeHumidityHandler {
    pub fn new(dataver: Dataver, level: Arc<SharedSoilMoistureLevel>) -> Self {
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
        Ok(match self.level.get() {
            Some(level) => Nullable::some(u16::from(level.value()) * HUNDREDTHS),
            None => Nullable::none(),
        })
    }

    fn min_measured_value(&self, _ctx: impl ReadContext) -> Result<Nullable<u16>, Error> {
        Ok(Nullable::some(0))
    }

    fn max_measured_value(&self, _ctx: impl ReadContext) -> Result<Nullable<u16>, Error> {
        Ok(Nullable::some(100 * HUNDREDTHS))
    }

    /// Report every level change: bump the data version (so filtered reads
    /// see a new version) and flag `measured_value` for subscription reports.
    async fn run(&self, ctx: impl HandlerContext) -> Result<(), Error> {
        loop {
            self.level.changed.wait().await;

            self.dataver_changed();
            ctx.notify_attr_changed(
                HUMIDITY_ENDPOINT_ID,
                Self::CLUSTER.id,
                AttributeId::MeasuredValue as _,
            );
        }
    }
}
