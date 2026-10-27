# Plant Saver

## Materials

- [ESP32-C6](https://www.espressif.com/sites/default/files/documentation/esp32-c6_datasheet_en.pdf)
  - Architecture: RISC-V
    - [prepare the environment](https://esp-rs.github.io/book/installation/riscv-and-xtensa.html)
- ESP32-C6 DevKit (8 MB flash)
- [moisture sensor: Capacitive Soil Moisture Sensor v1.2](https://boutique.semageek.com/en/1522-capacitive-soil-moisture-sensor-v12-3002844163382.html)

## Tools

- [mise](https://mise.jdx.dev) — installs the pinned nightly Rust and `espflash`
- [ldproxy](https://crates.io/crates/ldproxy) — `cargo install ldproxy`

## Getting started

- `mise install` (or install the nightly from `rust-toolchain.toml` with rustup)
- `cargo install ldproxy`
- No `espup`/`esp` toolchain needed: the ESP32-C6 is RISC-V and builds on stock nightly

## Commands

- read output: `espflash monitor` (port from `ESPFLASH_PORT`)
- run: `cargo run`
- build: `cargo build`

## Simulation (Wokwi)

- Install: `pip install wokwi-cli`
- Build firmware: `cargo build --release`
- Run simulation: `wokwi-cli` from repo root
- Adjust the potentiometer to simulate moisture changes
- Web project: <https://wokwi.com/projects/323845300685898322>

## Ressources

[Pants API - Trefle](https://docs.trefle.io/reference/#tag/Plants/operation/getPlant)

- Rust cross compilations
  - <https://sigmaris.info/blog/2019/02/cross-compiling-rust-on-mac-os-for-an-arm-linux-router/>
  - <https://wiki.osdev.org/Raspberry_Pi_Bare_Bones>
  - <https://hackernoon.com/building-a-wireless-thermostat-in-rust-for-raspberry-pi-part-2>
  - <https://amritrathie.vercel.app/posts/2020/03/06/cross-compiling-rust-from-macos-to-raspberry-pi/>
- Raspberry GPIO
  - <https://www.etechnophiles.com/raspberry-pi-zero-gpio-pinout-specifications-programming-language/>
  - <https://docs.rs/gpio/latest/gpio/>
- Matter (smart home universal protocol)
  - <https://project-chip.github.io/connectedhomeip-doc/index.html>
  - <https://github.com/project-chip/connectedhomeip>
  - <https://github.com/project-chip/rs-matter>
  - <https://github.com/project-chip/certification-tool>
  - <https://github.com/project-chip/certification-tool/blob/main/docs/Matter_TH_User_Guide/Matter_TH_User_Guide.adoc>
- ESP32-C6
  - <https://www.espressif.com/sites/default/files/documentation/esp32-c6_datasheet_en.pdf>
  - <https://github.com/esp-rs>
  - <https://esp-rs.github.io/book/installation/index.html>
  - <https://esp-rs.github.io/book/installation/riscv.html>
- ESP32-C6 pinout
  - [ESP-IDF GPIO summary](https://docs.espressif.com/projects/esp-idf/en/v5.5.3/esp32c6/api-reference/peripherals/gpio.html#gpio-summary)
  - [ESP-IDF ADC oneshot driver](https://docs.espressif.com/projects/esp-idf/en/v5.5.3/esp32c6/api-reference/peripherals/adc_oneshot.html)
  - [ESP-IDF ADC calibration driver](https://docs.espressif.com/projects/esp-idf/en/v5.5.3/esp32c6/api-reference/peripherals/adc_calibration.html)
  - [ESP-IDF ADC GPIO/channel mapping](https://github.com/espressif/esp-idf/blob/v5.5.3/components/soc/esp32c6/include/soc/adc_channel.h)

## ESP32-C6 wiring

Connect the capacitive soil-moisture sensor to `3V3`, `GND`, and `GPIO1` (`ADC1_CH1`).
`GPIO1` is ADC-capable and is not a strapping pin.
