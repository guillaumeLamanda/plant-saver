Status: needs-triage

## Parent

`.scratch/matter-humidity-sensor/PRD.md`

## What to build

Establish Seam 2 — the Matter controller integration test that commissions the device and reads cluster attributes over an in-process virtual transport (no external controller, no network). This is the single new test seam for the whole Matter surface; slice 03 deferred it, slice 04 (On/Off) will extend it once it exists.

Intended behavior: a test binary drives a `rs-matter` device node (endpoint 1 carrying `RelativeHumidityMeasurement` + On/Off) and a controller in the same process over a fake/loopback transport; it commissions (or pre-sets a session), reads `measured_value` / `on_off`, and asserts they track a fake sensor source across dry/wet/boundary readings.

## Why it was deferred from slice 03

Investigation against the pinned deps (`rs-matter = 0.2.0`, `rs-matter-stack = 0.1.0`, `esp-idf-matter` git rev `3dcc213`) found the test seam is not deliverable with the current public API and no hardware:

- **No public `Controller` API.** `rs-matter` exports `dm`, `im`, `pairing`, `transport`, etc. but no controller/commissioner module. A codegen-generated IM *client* API does exist per cluster (`exchange.relative_humidity_measurement().measured_value_read(endpoint)`), but it consumes a `transport::exchange::Exchange`, which requires a commissioned session.
- **The in-process harness is test-internal.** `rs-matter`'s own attribute-read tests use `tests/common/e2e::E2eRunner` (two `Matter` instances over a fake UDP network, pre-set session). That harness lives under `tests/` and is not part of the published crate — it cannot be depended on. Reusing it means porting hundreds of lines of low-level transport/session/TLV code.
- **Handler-level unit test is infeasible.** `ClusterHandler` accessors take `impl ReadContext`, which requires `OperationContext` → `exchange(&self) -> &Exchange` plus `HandlerContext` (`matter()`, `crypto()`, `kv()`, ...). The concrete context types (`ReadContextInstance`, `MatchContextInstance`) are `pub(crate)`, so a fake context cannot be constructed from this crate.
- **No host test runner.** `esp-idf-svc` is a hard dependency, so `cargo test` builds an xtensa binary that only runs on the ESP32 (`mise run test` flashes). The e2e harness is pure-Rust and would run on-chip without network, but it still cannot be executed/iterated without hardware in this environment.

## Options to unblock

1. **Public controller/e2e API.** Track rs-matter for a re-export of `E2eRunner` (or a higher-level `Controller`) and switch to it when available — then write the test against the generated cluster client API.
2. **Port the harness.** Copy `rs-matter`'s `tests/common/e2e` into this repo and wire the generated client API; iterate on hardware via `mise run test`. High effort, high risk.
3. **Upgrade deps.** Move to a dependency version that exposes a public controller API. Conflicts with the AGENTS.md dep-pinning policy (API instability risk); needs explicit re-evaluation.

## Acceptance criteria

- [ ] A test binary commissions (or pre-sets a session with) the device over an in-process virtual/loopback transport — no external controller, no WiFi/BLE, no extra hardware
- [ ] It reads `RelativeHumidityMeasurement.measured_value` and asserts it tracks a fake sensor source (dry → low, wet → high, clamped 0–100); `min` = 0, `max` = 100
- [ ] It reads the On/Off `on_off` attribute (once slice 04 lands) and asserts it matches `SoilMoisture::needs_water` for dry/wet/boundary readings
- [ ] `cargo test --no-run` compiles; `mise run test` runs green on-device
- [ ] No new sensor logic; the test drives the device through the Matter surface only

## Blocked by

- A path forward from the options above (needs triage decision).
- Slice 04 (`.scratch/matter-humidity-sensor/issues/04-onoff-needs-water-cluster.md`) for the On/Off assertions; the cluster read itself can be written as soon as the harness exists.
