# Vendored esp-idf-svc

Source: https://github.com/esp-rs/esp-idf-svc
Revision: `b1c8938751045e82a08bead0b123aa4cf138d147`

This snapshot retains the BLE GATT fix required by this project and adds
`ThreadDriver::start_with_stack_size` / `EspThread::start_with_stack_size` so
the ESP32-H2 can use an 8 KiB OpenThread runner stack. The existing `start`
behavior remains at 12 KiB.
