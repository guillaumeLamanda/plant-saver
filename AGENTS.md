# AGENTS.md

## Project

ESP32-C6 embedded Rust firmware: soil-moisture sensor + Matter (smart home) stack, cross-compiled for the `riscv32imac-esp-espidf` target. This is not a host Rust project — standard cargo workflows need the ESP toolchain env (see below).

## Toolchain & env

- Uses the custom **`esp` Rust toolchain** (installed via `espup`), pinned in `rust-toolchain.toml`. Stock `rustc` cannot build this repo.
- Every build needs `PATH` (riscv32-esp-elf) from `scripts/export-esp.sh`. **Bare `cargo build` fails.** Either use `mise run <task>` or run `source scripts/export-esp.sh && cargo ...`.
- ESP-IDF version pinned via `ESP_IDF_VERSION=v5.5.3` in `.cargo/config.toml`.
- `esp-idf-svc` is patched to a git rev in `Cargo.toml` `[patch.crates-io]` (crates.io 0.52.1 has a BLE GATT bug). Don't "fix" this by switching back to the crates.io version without re-checking.

## Commands

Prefer `mise run <task>` (sources the ESP env for you):

| Task | Does |
| --- | --- |
| `mise run build` / `release` | debug / release build |
| `mise run check` | `cargo check` (fast) |
| `mise run clippy` | clippy |
| `mise run run` | build + **flash to ESP32-C6**, then monitor |
| `mise run monitor` | attach to serial output only |
| `mise run test` | build tests + **flash to ESP32-C6**, run on-device |

- `cargo run`, `mise run run`, and `mise run test` all flash hardware. They need a physical ESP32-C6 on `ESPFLASH_PORT` (default `/dev/tty.usbserial-0001`, set in `mise.toml` + `.cargo/config.toml`). The `espflash flash --monitor` runner is configured in `.cargo/config.toml`.
- `cargo fmt` is **not** a mise task but CI enforces it — run `cargo fmt --all -- --check` before committing.

## Tests run on hardware, not host

`src/sensors/mod.rs` has an `Adc` trait with `FakeAdc`/`FailingAdc` fakes, but `esp-idf-svc` (→ `esp-idf-sys`, the ESP-IDF C build) is a hard dependency, so test binaries are riscv32 binaries that only execute on the ESP32-C6. `mise run test` flashes and runs them on-device; **there is no host test runner.** `cargo test --no-run` compiles; `cargo test` flashes. (The `mise run test` description claiming "no ESP hardware needed" is wrong.)

## CI

`.github/workflows/rust_ci.yml` runs three matrix jobs: `cargo build --release`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --workspace -- -D warnings`. **No tests in CI.** Clippy warnings are errors.

## Wokwi simulation — not supported for ESP32-C6

1. `pip install wokwi-cli`
2. `mise run release`
3. `wokwi-cli` from repo root
4. The potentiometer in `wokwi/diagram.json` simulates soil moisture; the red LED reflects the needs-water state.

Note: Wokwi does not support ESP32-C6. The `wokwi/` configs are for the legacy ESP32 devkit target.

## Architecture

- `src/main.rs` — entrypoint. Inits ADC (gpio1), spawns the Matter stack on a 20 KB thread, then loops reading soil moisture. **Poll interval is `debug_assertions`-gated: 10 s in debug, 10 min in release** — matters when testing the loop.
- `src/sensors/mod.rs` — `SoilMoisture` (calibration → 0–100 %, adaptive bounds), `SoilMoistureLevel`. Unit tests live here.
- `src/matter_device.rs` — Thread+BLE coex Matter stack using **test DACs** (real VID/PID provisioning not yet wired; see reserved `MatterConfig` in `config.rs`). Endpoint 1 reserved for humidity, 2+ for a watering actuator.
- `src/config.rs` — `SoilMoistureConfig` (dry=4095, wet=1500, threshold=65 %, DB_12 atten) and reserved `MatterConfig`.
- `sdkconfig.defaults` + `partitions.csv` — 8 MB flash, custom partition table with a 3 MB factory app (Matter+BLE needs >1 MB), **Bluedroid** BLE (not NimBLE), FreeRTOS 1000 Hz. Editing flash/BLE/partition config here changes the build.
- `build.rs` — just `embuild::espidf::sysenv::output()`; esp-idf-sys does the rest.

## Domain language, issues, triage

- `CONTEXT.md` is the domain glossary (`SoilMoisture`, `SoilMoistureLevel`, `ADC adapter`, `Calibration`). Use these terms; don't drift to synonyms. Consumption rules in `docs/agents/domain.md`.
- Issues live as markdown under `.scratch/<feature>/` (`PRD.md` + `issues/NN-slug.md`). See `docs/agents/issue-tracker.md`.
- Triage roles: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.
