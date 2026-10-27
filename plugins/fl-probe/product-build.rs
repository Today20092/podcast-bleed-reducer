fn main() {
    println!("cargo:rustc-check-cfg=cfg(cleanslate_podcast)");
    println!("cargo:rustc-check-cfg=cfg(cleanslate_acceptance_driver)");
    if std::env::var_os("CARGO_FEATURE_ACCEPTANCE_DRIVER").is_some() {
        assert_eq!(
            std::env::var("CARGO_PKG_NAME").as_deref(),
            Ok("cleanslate-podcast")
        );
        assert_eq!(
            std::env::var("PROFILE").as_deref(),
            Ok("debug"),
            "The acceptance driver must never ship in a release artifact"
        );
        println!("cargo:rustc-cfg=cleanslate_acceptance_driver");
    }
    match std::env::var("CARGO_PKG_NAME").as_deref() {
        Ok("cleanslate-podcast") => println!("cargo:rustc-cfg=cleanslate_podcast"),
        Ok("cleanslate-fl-probe") => {}
        _ => panic!("Unsupported CleanSlate adapter package identity"),
    }
}
