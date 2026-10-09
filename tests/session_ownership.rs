use rust_app::model::{
    DiarizationEvidence, ProbabilityFrames, ProbabilityHistoryWriter, RuntimeProvenance,
};
use rust_app::{
    OwnershipTrainingSettings, PreparedAnalysisAudio, Project, SessionOwnershipCache,
    TICKS_PER_SECOND, TimeRange,
};
use std::path::PathBuf;
fn session() -> (
    PathBuf,
    Project,
    Vec<PreparedAnalysisAudio>,
    DiarizationEvidence,
) {
    session_gain(1.0)
}
fn session_gain(
    quiet_gain: f32,
) -> (
    PathBuf,
    Project,
    Vec<PreparedAnalysisAudio>,
    DiarizationEvidence,
) {
    let root = std::env::temp_dir().join(format!(
        "cleanslate-ownership-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut state = 7654321_u32;
    let noise: Vec<f32> = (0..64_000)
        .map(|_| {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            ((state >> 8) as f32 / 16777216.0 - 0.5) * 0.2
        })
        .collect();
    let mut paths = vec![];
    for track in 0..2 {
        let path = root.join(format!("speaker-{track}.wav"));
        let mut format = bwavfile::WaveFmt::new_pcm_mono(16000, 32);
        format.tag = 3;
        let mut writer = bwavfile::WaveWriter::create(&path, format)
            .unwrap()
            .audio_frame_writer()
            .unwrap();
        let samples: Vec<_> = noise
            .iter()
            .enumerate()
            .map(|(i, &a)| {
                let b = noise[(i + 700) % noise.len()];
                let active_a = (8000..24000).contains(&i) || (48000..60000).contains(&i);
                let active_b = (32000..44000).contains(&i) || (52000..60000).contains(&i);
                let (a, b) = (
                    if active_a { a } else { a * 0.0001 },
                    if active_b {
                        b * if i >= 52000 { quiet_gain } else { 1.0 }
                    } else {
                        b * 0.0001
                    },
                );
                if track == 0 { a + 0.1 * b } else { b + 0.2 * a }
            })
            .collect();
        writer.write_frames(&samples).unwrap();
        writer.end().unwrap();
        paths.push(path);
    }
    let mut project = Project::create(root.join("project.cleanslate"), paths).unwrap();
    project.set_supplied_alignment(true).unwrap();
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    for (track, (start, end)) in [
        (TICKS_PER_SECOND / 2, TICKS_PER_SECOND * 3 / 2),
        (2 * TICKS_PER_SECOND, TICKS_PER_SECOND * 11 / 4),
    ]
    .into_iter()
    .enumerate()
    {
        project
            .add_reference_label(
                TimeRange::new(start, end).unwrap(),
                vec![project.sources[track].speaker_id],
                &[],
            )
            .unwrap();
    }
    let audio: Vec<_> = project
        .source_paths()
        .iter()
        .map(|path| rust_app::prepare_analysis_audio_in(path, &root.join("copies"), None).unwrap())
        .collect();
    let mut writer = ProbabilityHistoryWriter::create(&root.join("neural.f32")).unwrap();
    let probabilities: Vec<_> = (0..401)
        .flat_map(|frame| {
            (0..8).map(move |channel| {
                let active = match channel {
                    0 => (50..150).contains(&frame) || (300..375).contains(&frame),
                    1 => (200..275).contains(&frame) || (325..375).contains(&frame),
                    _ => false,
                };
                if active { 0.95 } else { 0.01 }
            })
        })
        .collect();
    writer
        .append(&ProbabilityFrames {
            start: 0,
            frames: 401,
            speakers: 8,
            seconds_per_frame: 0.01,
            probabilities,
        })
        .unwrap();
    let evidence = DiarizationEvidence {
        schema: 2,
        sources: audio.iter().map(|a| a.identity.clone()).collect(),
        input_sha256: audio.iter().map(|a| a.audio_sha256().to_string()).collect(),
        history: writer.finish().unwrap(),
        runtime: RuntimeProvenance {
            revision: rust_app::model::RUNTIME_REVISION.into(),
            recipe_sha256: "1".repeat(64),
            binaries: Default::default(),
        },
        model_revision: rust_app::model::MODEL_REVISION.into(),
        model_sha256: rust_app::model::NEMOTRON.sha256.into(),
        preset: "v3-offline".into(),
        mix_recipe: "mean-global-peak-floor-one-v1".into(),
        mix_peak: 1.0,
        mix_gains: vec![0.5; 2],
        sample_count: 64000,
        reported_seconds_per_frame: 0.01,
        compactions: 0,
        maximum_retained_frames: 401,
    };
    (root, project, audio, evidence)
}
#[test]
fn held_out_episode_allows_two_direct_owners_and_rejects_speech_bleed() {
    let (root, project, audio, evidence) = session();
    let mut cache = SessionOwnershipCache::default();
    let result = cache
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    let solo = result.estimates_at(TICKS_PER_SECOND * 31 / 10).unwrap();
    assert!(
        solo[0].probability.unwrap() > 0.8,
        "Unlabeled held-out direct solo: {solo:?}"
    );
    assert!(
        solo[1].probability.unwrap() < 0.2,
        "VAD on bleed must not become the assigned person's ownership: {solo:?}"
    );
    let overlap = result.estimates_at(TICKS_PER_SECOND * 7 / 2).unwrap();
    assert!(
        overlap
            .iter()
            .all(|estimate| estimate.probability.is_some_and(|p| p > 0.7)),
        "Independent owners can coexist: {overlap:?}"
    );
    assert!(
        overlap
            .iter()
            .map(|estimate| estimate.probability.unwrap())
            .sum::<f32>()
            > 1.4
    );
    assert_eq!(cache.stats().fixed_vad_preparations, 1);
    let grid = result.temporal_evidence(&project).unwrap();
    let temporal = rust_app::TemporalOwnershipCache::default()
        .optimize(
            &project,
            &result,
            rust_app::TemporalSettings::default(),
            None,
        )
        .unwrap();
    let inspection = grid.evidence_at(TICKS_PER_SECOND * 31 / 10).unwrap();
    assert_eq!(inspection[0].probability, solo[0].probability);
    assert!(
        inspection
            .iter()
            .all(|row| row.raw_speech.is_some() && row.assigned_activity.is_some())
    );
    assert!(
        temporal
            .states_at(TICKS_PER_SECOND * 7 / 2)
            .unwrap()
            .iter()
            .all(|state| *state != rust_app::TemporalTrackState::Absent)
    );
    let mut changed_provenance = (*result).clone();
    changed_provenance.cache_key = "f".repeat(64);
    let changed_temporal = rust_app::TemporalOwnershipCache::default()
        .optimize(
            &project,
            &changed_provenance,
            rust_app::TemporalSettings::default(),
            None,
        )
        .unwrap();
    assert_ne!(
        changed_temporal.cache_key, temporal.cache_key,
        "Upstream recipe/runtime identity remains part of the temporal artifact even with identical numeric evidence."
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn labels_retrain_models_while_fixed_evidence_and_saved_models_reuse_safely() {
    let (root, mut project, audio, evidence) = session();
    let mut cache = SessionOwnershipCache::default();
    let before = cache
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    let fits = cache.stats().model_fits;
    let warm = cache
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&before, &warm));
    assert_eq!(cache.stats().model_fits, fits);
    project.protect_track(0).unwrap();
    project
        .replace_manual_gain_edits(vec![rust_app::ManualGainEdit {
            id: 1,
            source: rust_app::SourceBinding {
                track: project.sources[1].track_id,
                fingerprint: project.sources[1].fingerprint.clone(),
            },
            points: vec![
                rust_app::EnvelopePoint { time: 0, gain: 0.5 },
                rust_app::EnvelopePoint {
                    time: TICKS_PER_SECOND / 4,
                    gain: 0.5,
                },
            ],
        }])
        .unwrap();
    let gain_edit = cache
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&before, &gain_edit));
    assert_eq!(cache.stats().model_fits, fits);
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 3 / 4, TICKS_PER_SECOND * 5 / 4).unwrap(),
            vec![project.sources[0].speaker_id],
            &[],
        )
        .unwrap();
    let after = cache
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    assert!(
        before.temporal_evidence(&project).is_err(),
        "Old owner estimates must not acquire new human truth implicitly."
    );
    assert_ne!(before.cache_key, after.cache_key);
    assert_eq!(cache.stats().fixed_vad_preparations, 1);
    for (a, b) in before
        .models
        .as_ref()
        .unwrap()
        .models
        .iter()
        .zip(&after.models.as_ref().unwrap().models)
    {
        assert_eq!(
            a.model.diagnostics().rows,
            b.model.diagnostics().rows,
            "Repeated physical truth is not another training example"
        );
    }
    assert_eq!(
        before.estimates_at(TICKS_PER_SECOND * 7 / 2).unwrap()[0].probability,
        after.estimates_at(TICKS_PER_SECOND * 7 / 2).unwrap()[0].probability
    );
    let models = rust_app::SessionOwnershipModels::from_json(
        &after.models.as_ref().unwrap().to_json().unwrap(),
    )
    .unwrap();
    let mut restored = SessionOwnershipCache::default();
    restored.restore_models(models).unwrap();
    let rebuilt = restored
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    assert_eq!(
        restored.stats().model_fits,
        0,
        "A compatible persisted small model rebuilds probabilities without training"
    );
    for (a, b) in after
        .estimates_at(TICKS_PER_SECOND * 7 / 2)
        .unwrap()
        .iter()
        .zip(rebuilt.estimates_at(TICKS_PER_SECOND * 7 / 2).unwrap())
    {
        assert!((a.probability.unwrap() - b.probability.unwrap()).abs() < 1e-6);
    }
    let cancelled = std::sync::atomic::AtomicBool::new(true);
    assert!(
        cache
            .analyze(
                &project,
                &audio,
                &evidence,
                OwnershipTrainingSettings::default(),
                Some(&cancelled)
            )
            .is_err()
    );
    assert_eq!(cache.last_completed().unwrap().cache_key, after.cache_key);
    assert!(
        cache
            .analyze(
                &project,
                &audio,
                &evidence,
                OwnershipTrainingSettings {
                    max_iterations: 1,
                    ..Default::default()
                },
                None
            )
            .is_err()
    );
    assert_eq!(cache.last_completed().unwrap().cache_key, after.cache_key);
    {
        use std::io::{Seek, SeekFrom, Write};
        let mut source = std::fs::OpenOptions::new()
            .write(true)
            .open(&project.source_paths()[0])
            .unwrap();
        source.seek(SeekFrom::End(-4)).unwrap();
        source.write_all(&0.125_f32.to_le_bytes()).unwrap();
    }
    assert!(
        cache
            .analyze(
                &project,
                &audio,
                &evidence,
                OwnershipTrainingSettings::default(),
                None
            )
            .is_err()
    );
    assert_eq!(cache.last_completed().unwrap().cache_key, after.cache_key);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn changed_finite_saved_parameters_reject_reuse_and_retain_prior_completed_result() {
    let (root, mut project, audio, evidence) = session();
    let mut accepted = SessionOwnershipCache::default();
    let original = accepted
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 3 / 4, TICKS_PER_SECOND * 5 / 4).unwrap(),
            vec![project.sources[0].speaker_id],
            &[],
        )
        .unwrap();
    let current = SessionOwnershipCache::default()
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    for parameter in ["coefficient", "intercept", "mask"] {
        let mut json: serde_json::Value =
            serde_json::from_str(&current.models.as_ref().unwrap().to_json().unwrap()).unwrap();
        if parameter == "coefficient" {
            let value = json["models"][0]["model"]["coefficients"][0]
                .as_f64()
                .unwrap();
            json["models"][0]["model"]["coefficients"][0] = serde_json::json!(value + 5.0);
        } else if parameter == "intercept" {
            let value = json["models"][0]["model"]["intercept"].as_f64().unwrap();
            json["models"][0]["model"]["intercept"] = serde_json::json!(value + 5.0);
        } else {
            json["models"][0]["model"]["retained"]
                .as_array_mut()
                .unwrap()
                .remove(0);
            json["models"][0]["model"]["coefficients"]
                .as_array_mut()
                .unwrap()
                .remove(0);
        }
        let decoded = rust_app::SessionOwnershipModels::from_json(&json.to_string());
        if parameter == "mask" {
            assert!(decoded.is_err());
            assert_eq!(
                accepted.last_completed().unwrap().cache_key,
                original.cache_key
            );
            continue;
        }
        let tampered = decoded.unwrap();
        accepted.restore_models(tampered).unwrap();
        assert!(
            accepted
                .analyze(
                    &project,
                    &audio,
                    &evidence,
                    OwnershipTrainingSettings::default(),
                    None
                )
                .is_err(),
            "Stored diagnostics cannot validate changed {parameter} against current human truth"
        );
        assert_eq!(
            accepted.last_completed().unwrap().cache_key,
            original.cache_key
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn ambiguous_identity_and_changed_payloads_preserve_instead_of_guessing_or_replacing_results() {
    use std::io::Write;
    let (root, project, audio, mut evidence) = session();
    let mut cache = SessionOwnershipCache::default();
    let original = cache
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    let original_history = evidence.history.clone();
    let mut writer = ProbabilityHistoryWriter::create(&root.join("ambiguous.f32")).unwrap();
    let probabilities: Vec<_> = (0..401)
        .flat_map(|frame| {
            (0..8).map(move |channel| {
                if channel < 2 && ((50..150).contains(&frame) || (200..275).contains(&frame)) {
                    0.8
                } else {
                    0.01
                }
            })
        })
        .collect();
    writer
        .append(&ProbabilityFrames {
            start: 0,
            frames: 401,
            speakers: 8,
            seconds_per_frame: 0.01,
            probabilities,
        })
        .unwrap();
    evidence.history = writer.finish().unwrap();
    let unresolved = cache
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    assert!(unresolved.models.is_none());
    assert!(
        unresolved
            .estimates_at(TICKS_PER_SECOND)
            .unwrap()
            .iter()
            .all(|estimate| estimate.probability.is_none()
                && estimate
                    .preservation_reason
                    .as_ref()
                    .is_some_and(|reason| reason.contains("identity")))
    );
    assert_eq!(
        cache.last_completed().unwrap().cache_key,
        original.cache_key
    );
    evidence.history = original_history;
    let mut native = std::fs::OpenOptions::new()
        .write(true)
        .open(evidence.history.path())
        .unwrap();
    native.write_all(&0.02_f32.to_le_bytes()).unwrap();
    drop(native);
    assert!(
        cache
            .analyze(
                &project,
                &audio,
                &evidence,
                OwnershipTrainingSettings::default(),
                None
            )
            .is_err()
    );
    assert_eq!(
        cache.last_completed().unwrap().cache_key,
        original.cache_key
    );
    let mut copy = std::fs::OpenOptions::new()
        .write(true)
        .open(audio[0].path())
        .unwrap();
    copy.write_all(&0.125_f32.to_le_bytes()).unwrap();
    drop(copy);
    assert!(
        cache
            .analyze(
                &project,
                &audio,
                &evidence,
                OwnershipTrainingSettings::default(),
                None
            )
            .is_err()
    );
    assert_eq!(
        cache.last_completed().unwrap().cache_key,
        original.cache_key
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn quiet_independent_overlap_carries_preservation_evidence_even_below_loud_bleed() {
    let (root, project, audio, evidence) = session_gain(0.08);
    let result = SessionOwnershipCache::default()
        .analyze(
            &project,
            &audio,
            &evidence,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
    let estimates = result.estimates_at(TICKS_PER_SECOND * 7 / 2).unwrap();
    assert!(
        estimates[1].preservation_reason.is_some(),
        "Quiet independent direct speech must be reviewed/preserved when incoming bleed cannot explain it: {estimates:?}"
    );
    assert!(
        estimates
            .iter()
            .all(|estimate| estimate.probability.is_none_or(f32::is_finite))
    );
    std::fs::remove_dir_all(root).unwrap();
}
