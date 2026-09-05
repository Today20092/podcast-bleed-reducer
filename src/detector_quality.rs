//! Read-only detector experiment. Neither app nor render opts into this filter.
use crate::{
    AutomixParameters, GainEnvelope, RenderBlocker,
    engine::{gain_envelope, load_and_validate},
    release_gate::{Label, LabelKind},
    vad::{self, Preprocessing},
};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::atomic::AtomicU8};

// The reference contract has five columns, no commas in filenames, and optional
// quoted fields. Keep the free-text notes as one tail field, including commas.
fn read_labels(csv: &str, paths: &[PathBuf], count: usize) -> Result<Vec<Label>, RenderBlocker> {
    let mut lines = csv.lines();
    if lines.next() != Some("track,start_seconds,end_seconds,label,notes") {
        return Err(RenderBlocker("expected reference label CSV header".into()));
    }
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<_> = line
                .splitn(5, ',')
                .map(|field| field.trim_matches('"'))
                .collect();
            let invalid = || RenderBlocker(format!("invalid or unmatched reference label: {line}"));
            if fields.len() != 5 {
                return Err(invalid());
            }
            let track = paths
                .iter()
                .position(|path| path.file_name().and_then(|n| n.to_str()) == Some(fields[0]))
                .ok_or_else(invalid)?;
            let time = |field: &str| -> Result<usize, RenderBlocker> {
                let seconds = field.parse::<f64>().map_err(|_| invalid())?;
                if !seconds.is_finite() || seconds < 0.0 || seconds * 48_000.0 > count as f64 {
                    return Err(invalid());
                }
                Ok((seconds * 48_000.0).round() as usize)
            };
            let start = time(fields[1])?;
            let end = time(fields[2])?;
            if start >= end {
                return Err(invalid());
            }
            let kind = match fields[3] {
                "direct-speech" | "laughter-breath" | "uncertain" => LabelKind::Preserve,
                "bleed" => LabelKind::Bleed,
                "room-noise" => LabelKind::Ignore,
                _ => return Err(invalid()),
            };
            Ok(Label {
                track,
                start,
                end,
                kind,
            })
        })
        .collect()
}

fn measure(samples: &[&[f32]], envelopes: &[GainEnvelope], labels: &[Label]) -> Value {
    let mut preservation_samples = 0;
    let mut changed_preservation_samples = 0;
    let (mut bleed_input, mut bleed_output) = (0.0, 0.0);
    let rows: Vec<_> = labels
        .iter()
        .map(|label| {
            let mut changed = 0;
            let (mut input, mut output) = (0.0, 0.0);
            let mut minimum_gain = 1.0_f32;
            for (offset, &original) in samples[label.track][label.start..label.end]
                .iter()
                .enumerate()
            {
                let gain = envelopes[label.track].gain_at(label.start + offset);
                changed += usize::from(gain != 1.0);
                minimum_gain = minimum_gain.min(gain);
                let original = f64::from(original);
                input += original * original;
                output += (original * f64::from(gain)).powi(2);
            }
            if label.kind == LabelKind::Preserve {
                preservation_samples += label.end - label.start;
                changed_preservation_samples += changed;
            } else if label.kind == LabelKind::Bleed {
                bleed_input += input;
                bleed_output += output;
            }
            json!({"track": label.track, "start_sample": label.start, "end_sample": label.end,
            "kind": format!("{:?}", label.kind), "changed_samples": changed,
            "minimum_gain": minimum_gain, "attenuation_db": attenuation(input, output)})
        })
        .collect();
    let (mut overlap_samples, mut changed_overlap_samples) = (0, 0);
    for (span, tracks) in overlap_spans(labels) {
        overlap_samples += span[1] - span[0];
        changed_overlap_samples += (span[0]..span[1])
            .filter(|&sample| {
                tracks
                    .iter()
                    .any(|&track| envelopes[track].gain_at(sample) != 1.0)
            })
            .count();
    }
    json!({"preservation_samples": preservation_samples,
        "changed_preservation_samples": changed_preservation_samples,
        "overlap_samples": overlap_samples, "changed_overlap_samples": changed_overlap_samples,
        "bleed_attenuation_db": attenuation(bleed_input, bleed_output), "labels": rows})
}

fn attenuation(input: f64, output: f64) -> Option<f64> {
    (input > 0.0 && output > 0.0).then(|| 10.0 * (output / input).log10())
}

