//! Headless first-use acquisition and offline CPU deployment qualification.
use rust_app::model::{self, ModelCache, NEMOTRON, NativeRuntime};
use std::sync::atomic::AtomicBool;

fn main() {
    if let Err(error) = check() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn check() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let mut download = false;
    let mut runtime = model::runtime_directory()?;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--download" => download = true,
            "--runtime" => { index += 1; runtime = arguments.get(index).ok_or("--runtime needs a package bin directory")?.into(); },
            _ => return Err("Usage: model-check [--download] [--runtime DIRECTORY]; default is offline verification/inference".into()),
        }
        index += 1;
    }
    println!(
        "Required weights: Nemotron-3-Diarization q8_0, {} bytes, OpenMDW-1.1",
        NEMOTRON.bytes
    );
    println!(
        "Model revision: {}\nSHA-256: {}\nAgreement: https://openmdw.ai/license/1-1/",
        model::MODEL_REVISION,
        NEMOTRON.sha256
    );
    let cache = ModelCache::default_location()?;
    if download {
        let mut reported = 0;
        cache.download_required(&AtomicBool::new(false), |bytes| {
            if bytes.saturating_sub(reported) >= 4 * 1024 * 1024 || bytes == NEMOTRON.bytes {
                println!("Downloaded {bytes}/{}", NEMOTRON.bytes);
                reported = bytes;
            }
        })?;
    }
    let runtime = NativeRuntime::open(&runtime)?;
    let model = runtime.load_model(&cache)?;
    let mut stream = model.stream()?;
    stream.push(&vec![0.0; 16_000])?;
    stream.finish()?;
    let probabilities = stream.probabilities()?;
    if probabilities.frames == 0 {
        return Err("Native CPU inference produced no frames".into());
    }
    println!(
        "Offline CPU inference passed: {} frames, {} speakers, {} seconds/frame",
        probabilities.frames, probabilities.speakers, probabilities.seconds_per_frame
    );
    println!(
        "Runtime provenance: {}",
        serde_json::to_string(runtime.provenance()).map_err(|e| e.to_string())?
    );
    Ok(())
}
