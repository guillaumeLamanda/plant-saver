Status: ready-for-agent

# PRD — Matter Soil Moisture Sensor

## Problem Statement

As a plant owner, I read my plant's soil moisture from a serial console. That value stays trapped on my desk — I can't see it from anywhere else, and nothing in my home can react to it. I want the soil moisture to show up in my smart home (Apple Home) alongside my other sensors, so that I can monitor the plant from the same app I already use, and so that later an automation can water the plant when it gets dry.

## Solution

Turn the ESP32 plant-saver into a Matter-compliant smart device. Over Matter, the device exposes its `SoilMoistureLevel` (0–100%) as a `RelativeHumidityMeasurement` cluster, plus a read-only On/Off signal that is `true` when the plant needs water (humidity below a configured threshold). The device commissions into Apple Home over BLE, then runs operational traffic over WiFi. After commissioning, the moisture reading and the "needs water" state are visible and queryable from any Matter controller.

## User Stories

1. As a plant owner, I want my plant's soil moisture to appear as a humidity sensor in Apple Home, so that I can see it next to my other home sensors.
2. As a plant owner, I want the moisture value to update over time in Apple Home, so that I can tell whether the plant is drying out without going to the device.
3. As a plant owner, I want a "needs water" indicator in Apple Home, so that I get a clear yes/no signal instead of having to interpret a percentage.
4. As a plant owner, I want the "needs water" threshold to be configurable, so that I can tune it to my plant's needs.
5. As a plant owner, I want to commission the device into Apple Home by scanning a QR code, so that I don't have to manually pair or enter codes.
6. As a plant owner, I want commissioning to happen over BLE and the device to then run over WiFi, so that it works with Apple Home's standard onboarding flow.
7. As a plant owner, I want the device to stay commissioned after a power cycle, so that I don't have to re-pair it every time it reboots.
8. As a plant owner, I want the device to recover gracefully from a failed ADC read, so that one bad reading doesn't crash the Matter stack.
9. As a developer, I want the soil moisture conversion and "needs water" threshold logic tested without hardware, so that I can refactor confidently.
10. As a developer, I want a Matter controller integration test that commissions the device and reads its attributes, so that I can verify the Matter surface without Apple Home.
11. As a developer, I want the integration test to run in-process with a virtual transport when possible, so that it is fast and needs no network or hardware.
12. As a developer, I want the Matter stack and clusters tested via a controller, so that I'm testing the externally observable behavior, not implementation details.
13. As a developer, I want the sensor module to keep using the ADC adapter seam, so that the existing unit tests still pass after the Matter work.
14. As a developer, I want the device to use ADC1 (not ADC2), so that the analog reads don't conflict with WiFi.
15. As a plant owner, I want future support for an automatic watering actuator, so that the plant can water itself eventually.
16. As a developer, I want the endpoint layout to leave room for a future watering endpoint, so that adding an actuator later doesn't require renumbering the sensor endpoint.
17. As a plant owner, I want the device to log over serial, so that I can debug commissioning and runtime issues from my computer.
18. As a developer, I want the QR code payload printed to serial at boot, so that I can commission without extra tooling.
19. As a plant owner, I want the moisture value clamped to 0–100%, so that the Matter attribute never reports an out-of-range value.
20. As a developer, I want to validate Apple Home commissioning early, so that I find rs-matter conformance issues before building the whole feature out.
21. As a developer, I want Matter dependencies pinned to exact versions, so that upstream API changes don't break my build.
22. As a developer, I want heap usage monitored during Matter runtime, so that I catch ESP32 memory pressure before it crashes.

## Implementation Decisions

