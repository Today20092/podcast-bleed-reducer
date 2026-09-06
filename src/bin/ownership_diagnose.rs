//! Read-only control ablation using one prepared cache and a saved session.
use rust_app::{Decision, OwnershipSettings, PreviewCache, Project, Region, SourceStamp};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{fs, io::Write, sync::atomic::AtomicU8, time::Instant};

#[derive(Deserialize)]
struct Case {
    name: String,
    track: usize,
    start_seconds: f64,
    end_seconds: f64,
}

fn decision_counts(regions: &[Region], track: usize, start: usize, end: usize) -> [usize; 4] {
    let mut counts = [0; 4];
    for region in regions {
        let samples = region.end.min(end).saturating_sub(region.start.max(start));
        let bucket = match &region.decision {
            Decision::Owned(owners) if owners.contains(&track) => 0,
            Decision::Owned(_) => 1,
            Decision::Uncertain => 2,
            Decision::NoSpeech => 3,
        };
        counts[bucket] += samples;
    }
    counts
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err(
            "ownership_diagnose <session.cleanslate> <cases.json> <new-output.json>".into(),
        );
    }
    if std::path::Path::new(&args[2]).exists() {
        return Err("output already exists".into());
    }
    // Deserialize directly: no project mutators, autosave, or audio output.
    let session_bytes = fs::read(&args[0])?;
    let project: Project = serde_json::from_slice(&session_bytes)?;
    let case_bytes = fs::read(&args[1])?;
    let cases: Vec<Case> = serde_json::from_slice(&case_bytes)?;
    let paths = project.source_paths();
    for case in &cases {
        if case.track >= paths.len()
            || !case.start_seconds.is_finite()
            || !case.end_seconds.is_finite()
            || case.start_seconds < 0.0
            || case.end_seconds <= case.start_seconds
        {
            return Err("invalid diagnostic interval".into());
        }
    }
    let stamps = paths
        .iter()
        .map(|p| SourceStamp::read(p))
        .collect::<Result<Vec<_>, _>>()?;
    let parameters = project.parameters();
    let mut no_padding = parameters;
    no_padding.speech_padding_ms = 0.0;
    let mut default_gates = parameters;
    default_gates.speech_sensitivity = 0.5;
    default_gates.bleed_discrimination_db = 6.0;
    let mut default_gates_no_padding = default_gates;
    default_gates_no_padding.speech_padding_ms = 0.0;
    let guided = project.ownership_settings().clone();
    let mut cache = PreviewCache::default();
    let progress = AtomicU8::new(0);
    let mut modes = Vec::new();
    for (name, parameters, settings) in [
        ("saved", parameters, guided.clone()),
        ("saved_without_padding", no_padding, guided.clone()),
        ("default_gates", default_gates, guided.clone()),
        (
            "default_gates_without_padding",
            default_gates_no_padding,
            guided,
        ),
        (
            "automatic_saved_gates",
            parameters,
            OwnershipSettings {
                enabled: true,
                ..Default::default()
            },
        ),
    ] {
        eprintln!("analyzing {name}");
        let started = Instant::now();
        let preview = cache.analyze_with_ownership(
            &paths,
            project.protections(),
            parameters,
            &settings,
            &progress,
            None,
        )?;
        let elapsed_ms = started.elapsed().as_millis();
        let mut rows = Vec::new();
        for case in &cases {
            let start = (case.start_seconds * 48_000.0).round() as usize;
            let end = (case.end_seconds * 48_000.0).round() as usize;
            let curve = &preview.envelopes[case.track];
            if end > curve.sample_count() || start == end {
                return Err("diagnostic interval is outside the source".into());
            }
            let mut below_1_db = 0;
            let mut unity = 0;
            let mut minimum = 1.0_f32;
            let mut sum_squared = 0.0_f64;
            let mut longest_open = 0;
            let mut open = 0;
            for sample in start..end {
                let gain = curve.gain_at(sample);
                below_1_db += usize::from(gain < 10.0_f32.powf(-1.0 / 20.0));
                unity += usize::from(gain == 1.0);
                minimum = minimum.min(gain);
                sum_squared += f64::from(gain).powi(2);
                if gain >= 10.0_f32.powf(-0.1 / 20.0) {
                    open += 1;
                    longest_open = longest_open.max(open);
                } else {
                    open = 0;
                }
            }
            let counts = decision_counts(&preview.regions, case.track, start, end);
            rows.push(json!({"name": case.name, "track": case.track, "start_sample": start, "end_sample": end,
                "decision_samples": {"owned_target": counts[0], "owned_other": counts[1], "uncertain": counts[2], "no_speech": counts[3]},
                "unity_samples": unity, "below_minus_1_db_samples": below_1_db,
                "minimum_gain_db": 20.0 * minimum.log10(),
                "time_weighted_gain_db": 10.0 * (sum_squared / (end - start) as f64).log10(),
                "longest_above_minus_0_1_db_seconds": longest_open as f64 / 48_000.0}));
        }
        modes.push(json!({"name": name, "parameters": parameters, "settings": settings, "elapsed_ms": elapsed_ms, "cases": rows}));
    }
    if paths
        .iter()
        .map(|p| SourceStamp::read(p))
        .collect::<Result<Vec<_>, _>>()?
        != stamps
    {
        return Err("source changed during evaluation".into());
    }
    if fs::read(&args[0])? != session_bytes || fs::read(&args[1])? != case_bytes {
        return Err("diagnostic inputs changed during evaluation".into());
    }
    let report = json!({"source_session": args[0], "session": project, "sources": stamps,
        "case_definitions": serde_json::from_slice::<Value>(&case_bytes)?, "modes": modes,
        "preparations": cache.stats().preparations, "human_listening_performed": false,
        "gain_metric_note": "Time-weighted gain, not audio-energy-weighted attenuation. Screenshot ranges are provisional."});
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
        .write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_counts_clip_and_distinguish_preservation_causes() {
        let regions = vec![
            Region {
                start: 0,
                end: 10,
                decision: Decision::Owned(vec![0]),
            },
            Region {
                start: 10,
                end: 20,
                decision: Decision::Owned(vec![1]),
            },
            Region {
                start: 20,
                end: 30,
                decision: Decision::Uncertain,
            },
            Region {
                start: 30,
                end: 40,
                decision: Decision::NoSpeech,
            },
        ];
        assert_eq!(decision_counts(&regions, 0, 5, 35), [5, 10, 10, 5]);
    }
}