fn overlap_spans(labels: &[Label]) -> Vec<([usize; 2], Vec<usize>)> {
    let preserved: Vec<_> = labels
        .iter()
        .filter(|l| l.kind == LabelKind::Preserve)
        .collect();
    let mut edges: Vec<_> = preserved.iter().flat_map(|l| [l.start, l.end]).collect();
    edges.sort_unstable();
    edges.dedup();
    edges
        .windows(2)
        .filter_map(|span| {
            let mut tracks: Vec<_> = preserved
                .iter()
                .filter(|l| l.start <= span[0] && l.end >= span[1])
                .map(|l| l.track)
                .collect();
            tracks.sort_unstable();
            tracks.dedup();
            (tracks.len() >= 2).then_some(([span[0], span[1]], tracks))
        })
        .collect()
}

/// Evaluate supplied aligned WAVs without rendering, modifying, or copying files.
/// The report records all settings; no experimental choice is installed in a project.
pub fn compare(paths: &[PathBuf], csv: &str) -> Result<Value, RenderBlocker> {
    let (stems, validation) = load_and_validate(paths)?;
    let count = stems[0].samples.len();
    let labels = read_labels(csv, paths, count)?;
    if !labels.iter().any(|l| l.kind == LabelKind::Preserve)
        || !labels.iter().any(|l| l.kind == LabelKind::Bleed)
    {
        return Err(RenderBlocker(
            "comparison requires preservation and bleed labels".into(),
        ));
    }
    let samples: Vec<_> = stems.iter().map(|stem| stem.samples.as_slice()).collect();
    let mut comparisons = Vec::new();
    for preprocessing in [Preprocessing::Legacy, Preprocessing::AntiAlias] {
        let facts = vad::prepare(&samples, &AtomicU8::new(0), None, preprocessing)?;
        for threshold in [0.2, 0.5] {
            let parameters = AutomixParameters {
                speech_sensitivity: threshold,
                ..Default::default()
            };
            let regions = facts.regions(
                threshold,
                parameters.bleed_discrimination_db,
                (parameters.speech_padding_ms / 32.0).round() as usize,
            );
            let envelopes: Vec<_> = (0..paths.len())
                .map(|track| {
                    gain_envelope(
                        track,
                        count,
                        &regions,
                        &[],
                        10.0_f32.powf(parameters.attenuation_db / 20.0),
                        parameters.open_ms * 48.0,
                        parameters.close_ms * 48.0,
                    )
                })
                .collect();
            comparisons.push(
                json!({"preparation_id": preprocessing.identity(), "parameters": parameters,
                "region_count": regions.len(), "evidence": measure(&samples, &envelopes, &labels)}),
            );
        }
    }
    // Raw energy is evidence only. Microphone gain and placement differ, and known
    // simultaneous speech cannot be represented by selecting just one track.
    let cross_microphone: Vec<_> = labels
        .iter()
        .map(|label| {
            let rms: Vec<f64> = samples
                .iter()
                .map(|track| {
                    (track[label.start..label.end]
                        .iter()
                        .map(|&s| f64::from(s).powi(2))
                        .sum::<f64>()
                        / (label.end - label.start) as f64)
                        .sqrt()
                })
                .collect();
            let loudest = rms
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .unwrap()
                .0;
            json!({"track": label.track, "start_sample": label.start, "end_sample": label.end,
            "kind": format!("{:?}", label.kind), "rms_by_track": rms, "loudest_track": loudest})
        })
        .collect();
    Ok(
        json!({"paths": paths, "sample_count": count, "sample_rate": 48_000,
        "formats": stems.iter().map(|stem| json!({"bits_per_sample": stem.format.bits_per_sample,
            "format_tag": stem.format.tag, "bext_time_reference": stem.bext.as_ref().map(|b| b.time_reference)})).collect::<Vec<_>>(),
        "validation_warnings": validation.warnings, "comparisons": comparisons,
        "cross_microphone": cross_microphone, "human_listening_performed": false}),
    )
}

fn ownership_metrics(
    samples: &[f32],
    start: usize,
    envelope: &GainEnvelope,
    baseline: &GainEnvelope,
) -> Value {
    let mut counts = [0usize; 6];
    let (mut input, mut output) = (0.0, 0.0);
    let (mut minimum, mut minimum_ratio) = (1.0_f32, 1.0_f32);
    let thresholds = [1.0, 10.0_f32.powf(-0.1 / 20.0), 10.0_f32.powf(-1.0 / 20.0)];
    for (offset, &sample) in samples.iter().enumerate() {
        let gain = envelope.gain_at(start + offset);
        let ratio = gain / baseline.gain_at(start + offset);
        minimum = minimum.min(gain);
        minimum_ratio = minimum_ratio.min(ratio);
        for (i, &threshold) in thresholds.iter().enumerate() {
            counts[i] += usize::from(gain < threshold);
            counts[i + 3] += usize::from(ratio < threshold);
        }
        input += f64::from(sample).powi(2);
        output += (f64::from(sample) * f64::from(gain)).powi(2);
    }
    json!({"sample_count": samples.len(), "below_unity_samples": counts[0],
        "below_minus_0_1_db_samples": counts[1], "below_minus_1_db_samples": counts[2],
        "additional_below_unity_samples": counts[3],
        "additional_below_minus_0_1_db_samples": counts[4], "additional_below_minus_1_db_samples": counts[5],
        "minimum_gain_db": 20.0 * minimum.log10(), "worst_additional_gain_db": 20.0 * minimum_ratio.log10(),
        "input_energy": input, "output_energy": output, "energy_attenuation_db": attenuation(input, output)})
}

