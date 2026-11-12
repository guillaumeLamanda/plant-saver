# AGENTS.md

## Project

ESP32-C6 embedded Rust firmware: soil-moisture sensor + Matter (smart home) stack, cross-compiled for the `riscv32imac-esp-espidf` target (set as the default in `.cargo/config.toml`). The firmware is not a host Rust project; only the ESP-free `crates/soil-moisture` member is tested on the host.

## Workflows: mise is the entry point

Every workflow — local and CI — is a mise task; `mise tasks` lists them. Run `mise run ci` before pushing: it is exactly what CI runs (`fmt:check`, `clippy` with `-D warnings`, `test:host`, `release`). Add any new workflow as a mise task, never as a raw command in CI or the docs.

Git hooks are managed by hk (`hk.pkl`), installed once per clone with `mise run hooks:install`. `pre-commit` runs `fmt` (and formats `*.pkl`) and stages the fixes; `pre-push` runs `fmt:check` + `clippy`. The release build stays CI-only. Skip the hooks with `HK=0`.

Tasks that **flash hardware** (`run`, `monitor`, `test`, `erase`, and plain `cargo run` / `cargo test`) need a physical ESP32-C6 plugged in. espflash auto-detects its port (macOS renames it per USB socket, so no default is hardcoded); force one with `ESPFLASH_PORT` in the gitignored `mise.local.toml`. To check that the on-device tests compile without hardware, use `mise run test:build`. Host tests need no hardware: `mise run test:host`.

## Toolchain & env

- **Single sources of truth:** the nightly lives only in `rust-toolchain.toml` (mise reads it as an idiomatic version file); tool versions in `mise.toml` + `mise.lock`; cargo/ESP-IDF env (`MCU`, `ESP_IDF_VERSION`, target, build-std) only in `.cargo/config.toml` so plain `cargo` works.
- Stock nightly: `riscv32imac-esp-espidf` is tier 3, so `std` is built from source (`build-std` + `rust-src`). No Xtensa `esp` toolchain / `espup`.
- The nightly is pinned on purpose: `nightly-2026-10-26` fails to build `std` for espidf (libc `stat` fields). Renovate skips it; test a newer nightly before bumping by hand.
- `espflash` and `ldproxy` come from GitHub release binaries via mise. `cargo install` run inside this repo picks up the riscv target + build-std from `.cargo/config.toml` and fails.
- `MCU` is forced to `esp32c6` in `.cargo/config.toml`, and `main.rs` has a `compile_error!` for any other chip. If you change `MCU`, `cargo clean -p esp-idf-sys`: ESP-IDF's CMake dir keeps the old target and fails with "CMAKE_TOOLCHAIN_FILE ... does not match IDF_TARGET".
- `esp-idf-sys` downloads ESP-IDF (`v5.5.3`) + `riscv32-esp-elf` GCC into `.embuild/`; no `PATH`/`LIBCLANG_PATH` setup.
- Matter crates come from crates.io (`esp-idf-matter` 0.1, `rs-matter-stack` 0.3, `rs-matter` 0.4, `esp-idf-svc` 0.53). The former `vendor/` patches (BLE GATT fix, Thread dataset restart) are in these releases.
- `rs-matter` uses the mbedTLS crypto backend, which needs `CONFIG_MBEDTLS_HKDF_C=y` in `sdkconfig.defaults` — without it the link fails on `mbedtls_hkdf`.

## Tests

- **Host:** the sensor logic lives in `crates/soil-moisture`, which has no ESP dependencies. Its unit tests (fake `Adc`) run on the host with `mise run test:host`; they are part of `ci` and of CI.
- **Firmware crate:** `esp-idf-svc` (→ the ESP-IDF C build) is a hard dependency, so its test binaries are riscv32 binaries that only execute on the ESP32-C6. **There is no host test runner for it**; `mise run test` flashes hardware, `mise run test:build` only compiles.

## CI

`.github/workflows/rust_ci.yml` installs tools with `jdx/mise-action` and runs `mise run <task>` for `release`, `fmt:check`, `clippy` and `test:host`. `.embuild/` is cached, keyed on `.cargo/config.toml`, `sdkconfig.defaults` and `Cargo.lock`.

## Architecture

- Cargo workspace: root package `plant-saver` (the firmware) + member `crates/soil-moisture`.
- `crates/soil-moisture` — ESP-free sensor domain: `SoilMoistureLevel` (0–100), the `Adc` trait (`read_raw() -> Result<u16, Error>`), `SoilMoistureConfig`, and `SoilMoisture<A: Adc>` (median of a 9-sample burst, plausibility range, exponential smoothing with alpha 0.5, linear dry→wet mapping). `SoilMoistureConfig::default()` holds the production values: fixed bounds `dry_raw=2780` = 0 %, `wet_raw=2100` = 100 % (still an estimate, re-measure after a fresh watering), plausible raw range 500..=4000, `needs_water_threshold=65` reserved for a later slice. The level is **optional**: absent until the first plausible reading, and absent again (smoothing restarts) after 3 consecutive faults (implausible raw value or ADC error).
- `src/main.rs` — entrypoint. Inits ADC (gpio1), spawns the Matter stack on a 20 KB thread, then loops reading soil moisture; the log line shows millivolts next to the raw value. **Poll interval is `debug_assertions`-gated: 10 s in debug, 10 min in release** — matters when testing the loop. If the Matter stack returns or aborts, it logs the error, waits about 10 s and restarts the chip.
- `src/sensors/mod.rs` — `EspAdc`, the production `Adc` implementation over the ESP ADC driver, using the chip's curve calibration. The level is still computed from raw counts; moving the bounds to millivolts needs new measurements.
- `src/matter_device.rs` — Thread+BLE coex Matter stack using **test DACs** (real VID/PID provisioning not yet wired; see reserved `MatterConfig` in `config.rs`). Endpoint 1 reserved for humidity, 2+ for a watering actuator. `RelativeHumidityMeasurement.measured_value` is null while the level is absent.
- `src/config.rs` — `SOIL_MOISTURE_ATTENUATION` (DB_12; the raw bounds were measured at this attenuation) and reserved `MatterConfig`.
- `sdkconfig.defaults` — 8 MB flash, **DIO** flash mode, **Bluedroid** BLE (not NimBLE), OpenThread, FreeRTOS 1000 Hz. Editing it changes the ESP-IDF build. The flash mode must match what espflash writes (DIO by default). With QIO, NVS reads came back blank, so nothing persisted across reboots.
- No custom partition table: flashing uses espflash's default (24 KB NVS at 0x9000, one factory app filling the rest of the 8 MB flash).
- `build.rs` — declares the `esp32c6` cfg and calls `embuild::espidf::sysenv::output()`; esp-idf-sys does the rest.

## Domain language, issues, triage

- `CONTEXT.md` is the domain glossary (`SoilMoisture`, `SoilMoistureLevel`, `ADC adapter`, `Calibration`). Use these terms; don't drift to synonyms. Consumption rules in `docs/agents/domain.md`.
- Issues live as markdown under `.scratch/<feature>/` (`PRD.md` + `issues/NN-slug.md`). See `docs/agents/issue-tracker.md`.
- Triage roles: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.
