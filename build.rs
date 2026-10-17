fn main() {
    println!("cargo:rerun-if-changed=assets/cleanslate.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/cleanslate.ico")
            .compile()
            .expect("compile the CleanSlate Windows icon resource");
    }
}