- **Matter SDK**: use `rs-matter` (pure Rust) plus the higher-level `rs-matter-stack` and the ESP32-specific glue. Chosen for coherence with the existing `esp-idf-svc` Rust stack. Risk: Apple Home conformance — mitigate by validating commissioning against a real controller early.
- **Module layout**: split the firmware into a sensor module, a Matter device module, and a config module. The sensor module owns the ADC hardware and produces a `SoilMoistureLevel`. The Matter device module owns the Matter stack, endpoints, and cluster handlers. The config module holds all tunables (GPIO pins, calibration bounds, needs-water threshold, Matter Vendor ID / Product ID / device name).
- **Sensor module**: reuses the existing `SoilMoisture` module and its `ADC adapter` seam. Add a `needs_water()` operation derived from the current `SoilMoistureLevel` against the configured threshold. Calibration and clamping behavior are unchanged.
- **ADC migration**: move the moisture sensor from ADC2 to ADC1 (GPIO36) because ADC2 conflicts with WiFi, which Matter needs for operational traffic. This is a hardware rewire plus a code change in which ADC driver and pin the sensor module binds.
- **Endpoint model**: endpoint 0 is the root node (standard commissioning clusters). Endpoint 1 is the humidity sensor carrying `RelativeHumidityMeasurement` and a read-only On/Off cluster. Endpoint 2+ is reserved for a future watering actuator so the sensor endpoint number is stable.
- **RelativeHumidityMeasurement cluster**: `measured_value` is the current `SoilMoistureLevel` (0–100); `min_measured_value` is 0; `max_measured_value` is 100. The cluster handler is pull-based — when a controller reads the attribute, the handler calls the sensor module and returns the current level.
- **On/Off cluster**: `on_off` is `true` when the `SoilMoistureLevel` is below the configured threshold (the plant needs water). It is read-only / used as a signal, not as a writable actuator. This is the hook a future watering automation or actuator will react to.
- **Commissioning**: BLE + WiFi coexistence. The device advertises over BLE, the controller scans the QR code, commissioning happens over BLE, and operational traffic then runs over WiFi. The QR code payload is generated from the device's setup discriminant and passkey and printed to serial at boot.
- **Persistence**: fabric data, network credentials, and operational state are stored in NVS flash via the Matter stack's key-value blob store, so commissioning survives reboots.
- **Logging**: replace serial `println!` with `log::info!` bound to the ESP logging facilities, so Matter stack logs and application logs share one stream.
- **Dead code removal**: remove the luminosity LED stub and unused GPIO, since they are unrelated to this feature.
- **Future watering (out of scope now)**: when revisited, the actuator will live on endpoint 2+ and expose either a simple On/Off cluster (for a valve/pump) or the Matter `Irrigation System` device type. The architecture decision (controller-mediated rule vs device-local control) is deferred and does not affect this PRD.

## Testing Decisions

- **What makes a good test here**: test externally observable behavior, not implementation details. For the sensor, that means "given this raw ADC reading, the produced `SoilMoistureLevel` and `needs_water` flag are these values." For the Matter surface, that means "given a commissioned device, a controller reads these attribute values."
- **Seam 1 (existing, reused) — ADC adapter**: the `Adc` trait and `FakeAdc` already let the `SoilMoisture` module be unit-tested without hardware. Extend these tests to cover the new `needs_water` threshold logic (dry reading → needs water true; wet reading → needs water false; boundary at the threshold). No new seam introduced.
- **Seam 2 (new, highest point) — Matter controller integration**: a test binary using the `rs-matter` controller API commissions the device and reads the `RelativeHumidityMeasurement.measured_value` and On/Off `on_off` attributes, asserting they reflect the sensor state. Preferred mode is in-process over a virtual transport (no hardware, no network); fall back to a loopback transport if the virtual path proves impractical. This is the single new seam and it sits at the highest point — the whole device as seen by a Matter controller.
- **Prior art**: the existing unit tests in the sensor module (dry/wet/mid/adaptive-bounds) are the pattern for Seam 1. The Matter stack's own controller examples are the pattern for Seam 2.
- **Not tested at unit level**: the Matter cluster handlers are thin pass-throughs to the sensor module, so they are covered by the controller integration test rather than duplicated in unit tests.

## Out of Scope

- Automatic watering actuator (valve/pump control) and any device-local watering logic.
- The architecture choice for future watering (controller-mediated rule vs device-local control).
- Real Device Attestation Credentials and secure-flash provisioning (test DACs only for this phase).
- OTA firmware updates.
- NVS-persisted calibration (threshold stays compile-time configured for now).
- Matter certification conformance testing.
- Factory provisioning flow and end-user documentation.
- Memory/heap optimization beyond monitoring.

## Further Notes

- This PRD refines and supersedes the design note at `docs/superpowers/specs/2026-05-19-matter-smart-device-design.md` for execution purposes. The design note remains as background.
- Phase 1 is split into five independently-shippable sub-phases (refactor + ADC migration; Matter stack boot; humidity cluster; on/off cluster; test harness) so each can be validated on its own before the next begins.
- Apple Home conformance should be validated at the first opportunity the Matter stack boots and commissions — do not wait until the end of the phase.
- All vocabulary in this PRD follows `CONTEXT.md` (SoilMoisture, SoilMoistureLevel, ADC adapter, Calibration, Soil moisture).
