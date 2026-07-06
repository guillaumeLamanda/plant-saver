Status: ready-for-agent

## Parent

`.scratch/matter-humidity-sensor/PRD.md`

## What to build

Expose the `SoilMoistureLevel` over Matter as a `RelativeHumidityMeasurement` cluster on endpoint 1, and establish the Matter controller integration test seam (Seam 2 from the PRD) by writing the first end-to-end controller test against this cluster.

End-to-end behavior: a Matter controller (Apple Home, or the test controller) reads the `measured_value` attribute on endpoint 1 and gets the current `SoilMoistureLevel` (0–100, clamped). The value updates as the sensor reads. `min_measured_value` is 0, `max_measured_value` is 100. The cluster handler is pull-based — when a controller reads the attribute, the handler calls the `SoilMoisture` module and returns the current level. No new sensor logic; this slice wires the existing sensor module to a Matter cluster.

This slice also establishes the controller integration test seam: a test binary using the `rs-matter` controller API that commissions the device and reads `RelativeHumidityMeasurement.measured_value`, asserting it reflects the sensor state. Preferred mode is in-process over a virtual transport (no hardware, no network); fall back to a loopback transport if the virtual path proves impractical. This is the single new test seam for the whole Matter surface — slice 04 will extend it.

## Acceptance criteria

- [ ] Endpoint 1 carries a `RelativeHumidityMeasurement` cluster
- [ ] `measured_value` returns the current `SoilMoistureLevel` (0–100, clamped); `min` = 0, `max` = 100
- [ ] `measured_value` updates as the sensor reads (controller sees fresh values)
- [ ] Cluster handler is pull-based, calling the `SoilMoisture` module (no caching, no new sensor logic)
- [ ] Apple Home displays the humidity sensor with a live-updating value
- [ ] A controller integration test (Seam 2) commissions the device and reads `measured_value`, asserting it reflects the sensor state
- [ ] The test runs in-process over a virtual transport if feasible, else loopback; no hardware/network required to run it
- [ ] `cargo test` passes; `cargo build` succeeds

## Blocked by

- `.scratch/matter-humidity-sensor/issues/02-matter-stack-commissioning.md` (needs a commissionable Matter stack + endpoint model)
