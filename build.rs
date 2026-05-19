fn main() {
    println!("cargo:rustc-check-cfg=cfg(esp_idf_version_major, values(\"4\"))");
    embuild::espidf::sysenv::output();
}