/// Compare the actual preview path. Split controls scoring only; automatic estimates
/// use the imported session, as in the app. Human examples must be external to holdout.
pub fn compare_ownership(
    paths: &[PathBuf],
    csv: &str,
    guided: &crate::OwnershipSettings,
    split: &str,
) -> Result<Value, RenderBlocker> {
    use crate::{OwnershipSettings, PreviewCache, audio_io::Stem};
    if !matches!(split, "development" | "holdout" | "all") {
        return Err(RenderBlocker(
            "split must be development, holdout, or all".into(),
        ));
    }
    let stamps = paths
        .iter()
        .map(|path| crate::SourceStamp::read(path))
        .collect::<Result<Vec<_>, _>>()?;
    let mut cache = PreviewCache::default();
    let progress = AtomicU8::new(0);
    let mut previews = Vec::new();
    for (name, threshold, settings) in [
        ("legacy_default", 0.5, OwnershipSettings::default()),
        ("legacy_0_20", 0.2, OwnershipSettings::default()),
        (
            "automatic_joint",
            0.5,
            OwnershipSettings {
                enabled: true,
                ..Default::default()
            },
        ),
        ("guided_joint", 0.5, guided.clone()),
    ] {
        eprintln!("analyzing {name}");
        let parameters = AutomixParameters {
            speech_sensitivity: threshold,
            ..Default::default()
        };
        previews.push((
            name,
            cache.analyze_with_ownership(paths, &[], parameters, &settings, &progress, None)?,
        ));
    }
    let count = previews[0].1.envelopes[0].sample_count();
    let labels: Vec<_> = read_labels(csv, paths, count)?
        .into_iter()
        .filter(|l| match split {
            "development" => l.start < 185 * 48_000,
            "holdout" => l.start >= 185 * 48_000,
            _ => true,
        })
        .collect();
    if labels.is_empty() {
        return Err(RenderBlocker("no labels in scoring split".into()));
    }
    let mut rows = Vec::new();
    // Read one original stem at a time after preparation has discarded its PCM.
    for (track, path) in paths.iter().enumerate() {
        let stem = Stem::read(path)?;
        for label in labels.iter().filter(|l| l.track == track) {
            let metrics = previews
                .iter()
                .map(|(name, preview)| {
                    (
                        (*name).to_owned(),
                        ownership_metrics(
                            &stem.samples[label.start..label.end],
                            label.start,
                            &preview.envelopes[track],
                            &previews[0].1.envelopes[track],
                        ),
                    )
                })
                .collect::<serde_json::Map<_, _>>();
            rows.push(json!({"track": track, "path": path, "start_seconds": label.start as f64 / 48_000.0,
                "end_seconds": label.end as f64 / 48_000.0, "kind": format!("{:?}", label.kind), "metrics": metrics}));
        }
    }
    let mut summaries = serde_json::Map::new();
    for (name, _) in &previews {
        let mut groups = serde_json::Map::new();
        for kind in ["Preserve", "Bleed", "Ignore"] {
            let selected: Vec<_> = rows
                .iter()
                .filter(|r| r["kind"] == kind)
                .map(|r| &r["metrics"][name])
                .collect();
            let mut summary = serde_json::Map::new();
            for key in [
                "sample_count",
                "below_unity_samples",
                "below_minus_0_1_db_samples",
                "below_minus_1_db_samples",
                "additional_below_unity_samples",
                "additional_below_minus_0_1_db_samples",
                "additional_below_minus_1_db_samples",
            ] {
                let total: u64 = selected.iter().map(|m| m[key].as_u64().unwrap()).sum();
                summary.insert(key.into(), json!(total));
            }
            let input: f64 = selected
                .iter()
                .map(|m| m["input_energy"].as_f64().unwrap())
                .sum();
            let output: f64 = selected
                .iter()
                .map(|m| m["output_energy"].as_f64().unwrap())
                .sum();
            summary.insert(
                "energy_attenuation_db".into(),
                json!(attenuation(input, output)),
            );
            for key in ["minimum_gain_db", "worst_additional_gain_db"] {
                summary.insert(
                    key.into(),
                    json!(
                        selected
                            .iter()
                            .map(|m| m[key].as_f64().unwrap())
                            .fold(0.0_f64, f64::min)
                    ),
                );
            }
            groups.insert(kind.into(), json!(summary));
        }
        summaries.insert((*name).into(), json!(groups));
    }
    let mut overlap = Vec::new();
    for (span, tracks) in overlap_spans(&labels) {
        let changed = previews
            .iter()
            .map(|(name, p)| {
                (
                    (*name).into(),
                    json!(
                        (span[0]..span[1])
                            .filter(|&s| tracks.iter().any(|&t| p.envelopes[t].gain_at(s) < 1.0))
                            .count()
                    ),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        overlap.push(json!({"start_sample": span[0], "end_sample": span[1], "tracks": tracks, "changed_samples": changed}));
    }
    if paths
        .iter()
        .map(|path| crate::SourceStamp::read(path))
        .collect::<Result<Vec<_>, _>>()?
        != stamps
    {
        return Err(RenderBlocker("source changed during evaluation".into()));
    }
    Ok(
        json!({"paths": paths, "sources": stamps, "sample_rate": 48_000, "sample_count": count,
        "split": split, "guided_settings": guided, "parameters": AutomixParameters::default(),
        "baseline_for_added_attenuation": "legacy_default", "summaries": summaries, "labels": rows, "overlap": overlap,
        "preparations": cache.stats().preparations, "human_listening_performed": false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ownership_metrics_distinguish_exact_tails_from_added_damage() {
        use crate::{Decision, Region};
        let regions = [Region {
            start: 0,
            end: 4,
            decision: Decision::Owned(vec![0]),
        }];
        let unity = gain_envelope(0, 4, &regions, &[], 0.5, 1.0, 1.0);
        let half = gain_envelope(1, 4, &regions, &[], 0.5, 1.0, 1.0);
        let metrics = ownership_metrics(&[1.0; 4], 0, &half, &unity);
        assert_eq!(metrics["below_unity_samples"], 4);
        assert_eq!(metrics["below_minus_1_db_samples"], 4);
        assert_eq!(metrics["additional_below_minus_1_db_samples"], 4);
        assert!((metrics["energy_attenuation_db"].as_f64().unwrap() + 6.0206).abs() < 0.0001);
        let same = ownership_metrics(&[1.0; 4], 0, &half, &half);
        assert_eq!(same["additional_below_unity_samples"], 0);
    }

    #[test]
    fn measurement_counts_overlap_once_and_measures_applied_gain() {
        use crate::{Decision, Region};
        let samples = [vec![1.0; 10], vec![1.0; 10], vec![1.0; 10]];
        let regions = [Region {
            start: 0,
            end: 10,
            decision: Decision::Owned(vec![0]),
        }];
        let envelopes: Vec<_> = (0..3)
            .map(|track| gain_envelope(track, 10, &regions, &[], 0.5, 1.0, 1.0))
            .collect();
        let labels = [
            Label {
                track: 0,
                start: 0,
                end: 10,
                kind: LabelKind::Preserve,
            },
            Label {
                track: 1,
                start: 0,
                end: 8,
                kind: LabelKind::Preserve,
            },
            Label {
                track: 2,
                start: 2,
                end: 6,
                kind: LabelKind::Preserve,
            },
            Label {
                track: 2,
                start: 6,
                end: 10,
                kind: LabelKind::Bleed,
            },
        ];
        let report = measure(
            &samples.iter().map(Vec::as_slice).collect::<Vec<_>>(),
            &envelopes,
            &labels,
        );
        assert_eq!(report["preservation_samples"], 22);
        assert_eq!(report["changed_preservation_samples"], 12);
        assert_eq!(report["overlap_samples"], 8);
        assert_eq!(report["changed_overlap_samples"], 8);
        assert!((report["bleed_attenuation_db"].as_f64().unwrap() + 6.0206).abs() < 0.0001);
    }

    #[test]
    fn reference_labels_reject_unknown_sources_and_out_of_bounds_times() {
        let paths = vec![std::path::PathBuf::from("a.wav")];
        let parse = |row: &str| {
            read_labels(
                &format!("track,start_seconds,end_seconds,label,notes\n{row}"),
                &paths,
                48_000,
            )
        };
        assert_eq!(
            parse("\"a.wav\",0,1,\"direct-speech\",\"\"").unwrap().len(),
            1
        );
        assert!(parse("b.wav,0,1,bleed,").is_err());
        assert!(parse("a.wav,0,1.1,bleed,").is_err());
        assert!(parse("a.wav,NaN,1,bleed,").is_err());
        assert!(parse("a.wav,1,0,bleed,").is_err());
        assert!(parse("a.wav,0,1,typo,").is_err());
    }
}
