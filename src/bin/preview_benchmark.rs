//! Read-only benchmark: preview_benchmark <iterations> <stem.wav> [stem.wav ...]
use rust_app::{AutomixParameters, PreviewCache};
use std::{path::PathBuf, sync::atomic::AtomicU8, time::Instant};

fn distribution(mut milliseconds: Vec<f64>) -> serde_json::Value {
    milliseconds.sort_by(f64::total_cmp);
    serde_json::json!({
        "median_ms": milliseconds[milliseconds.len() / 2],
        "p95_ms": milliseconds[(milliseconds.len() * 95).div_ceil(100) - 1],
        "runs_ms": milliseconds,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let iterations = args
        .next()
        .and_then(|value| value.to_str()?.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .ok_or("usage: preview_benchmark <positive iterations> <stem.wav> [stem.wav ...]")?;
    let paths: Vec<PathBuf> = args.map(PathBuf::from).collect();
    let mut cache = PreviewCache::default();
    let progress = AtomicU8::new(0);
    let mut parameters = AutomixParameters::default();
    let start = Instant::now();
    let cold = cache.analyze(&paths, &[], parameters, &progress, None)?;
    let cold_ms = start.elapsed().as_secs_f64() * 1000.0;
    let mut gain_times = Vec::new();
    let mut ownership_times = Vec::new();
    for iteration in 0..iterations {
        parameters.attenuation_db = if iteration % 2 == 0 { -12.0 } else { -18.0 };
        parameters.open_ms = if iteration % 2 == 0 { 10.0 } else { 50.0 };
        let start = Instant::now();
        cache.analyze(&paths, &[], parameters, &progress, None)?;
        gain_times.push(start.elapsed().as_secs_f64() * 1000.0);
        parameters.speech_sensitivity = if iteration % 2 == 0 { 0.55 } else { 0.5 };
        let start = Instant::now();
        cache.analyze(&paths, &[], parameters, &progress, None)?;
        ownership_times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let restored = cache.analyze(&paths, &[], AutomixParameters::default(), &progress, None)?;
    assert_eq!(
        cold, restored,
        "restoring parameters must reproduce the cold preview"
    );
    let stats = cache.stats();
    assert_eq!(
        stats.preparations, 1,
        "warm previews must not rerun inference"
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "paths": paths, "sample_count": cold.envelopes[0].sample_count(),
            "cold_ms": cold_ms, "gain_only": distribution(gain_times),
            "ownership": distribution(ownership_times), "restored_preview_equal": true,
            "preparations": stats.preparations, "inference_frames": stats.inference_frames,
            "ownership_derivations": stats.ownership_derivations,
        }))?
    );
    Ok(())
}
