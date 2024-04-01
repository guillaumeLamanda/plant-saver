# Plant Saver

## Materials

- [ESP32](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32d_esp32-wroom-32u_datasheet_en.pdf)
  - Architecture: xtensia
    - [prepare the environment](https://esp-rs.github.io/book/installation/riscv-and-xtensa.html)

## Tools

- [espup](https://github.com/esp-rs/espup)

## Commands

- read output: `ESPFLASH_PORT=/dev/tty.usbserial-0001 espflash monitor`
- run: `cargo run`
- build: `cargo build`
