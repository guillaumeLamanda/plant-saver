# Plant Saver

## Materials

- [ESP32](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32d_esp32-wroom-32u_datasheet_en.pdf)
  - Architecture: xtensia
    - [prepare the environment](https://esp-rs.github.io/book/installation/riscv-and-xtensa.html)

## Tools

- [espup](https://github.com/esp-rs/espup)

## Getting started

- Install rust
- setting up espup

## Commands

- read output: `ESPFLASH_PORT=/dev/tty.usbserial-0001 espflash monitor`
- run: `cargo run`
- build: `cargo build`

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
