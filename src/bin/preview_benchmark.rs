//! Read-only benchmark: preview_benchmark <iterations> [--supplied-alignment] [--ownership settings.json] <source> ...
use rust_app::{AutomixParameters, OwnershipSettings, PreviewCache, Protection};
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
    let supplied: Vec<_> = std::env::args_os().skip(1).collect();
    if supplied
        .first()
        .is_some_and(|arg| arg == "--project" || arg == "--contention")
    {
        return rust_app::benchmark_editor_project_command(&supplied);
    }
    let mut args = std::env::args_os().skip(1);
    let iterations = args
        .next()
        .and_then(|value| value.to_str()?.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .ok_or("usage: preview_benchmark <positive iterations> <stem.wav> [stem.wav ...]")?;
    let mut args = args.peekable();
    let supplied_alignment = args.peek().is_some_and(|arg| arg == "--supplied-alignment");
    if supplied_alignment {
        args.next();
    }
    let ownership = if args.peek().is_some_and(|arg| arg == "--ownership") {
        args.next();
        serde_json::from_slice::<OwnershipSettings>(&std::fs::read(
            args.next()
                .ok_or("--ownership requires a settings JSON file")?,
        )?)?
    } else {
        OwnershipSettings::default()
    };
    let paths: Vec<PathBuf> = args.map(PathBuf::from).collect();
    if paths.is_empty() {
        return Err("at least one source WAV is required".into());
    }
    let mut cache = PreviewCache::default().with_supplied_alignment(supplied_alignment);
    let progress = AtomicU8::new(0);
    let mut parameters = AutomixParameters::default();
    let start = Instant::now();
    let cold =
        cache.analyze_with_ownership(&paths, &[], parameters, &ownership, &progress, None)?;
    let cold_ms = start.elapsed().as_secs_f64() * 1000.0;
    let mut timings = BTreeMap::<&str, Vec<f64>>::new();
    for control in [
        "attenuation",
        "open",
        "close",
        "sensitivity",
        "bleed",
        "padding",
        "protection",
        "calibration",
    ] {
        if control == "calibration" && ownership.examples.is_empty() {
            continue;
        }
        for iteration in 0..iterations {
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
                    end: cold.envelopes[0].duration().min(rust_app::TICKS_PER_SECOND),
                }]
            } else {
                vec![]
            };
            let start = Instant::now();
            let mut settings = ownership.clone();
            if control == "calibration" && alternate {
                settings.examples.clear();
                settings.sources.clear();
            }
            cache.analyze_with_ownership(
                &paths,
                &protections,
                parameters,
                &settings,
                &progress,
                None,
            )?;
            timings
                .entry(control)
                .or_default()
                .push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    let restored = cache.analyze_with_ownership(
        &paths,
        &[],
        AutomixParameters::default(),
        &ownership,
        &progress,
        None,
    )?;
    assert_eq!(
        cold, restored,
        "restoring parameters must reproduce the cold preview"
    );
    let stats = cache.stats();
    let joint = cache.joint_diagnostics().map(|diagnostics| {
        serde_json::json!({
            "feature_preparation_ms": diagnostics.feature_preparation_ms,
            "retained_feature_bytes": diagnostics.retained_feature_bytes,
            "pair_count": diagnostics.pair_count,
        })
    });
    assert_eq!(
        stats.preparations, 1,
        "warm previews must not rerun inference"
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "paths": paths, "sample_count": cold.envelopes[0].sample_count(),
            "cold_ms": cold_ms,
            "ownership": ownership,
            "joint_features": joint,
            "controls": timings.into_iter().map(|(name, runs)| (name, distribution(runs))).collect::<BTreeMap<_, _>>(),
            "restored_preview_equal": true,
            "preparations": stats.preparations, "inference_frames": stats.inference_frames,
            "ownership_derivations": stats.ownership_derivations,
        }))?
    );
    Ok(())
}
