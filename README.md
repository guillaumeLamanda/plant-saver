# Plant Saver

## Materials

- [ESP32](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32d_esp32-wroom-32u_datasheet_en.pdf)
  - Architecture: xtensia
    - [prepare the environment](https://esp-rs.github.io/book/installation/riscv-and-xtensa.html)
- ESP32 H2 Zero
- [moisture sensor: Capacitive Soil Moisture Sensor v1.2](https://boutique.semageek.com/en/1522-capacitive-soil-moisture-sensor-v12-3002844163382.html)

## Tools

- [espup](https://github.com/esp-rs/espup)

## Getting started

- Install rust
- setting up espup

## Commands

- read output: `ESPFLASH_PORT=/dev/tty.usbserial-0001 espflash monitor`
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
- ESP32
  - <https://wokwi.com/projects/323845300685898322>
  - <https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32d_esp32-wroom-32u_datasheet_en.pdf>
  - <https://github.com/esp-rs>
  - <https://esp-rs.github.io/book/installation/index.html>
  - <https://esp-rs.github.io/book/installation/riscv.html>
- ESP32-H2 Zero
  - [Waveshare board documentation and pinout](https://www.waveshare.com/wiki/ESP32-H2-Zero)
  - [Waveshare schematic](https://files.waveshare.com/wiki/ESP32-H2-Zero/ESP32-H2-Zero-Schematic.pdf)
  - [ESP32-H2 datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-h2_datasheet_en.pdf)
  - [ESP-IDF GPIO summary and strapping-pin restrictions](https://docs.espressif.com/projects/esp-idf/en/v5.5.3/esp32h2/api-reference/peripherals/gpio.html#gpio-summary)
  - [ESP-IDF ADC oneshot driver](https://docs.espressif.com/projects/esp-idf/en/v5.5.3/esp32h2/api-reference/peripherals/adc_oneshot.html)
  - [ESP-IDF ADC calibration driver](https://docs.espressif.com/projects/esp-idf/en/v5.5.3/esp32h2/api-reference/peripherals/adc_calibration.html)
  - [ESP-IDF ADC GPIO/channel mapping](https://github.com/espressif/esp-idf/blob/v5.5.3/components/soc/esp32h2/include/soc/adc_channel.h)

## ESP32-H2 Zero wiring

Connect the capacitive soil-moisture sensor to `3V3`, `GND`, and `GPIO1`
(`ADC1_CH0`). `GPIO1` is ADC-capable and is not a strapping pin. Avoid ADC-capable
`GPIO2` and `GPIO3` for this sensor because external circuitry can change their
level while the chip samples its boot configuration during reset.
