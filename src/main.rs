fn main() {
    eframe::run_native(
        "CleanSlate",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::<rust_app::app::CleanSlateApp>::default())),
    )
    .expect("CleanSlate window failed");
}
