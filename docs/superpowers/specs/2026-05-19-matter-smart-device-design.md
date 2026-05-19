# Plant Saver — Matter Smart Device Design

## Overview

Turn the ESP32 plant-saver into a Matter-compliant smart device that exposes soil moisture data and a "needs water" alert to Matter controllers (Apple Home). The device uses WiFi for operational traffic and BLE for commissioning.

## Architecture

### Module Structure

```
src/
├── main.rs            # Entry point: init peripherals, spawn tasks
├── sensor.rs          # ADC reading + moisture conversion
├── matter_device.rs   # rs-matter stack setup, cluster handlers, commissioning
└── config.rs          # Constants: GPIO pins, thresholds, calibration, Matter VID/PID
```

### Module Responsibilities

**`sensor.rs`** — owns the ADC hardware:

- `Sensor::new(adc, pin, config) -> Result<Sensor>`
- `Sensor::read_humidity() -> Result<u16>` — returns 0-100%
- `Sensor::needs_water() -> bool` — threshold check against config
- Reads ADC1 (not ADC2, to avoid WiFi conflict)
- Applies calibration, clamps output to 0-100
- On-demand reads (Matter attributes are pull-based)

**`matter_device.rs`** — owns the Matter stack:

- `MatterDevice::new(sensor, peripherals) -> Result<MatterDevice>`
- `MatterDevice::run() -> !` — blocking Matter event loop
- Sets up `EspWifiMatterStack`, registers endpoints, runs `run_coex()`

**`config.rs`** — all tunables:

- GPIO pin assignments: moisture ADC on GPIO36, LED on GPIO12
- ADC calibration constants (MAX_DRY, MAX_WET)
- Humidity threshold
- Matter Vendor ID / Product ID / Device Name

### Data Flow

```
Sensor (ADC1) --> MatterDevice (handlers) --> Matter controllers (Apple Home) over WiFi
```

Matter controllers query attributes → handlers call sensor → sensor reads ADC → value returns through Matter stack.

## Matter Device Model

### Endpoints

| Endpoint | Device Type | Clusters | Purpose |
|----------|-------------|----------|---------|
| 0 | Root Node | BasicInfo, NetworkCommissioning, GeneralCommissioning, etc. | Standard Matter commissioning |
| 1 | Humidity Sensor | RelativeHumidityMeasurement + On/Off | Humidity reading + "needs water" boolean |

### Cluster Details

**RelativeHumidityMeasurement** (Endpoint 1):

- `measured_value`: 0-100% (current soil humidity)
- `min_measured_value`: 0
- `max_measured_value`: 100
- Implemented via `rs_matter::import!(RelativeHumidityMeasurement)` with custom `ClusterAsyncHandler` calling `sensor.read_humidity()`

**On/Off** (Endpoint 1):

- `on_off` attribute: true when humidity < threshold (plant needs water)
- Uses built-in `OnOffHandler` from rs-matter with custom `OnOffHooks` reading sensor state

### Commissioning

- **Mode**: BLE + WiFi coex via `EspWifiMatterStack::run_coex()`
- **Flow**: Device advertises over BLE → controller scans QR code → commissioning over BLE → operational traffic switches to WiFi
- **QR code**: Generated from device setup discriminant + passkey, printed to serial console at boot

### Persistence

- NVS flash via `rs-matter-stack`'s `KvBlobStore`
- Stores: fabric data, network credentials, operational state
- Survives reboots — no re-commissioning after power loss

### Device Attestation

- Phase 1: Test DACs (development only)
- Phase 2+: Real DAC with secure flash storage (out of scope for initial roadmap)

## Dependencies

### New Crates

- `esp-idf-matter` — ESP32-specific Matter stack (WiFi+BLE coex, NVS persistence)
- `rs-matter` — Core Matter protocol library
- `rs-matter-stack` — Higher-level Matter device stack

### Existing Crates (kept)

- `esp-idf-svc` — ESP-IDF services (ADC, GPIO, WiFi, BLE)
- `log` — Logging
- `anyhow` — Error handling (now actually used)

## Roadmap

### Phase 1: Testable Local Setup

**Goal**: Working Matter device on your desk, controllable from Apple Home, with a repeatable test harness.

1. **Refactor current code** into `sensor.rs` / `config.rs` / `main.rs`
   - Extract sensor logic from `main.rs` into `sensor.rs`
   - Move all constants into `config.rs`
   - Replace `println!` with `log::info!`
   - Uncomment `link_patches()`
   - Remove dead code (luminosity LED stub, unused imports)

2. **Migrate ADC2 → ADC1**
   - ADC2 conflicts with WiFi; ADC1 does not
   - Move moisture sensor to ADC1 GPIO36 (VP) — must rewire physically

3. **Add Matter dependencies**
   - Add `esp-idf-matter`, `rs-matter`, `rs-matter-stack` to Cargo.toml
   - Update `sdkconfig.defaults` for BLE + WiFi coex memory requirements
   - Increase main task stack size for Matter

4. **Implement On/Off cluster**
   - Use built-in `OnOffHandler` + custom `OnOffHooks`
   - `on_off` = true when `sensor.needs_water()`

5. **Implement RelativeHumidityMeasurement cluster**
   - Use `rs_matter::import!(RelativeHumidityMeasurement)` for scaffolding
   - Implement `ClusterAsyncHandler` calling `sensor.read_humidity()`

6. **Commissioning flow**
   - BLE + WiFi coex via `EspWifiMatterStack::run_coex()`
   - Print QR code payload to serial console at boot
   - Commission from Apple Home using QR code

7. **Local validation**
   - Commission into Apple Home
   - Verify humidity attribute reads update correctly
   - Verify on/off state reflects "needs water"
   - Test across reboot (NVS persistence)

8. **Integration test harness**
   - Rust test binary using `rs-matter` controller API
   - Commissions device, reads attributes, asserts values
   - Enables repeatable testing without Apple Home

### Phase 2: Production Hardening

- Device Attestation with secure flash (real DAC)
- OTA firmware updates
- NVS-persisted calibration (adjust thresholds without reflash)
- Error recovery (graceful ADC failures, WiFi reconnection, watchdog)
- Memory optimization (heap monitoring, stack size tuning)
- Remove hardcoded WiFi credentials from `.cargo/config.toml`

### Phase 3: Polish & Cert Prep

- Matter certification conformance tests
- Apple Home-specific quirks and workarounds
- Factory provisioning flow
- End-user documentation (flashing + commissioning guide)

## Key Risks

| Risk | Mitigation |
|------|-----------|
| `esp-idf-matter` API instability | Pin exact version; follow upstream updates |
| ESP32 memory pressure (140KB heap needed) | Monitor heap; optimize if needed; `opt-level = "s"` |
| RelativeHumidityMeasurement handler not in examples | Follow `import!` macro pattern from OnOff examples |
| ADC1 requires rewiring from GPIO13 to GPIO36 | Hardware change; plan before firmware work |
| rs-matter doesn't pass all conformance tests yet | Target Apple Home specifically; test against real controller |
| mDNS edge cases | Use default mDNS impl; known workarounds documented |
