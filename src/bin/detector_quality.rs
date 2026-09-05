//! detector_quality <reference-labels.csv> <stem.wav> [stem.wav ...]
use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let labels = args
        .next()
        .ok_or("usage: detector_quality <reference-labels.csv> <stem.wav> [stem.wav ...]")?;
    let paths: Vec<PathBuf> = args.map(PathBuf::from).collect();
    let report = rust_app::detector_quality::compare(&paths, &fs::read_to_string(labels)?)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
