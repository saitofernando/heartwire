fn main() {
    println!("cargo:rerun-if-changed=icons/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource
            .set_icon("icons/icon.ico")
            .set("ProductName", "HeartWire")
            .set("FileDescription", "HeartWire")
            .set("CompanyName", "saitofernando")
            .set("LegalCopyright", "Copyright (c) 2026 saitofernando");
        resource.compile().expect("Windows resources");
    }
}
