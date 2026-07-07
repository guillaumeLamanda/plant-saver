//! Matter cluster handlers for the plant-saver device.
//!
//! Each submodule wires one Matter cluster to the existing sensor/config
//! modules. Slice 03 adds the humidity cluster; slice 04 will add the
//! read-only On/Off "needs water" cluster on the same endpoint.

pub mod humidity;
