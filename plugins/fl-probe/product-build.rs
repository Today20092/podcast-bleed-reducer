fn main() {
    println!("cargo:rustc-check-cfg=cfg(cleanslate_podcast)");
    match std::env::var("CARGO_PKG_NAME").as_deref() {
        Ok("cleanslate-podcast") => println!("cargo:rustc-cfg=cleanslate_podcast"),
        Ok("cleanslate-fl-probe") => {}
        _ => panic!("Unsupported CleanSlate adapter package identity"),
    }
}
