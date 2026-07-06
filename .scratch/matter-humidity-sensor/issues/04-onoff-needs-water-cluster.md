Status: ready-for-agent

## Parent

`.scratch/matter-humidity-sensor/PRD.md`

## What to build

Expose a read-only On/Off cluster on endpoint 1 that signals "the plant needs water", and extend the controller integration test (Seam 2, established in slice 03) to cover it.

End-to-end behavior: a Matter controller reads the `on_off` attribute on endpoint 1 and sees `true` when the `SoilMoistureLevel` is below the configured threshold (the plant needs water), `false` otherwise. The attribute is a read-only signal — it is not a writable actuator. This is the hook a future watering automation or actuator will react to. The threshold is the one already held in `config` and already exposed via `SoilMoisture::needs_water()` (from slice 01); this slice wires that operation to a Matter cluster.

Extend the controller integration test from slice 03 to also read the On/Off `on_off` attribute and assert it reflects the threshold (dry reading → on_off true; wet reading → on_off false; boundary at the threshold). No new test seam — reuse Seam 2.

## Acceptance criteria

- [ ] Endpoint 1 carries a read-only On/Off cluster
- [ ] `on_off` is `true` when `SoilMoistureLevel` < configured threshold, `false` otherwise
- [ ] `on_off` is not writable from a controller (read-only signal)
- [ ] Apple Home displays the needs-water state
- [ ] The controller integration test (from slice 03) is extended to read `on_off` and assert it matches the threshold for dry, wet, and boundary sensor readings
- [ ] `cargo test` passes; `cargo build` succeeds

## Blocked by

- `.scratch/matter-humidity-sensor/issues/03-humidity-cluster-controller-test.md` (needs endpoint 1 + the controller test seam to exist, so this slice only extends rather than re-establishes it)
