//! Read-only benchmark: preview_benchmark <iterations> <stem.wav> [stem.wav ...]
use rust_app::{AutomixParameters, PreviewCache, Protection};
use std::{collections::BTreeMap, path::PathBuf, sync::atomic::AtomicU8, time::Instant};

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
    let mut timings = BTreeMap::<&str, Vec<f64>>::new();
    for iteration in 0..iterations {
        for control in [
            "attenuation",
            "open",
            "close",
            "sensitivity",
            "bleed",
            "padding",
            "protection",
        ] {
            let alternate = iteration % 2 == 0;
            match control {
                "attenuation" => parameters.attenuation_db = if alternate { -12.0 } else { -18.0 },
                "open" => parameters.open_ms = if alternate { 10.0 } else { 50.0 },
                "close" => parameters.close_ms = if alternate { 100.0 } else { 500.0 },
                "sensitivity" => parameters.speech_sensitivity = if alternate { 0.55 } else { 0.5 },
                "bleed" => parameters.bleed_discrimination_db = if alternate { 8.0 } else { 6.0 },
                "padding" => parameters.speech_padding_ms = if alternate { 320.0 } else { 512.0 },
                _ => {}
            }
            let protections = if control == "protection" && alternate {
                vec![Protection {
                    track: None,
                    start: 0,
                    end: cold.envelopes[0].sample_count().min(48_000),
                }]
            } else {
                vec![]
            };
            let start = Instant::now();
            cache.analyze(&paths, &protections, parameters, &progress, None)?;
            timings
                .entry(control)
                .or_default()
                .push(start.elapsed().as_secs_f64() * 1000.0);
        }
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
            "cold_ms": cold_ms,
            "controls": timings.into_iter().map(|(name, runs)| (name, distribution(runs))).collect::<BTreeMap<_, _>>(),
            "restored_preview_equal": true,
            "preparations": stats.preparations, "inference_frames": stats.inference_frames,
            "ownership_derivations": stats.ownership_derivations,
        }))?
    );
    Ok(())
}
