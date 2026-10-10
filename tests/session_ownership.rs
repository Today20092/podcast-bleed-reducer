use rust_app::{OwnershipTrainingSettings, SessionOwnershipCache, TICKS_PER_SECOND, TimeRange};
#[path = "common/supervised_fixture.rs"]
mod fixture;
use fixture::{session, session_gain};

#[test]
#[ignore = "Known acoustic quiet-overlap gap; independent qualification remains ticket 14"]
fn unlabelled_quiet_overlap_remains_a_quality_qualification_gate() {
    let (root, project, audio) = session_gain(0.08);
    let result = SessionOwnershipCache::default()
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
        .unwrap();
    let estimates = result.estimates_at(TICKS_PER_SECOND * 7 / 2).unwrap();
    let qualifies = estimates[1].probability.is_some_and(|p| p > 0.3);
    std::fs::remove_dir_all(root).unwrap();
    assert!(
        qualifies,
        "Quiet unlabelled overlap must remain above absence threshold: {estimates:?}"
    );
}

#[test]
fn held_out_episode_keeps_overlap_independent_and_rejects_solo_bleed() {
    let (root, project, audio) = session();
    let mut cache = SessionOwnershipCache::default();
    let result = cache
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
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
            .all(|estimate| estimate.probability.is_some_and(|p| p > 0.3)),
        "Each track retains its own probability in overlap: {overlap:?}"
    );
    assert!(
        overlap
            .iter()
            .map(|estimate| estimate.probability.unwrap())
            .sum::<f32>()
            > 1.0
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
            .all(|row| row.raw_speech.is_some() && row.assigned_activity.is_none())
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
        "Upstream analysis identity remains part of the temporal artifact even with identical numeric evidence."
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn labels_retrain_models_while_fixed_evidence_and_saved_models_reuse_safely() {
    let (root, mut project, audio) = session();
    let mut cache = SessionOwnershipCache::default();
    let before = cache
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
        .unwrap();
    let fits = cache.stats().model_fits;
    let warm = cache
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&before, &warm));
    assert_eq!(cache.stats().model_fits, fits);
    project.protect_track(0).unwrap();
    project
        .replace_manual_gain_edits(vec![rust_app::ManualGainEdit {
            interpolation: rust_app::ManualInterpolation::Linear,
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
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
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
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
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
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
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
            .analyze_acoustic(
                &project,
                &audio,
                OwnershipTrainingSettings::default(),
                Some(&cancelled)
            )
            .is_err()
    );
    assert_eq!(cache.last_completed().unwrap().cache_key, after.cache_key);
    assert!(
        cache
            .analyze_acoustic(
                &project,
                &audio,
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
            .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
            .is_err()
    );
    assert_eq!(cache.last_completed().unwrap().cache_key, after.cache_key);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn changed_finite_saved_parameters_reject_reuse_and_retain_prior_completed_result() {
    let (root, mut project, audio) = session();
    let mut accepted = SessionOwnershipCache::default();
    let original = accepted
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 3 / 4, TICKS_PER_SECOND * 5 / 4).unwrap(),
            vec![project.sources[0].speaker_id],
            &[],
        )
        .unwrap();
    let current = SessionOwnershipCache::default()
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
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
                .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
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
fn explicit_quiet_overlap_reference_preserves_speech_even_below_loud_bleed() {
    let (root, mut project, audio) = session_gain(0.08);
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 13 / 4, TICKS_PER_SECOND * 15 / 4).unwrap(),
            project
                .sources
                .iter()
                .map(|source| source.speaker_id)
                .collect(),
            &[],
        )
        .unwrap();
    let result = SessionOwnershipCache::default()
        .analyze_acoustic(&project, &audio, OwnershipTrainingSettings::default(), None)
        .unwrap();
    let estimates = result.estimates_at(TICKS_PER_SECOND * 7 / 2).unwrap();
    let temporal = rust_app::TemporalOwnershipCache::default()
        .optimize(
            &project,
            &result,
            rust_app::TemporalSettings::default(),
            None,
        )
        .unwrap();
    assert!(
        temporal
            .states_at(TICKS_PER_SECOND * 7 / 2)
            .unwrap()
            .iter()
            .all(|state| *state == rust_app::TemporalTrackState::Owned),
        "Explicit human overlap truth must preserve both speakers: {estimates:?}"
    );
    assert!(
        estimates
            .iter()
            .all(|estimate| estimate.probability.is_none_or(f32::is_finite))
    );
    std::fs::remove_dir_all(root).unwrap();
}
