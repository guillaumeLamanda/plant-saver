fn main() {
    // esp-idf-sys forwards the MCU as a cfg (e.g. `esp32c6`) via sysenv.
    println!("cargo::rustc-check-cfg=cfg(esp32c6)");
    embuild::espidf::sysenv::output();
}
