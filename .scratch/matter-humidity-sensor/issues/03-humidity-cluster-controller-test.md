Status: ready-for-agent

## Parent

`.scratch/matter-humidity-sensor/PRD.md`

## What to build

Expose the `SoilMoistureLevel` over Matter as a `RelativeHumidityMeasurement` cluster on endpoint 1, and establish the Matter controller integration test seam (Seam 2 from the PRD) by writing the first end-to-end controller test against this cluster.

End-to-end behavior: a Matter controller (Apple Home, or the test controller) reads the `measured_value` attribute on endpoint 1 and gets the current `SoilMoistureLevel` (0–100, clamped). The value updates as the sensor reads. `min_measured_value` is 0, `max_measured_value` is 100. The cluster handler is pull-based — when a controller reads the attribute, the handler calls the `SoilMoisture` module and returns the current level. No new sensor logic; this slice wires the existing sensor module to a Matter cluster.

This slice also establishes the controller integration test seam: a test binary using the `rs-matter` controller API that commissions the device and reads `RelativeHumidityMeasurement.measured_value`, asserting it reflects the sensor state. Preferred mode is in-process over a virtual transport (no hardware, no network); fall back to a loopback transport if the virtual path proves impractical. This is the single new test seam for the whole Matter surface — slice 04 will extend it.

## Acceptance criteria

- [x] Endpoint 1 carries a `RelativeHumidityMeasurement` cluster
- [x] `measured_value` returns the current `SoilMoistureLevel` (0–100, clamped); `min` = 0, `max` = 100
- [x] `measured_value` updates as the sensor reads (controller sees fresh values — pull-based read of a shared latest-level cell)
- [x] Cluster handler is pull-based, calling the `SoilMoisture` module (no caching, no new sensor logic)
- [ ] Apple Home displays the humidity sensor with a live-updating value _(hardware-verified; not in this environment)_
- [ ] A controller integration test (Seam 2) commissions the device and reads `measured_value`, asserting it reflects the sensor state _(deferred — see issue 05)_
- [ ] The test runs in-process over a virtual transport if feasible, else loopback; no hardware/network required to run it _(deferred — see issue 05)_
- [x] `cargo build` succeeds; `cargo test --no-run` compiles (on-device run needs hardware)

## Blocked by

- `.scratch/matter-humidity-sensor/issues/02-matter-stack-commissioning.md` (needs a commissionable Matter stack + endpoint model)

## Comments

- Slice 03 ships the production cluster + pull-based handler (endpoint 1, `RelativeHumidityMeasurement`, `Arc<AtomicU8>` latest-level cell shared between the sensor loop and the Matter thread). Verified by `cargo check`, `cargo clippy -D warnings`, `cargo fmt --check`, and `cargo test --no-run`.
- The cluster encodes values as `u16` in hundredths of a percent (Matter spec: 0 = 0.00%, 10000 = 100.00%). The handler scales the 0–100 `SoilMoistureLevel` ×100 at the boundary, so `measured_value` is 0–10000 on the wire; `min` = 0, `max` = 10000 (i.e. 0% / 100% in domain terms).
- Humidity Sensor device type 0x0307 declared at revision 1 (introduced in Matter 1.3).
- Seam 2 (the controller integration test) is deferred to `.scratch/matter-humidity-sensor/issues/05-seam2-controller-test.md`: `rs-matter 0.2.0` exposes no public `Controller` API, the in-process `E2eRunner` harness is test-internal (not importable), handler-level unit tests are blocked by `pub(crate)` context types, and there is no host test runner (`esp-idf-svc` is a hard dep; tests flash to xtensa). Decision: ship the cluster now, defer the test per user direction.
- Live subscription updates (push on change) are out of scope for this slice; the handler is pull-based (reads return the current level). Push notifications from the sensor loop into the Matter stack need a separate change-notification path and hardware verification.
