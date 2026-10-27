# AGENTS.md

## Project

ESP32-C6 embedded Rust firmware: soil-moisture sensor + Matter (smart home) stack, cross-compiled for the `riscv32imac-esp-espidf` target (set as the default in `.cargo/config.toml`). This is not a host Rust project.

## Workflows: mise is the entry point

Every workflow — local and CI — is a mise task; `mise tasks` lists them. Run `mise run ci` before pushing: it is exactly what CI runs (`fmt:check`, `clippy` with `-D warnings`, `release`). Add any new workflow as a mise task, never as a raw command in CI or the docs.

Tasks that **flash hardware** (`run`, `monitor`, `test`, `erase`, and plain `cargo run` / `cargo test`) need a physical ESP32-C6 on `ESPFLASH_PORT` (default in `mise.toml`; override per machine in the gitignored `mise.local.toml`). To check that tests compile without hardware, use `mise run test:build`.

## Toolchain & env

- **Single sources of truth:** the nightly lives only in `rust-toolchain.toml` (mise reads it as an idiomatic version file); tool versions in `mise.toml` + `mise.lock`; cargo/ESP-IDF env (`MCU`, `ESP_IDF_VERSION`, target, build-std) only in `.cargo/config.toml` so plain `cargo` works.
- Stock nightly: `riscv32imac-esp-espidf` is tier 3, so `std` is built from source (`build-std` + `rust-src`). No Xtensa `esp` toolchain / `espup`.
- The nightly is pinned on purpose: `nightly-2026-10-26` fails to build `std` for espidf (libc `stat` fields). Renovate skips it; test a newer nightly before bumping by hand.
- `espflash` and `ldproxy` come from GitHub release binaries via mise. `cargo install` run inside this repo picks up the riscv target + build-std from `.cargo/config.toml` and fails.
- `MCU` is forced to `esp32c6` in `.cargo/config.toml`, and `main.rs` has a `compile_error!` for any other chip. If you change `MCU`, `cargo clean -p esp-idf-sys`: ESP-IDF's CMake dir keeps the old target and fails with "CMAKE_TOOLCHAIN_FILE ... does not match IDF_TARGET".
- `esp-idf-sys` downloads ESP-IDF (`v5.5.3`) + `riscv32-esp-elf` GCC into `.embuild/`; no `PATH`/`LIBCLANG_PATH` setup.
- Matter crates come from crates.io (`esp-idf-matter` 0.1, `rs-matter-stack` 0.3, `rs-matter` 0.4, `esp-idf-svc` 0.53). The former `vendor/` patches (BLE GATT fix, Thread dataset restart) are in these releases.
- `rs-matter` uses the mbedTLS crypto backend, which needs `CONFIG_MBEDTLS_HKDF_C=y` in `sdkconfig.defaults` — without it the link fails on `mbedtls_hkdf`.

## Tests run on hardware, not host

`esp-idf-svc` (→ the ESP-IDF C build) is a hard dependency, so test binaries are riscv32 binaries that only execute on the ESP32-C6. **There is no host test runner**, and CI runs no tests. There are currently no unit tests (the refactor removed the `Adc` trait and its fakes).

## CI

`.github/workflows/rust_ci.yml` installs tools with `jdx/mise-action` and runs `mise run <task>` for `release`, `fmt:check` and `clippy`. `.embuild/` is cached, keyed on `.cargo/config.toml`, `sdkconfig.defaults` and `Cargo.lock`.

## Architecture

- `src/main.rs` — entrypoint. Inits ADC (gpio1), spawns the Matter stack on a 20 KB thread, then loops reading soil moisture. **Poll interval is `debug_assertions`-gated: 10 s in debug, 10 min in release** — matters when testing the loop.
- `src/sensors/mod.rs` — `SoilMoisture` (calibration → 0–100 %, adaptive bounds), `SoilMoistureLevel`.
- `src/matter_device.rs` — Thread+BLE coex Matter stack using **test DACs** (real VID/PID provisioning not yet wired; see reserved `MatterConfig` in `config.rs`). Endpoint 1 reserved for humidity, 2+ for a watering actuator.
- `src/config.rs` — `SoilMoistureConfig` (dry=4095, wet=1500, threshold=65 %, DB_12 atten) and reserved `MatterConfig`.
- `sdkconfig.defaults` — 8 MB flash, **Bluedroid** BLE (not NimBLE), OpenThread, FreeRTOS 1000 Hz. Editing it changes the ESP-IDF build.
- `partitions.csv` — 3 MB factory app layout, **not currently wired in**: neither `sdkconfig.defaults` nor espflash references it, so flashing uses espflash's default table.
- `build.rs` — declares the `esp32c6` cfg and calls `embuild::espidf::sysenv::output()`; esp-idf-sys does the rest.

## Domain language, issues, triage

- `CONTEXT.md` is the domain glossary (`SoilMoisture`, `SoilMoistureLevel`, `ADC adapter`, `Calibration`). Use these terms; don't drift to synonyms. Consumption rules in `docs/agents/domain.md`.
- Issues live as markdown under `.scratch/<feature>/` (`PRD.md` + `issues/NN-slug.md`). See `docs/agents/issue-tracker.md`.
- Triage roles: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.
