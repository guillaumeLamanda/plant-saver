Status: ready-for-agent

## Parent

`.scratch/matter-humidity-sensor/PRD.md`

## What to build

Bring up the Matter stack so the device is commissionable into Apple Home, with no custom clusters yet — only the root node. This is the riskiest slice (rs-matter + ESP32 + Apple Home conformance) and intentionally carries no sensor clusters, so it can be validated on its own before anything hangs off it.

End-to-end behavior: the device boots, initialises the Matter stack with BLE+WiFi coexistence, prints a QR code payload to serial, and can be commissioned into Apple Home by scanning that QR code. After commissioning, the root node is visible to the controller. After a power cycle, the device rejoins its fabric without re-commissioning (NVS persistence). The endpoint model is established here: endpoint 0 is the root node; endpoint 1 is reserved for the humidity sensor (slice 03); endpoint 2+ is reserved for a future watering actuator so endpoint numbers stay stable.

Matter stack changes:
- Add `rs-matter`, `rs-matter-stack`, and the ESP32-specific Matter glue as dependencies, pinned to exact versions (upstream API instability is a known risk).
- Update `sdkconfig.defaults` for BLE + WiFi coexistence: increase the main task stack size, reserve heap for the Matter stack (~140KB), keep `opt-level = "s"`.
- Initialise NVS-backed key-value blob store for fabric data, network credentials, and operational state.
- Run the BLE+WiFi coex event loop; generate and print the QR code payload (setup discriminant + passkey) to serial at boot.
- Use test Device Attestation Credentials (development only).

No `RelativeHumidityMeasurement`, no On/Off, no sensor reads in this slice. Validate Apple Home commissioning here, early — do not wait for later slices.

## Acceptance criteria

- [ ] `rs-matter` + `rs-matter-stack` + ESP32 Matter glue added to the manifest, pinned to exact versions
- [ ] `sdkconfig.defaults` updated for BLE+WiFi coex (stack size, heap budget, `opt-level = "s"`)
- [ ] Device boots the Matter stack and prints a QR code payload to serial
- [ ] Device commissions into Apple Home by scanning the QR code (root node visible)
- [ ] After power cycle, the device rejoins its fabric without re-commissioning (NVS persistence)
- [ ] Endpoint 0 is the root node; endpoint 1 reserved for the sensor; endpoint 2+ reserved for a future actuator
- [ ] Heap usage is logged at runtime (to catch ESP32 memory pressure early)
- [ ] `cargo build` succeeds; no custom clusters present yet

## Blocked by

- `.scratch/matter-humidity-sensor/issues/01-prefactor-sensor-adc1.md` (needs ADC1 + clean sensor module + `log` integration)
