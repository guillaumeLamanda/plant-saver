Status: ready-for-agent

## Parent

`.scratch/matter-humidity-sensor/PRD.md`

## What to build

Prefactor the firmware so the Matter work becomes a series of thin wiring slices. This slice is the foundation: a clean sensor module on the right ADC, with the threshold logic already in place, and serial logging via the `log` crate.

End-to-end behavior: after flashing, the device reads soil moisture from the sensor on ADC1 (GPIO36), converts it to a `SoilMoistureLevel` (0–100, clamped), evaluates whether the plant needs water against a configured threshold, and logs both values to serial on each read cycle. A bad ADC read is handled gracefully (logged, skipped) rather than crashing the loop. The luminosity LED stub and its GPIO usage are removed.

Sensor-module changes:
- Migrate the moisture sensor from ADC2 to ADC1 (GPIO36). This is a hardware rewire plus a code change in which ADC driver and pin the `ADC adapter` binds. ADC2 conflicts with WiFi, which the Matter stack will need.
- Add a `needs_water()` operation to the `SoilMoisture` module, derived from the current `SoilMoistureLevel` against the configured threshold held in `config`. Pure logic, sitting behind the existing `ADC adapter` seam.
- Handle a failed ADC read gracefully so one bad reading doesn't abort the loop.
- Replace serial `println!` with `log::info!` bound to the ESP logging facilities, so application and (later) Matter stack logs share one stream.
- Move all tunables (GPIO pins, calibration bounds, needs-water threshold) into `config`.

This slice does not touch the Matter stack, BLE, WiFi, or any cluster. It only makes the sensor module Matter-ready.

## Acceptance criteria

- [ ] Moisture sensor reads from ADC1 GPIO36 (rewired + code updated); no ADC2 usage remains
- [ ] `SoilMoisture::needs_water()` returns true when `SoilMoistureLevel` is below the configured threshold, false otherwise
- [ ] Unit tests via the `ADC adapter` seam cover: dry reading → needs water true; wet reading → false; boundary at the threshold; existing dry/wet/mid/adaptive-bounds tests still pass
- [ ] A failed ADC read is logged and skipped; the read loop continues
- [ ] All serial output uses `log::info!` (no `println!` in application code)
- [ ] Luminosity LED stub and its GPIO are removed
- [ ] All tunables (pins, calibration bounds, threshold) live in `config`
- [ ] `cargo build` succeeds; flashed device logs `SoilMoistureLevel` + `needs_water` on each cycle
- [ ] Wokwi simulation still runs (update the diagram for the new pin)

## Blocked by

- None — can start immediately
