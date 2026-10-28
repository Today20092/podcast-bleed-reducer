use rust_app::{AcousticEvidenceCache, Project, TICKS_PER_SECOND, TimeRange};
use std::path::PathBuf;
fn fixture() -> (PathBuf, Project, Vec<rust_app::PreparedAnalysisAudio>) {
    fixture_scale(1.0)
}
fn fixture_scale(scale: f32) -> (PathBuf, Project, Vec<rust_app::PreparedAnalysisAudio>) {
    fixture_profile(scale, "scalar")
}
fn fixture_profile(
    scale: f32,
    profile: &str,
) -> (PathBuf, Project, Vec<rust_app::PreparedAnalysisAudio>) {
    static NEXT_SESSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "cleanslate-acoustics-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT_SESSION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
    let mut state = 1234567_u32;
    let signals: Vec<f32> = (0..64_000)
        .map(|_| {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            ((state >> 8) as f32 / 16777216.0 - 0.5) * 0.2
        })
        .collect();
    let mut paths = vec![];
    for track in 0..2 {
        let path = root.join(format!("speaker-{track}.wav"));
        let mut format = bwavfile::WaveFmt::new_pcm_mono(16_000, 32);
        format.tag = 3;
        let mut writer = bwavfile::WaveWriter::create(&path, format)
            .unwrap()
            .audio_frame_writer()
            .unwrap();
        let samples: Vec<_> = signals
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let colored = |gain: f32| {
                    gain * (0.6 * signals[i.saturating_sub(64)]
                        + 0.4 * signals[i.saturating_sub(65)])
                };
                let (a, b) = if (8_000..24_000).contains(&i) {
                    let leak = match profile {
                        "colored" => colored(0.2),
                        "below_floor" => colored(0.00001),
                        "noisy" => signals[(i + 500) % signals.len()] * 0.2,
                        "movement" => sample * if i < 16_000 { 0.2 } else { 0.25 },
                        "bad_movement" => sample * if i < 16_000 { 0.05 } else { 0.8 },
                        _ => sample * 0.2,
                    };
                    (sample, leak)
                } else if (32_000..48_000).contains(&i) {
                    (0.1 * sample, sample)
                } else if profile == "colored" && (48_000..56_000).contains(&i) {
                    (
                        sample,
                        colored(0.2) + 0.08 * signals[(i + 500) % signals.len()],
                    )
                } else {
                    (sample * 0.0001, sample * 0.0001)
                };
                scale * if track == 0 { a } else { b }
            })
            .collect();
        writer.write_frames(&samples).unwrap();
        writer.end().unwrap();
        paths.push(path);
    }
    let mut project = Project::create(root.join("session.cleanslate"), paths).unwrap();
    project.set_supplied_alignment(true).unwrap();
    let speakers: Vec<_> = project
        .sources
        .iter()
        .map(|source| source.speaker_id)
        .collect();
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND / 2, TICKS_PER_SECOND * 3 / 2).unwrap(),
            vec![speakers[0]],
            &[],
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(2 * TICKS_PER_SECOND, 3 * TICKS_PER_SECOND).unwrap(),
            vec![speakers[1]],
            &[],
        )
        .unwrap();
    let audio = project
        .source_paths()
        .iter()
        .map(|path| rust_app::prepare_analysis_audio_in(path, &root.join("cache"), None).unwrap())
        .collect();
    (root, project, audio)
}
#[test]
fn trusted_solo_scalar_evidence_is_finite_and_does_not_change_audio() {
    let (root, project, audio) = fixture();
    let hashes: Vec<_> = audio
        .iter()
        .map(|copy| copy.audio_sha256().to_string())
        .collect();
    let mut cache = AcousticEvidenceCache::default();
    let evidence = cache.analyze(&project, &audio, None).unwrap();
    let fit = evidence
        .transfer(project.sources[0].track_id, project.sources[1].track_id)
        .unwrap();
    assert!(
        (fit.level_db.unwrap() + 13.9794).abs() < 1.0,
        "Known 0.2 leakage amplitude should be about -14 dB: {fit:?}"
    );
    assert!(fit.reliability >= 0.8);
    let features = evidence.features_at(TICKS_PER_SECOND).unwrap();
    assert!(
        features
            .iter()
            .flat_map(|frame| frame.normalized_values())
            .all(f32::is_finite)
    );
    assert_eq!(
        hashes,
        audio
            .iter()
            .map(|copy| copy.audio_sha256().to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(cache.stats().fixed_preparations, 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unchanged_acoustic_requests_share_completed_fit() {
    let (root, project, audio) = fixture();
    let mut cache = AcousticEvidenceCache::default();
    let first = cache.analyze(&project, &audio, None).unwrap();
    let second = cache.analyze(&project, &audio, None).unwrap();
    let source = project.sources[0].track_id;
    let target = project.sources[1].track_id;
    assert!(
        std::ptr::eq(
            first.transfer(source, target).unwrap(),
            second.transfer(source, target).unwrap(),
        ),
        "Unchanged requests must share the completed immutable fit instead of copying it"
    );
    assert_eq!(cache.stats().reference_fits, 1);
    assert_eq!(
        first.features_at(TICKS_PER_SECOND).unwrap()[0].normalized_values(),
        second.features_at(TICKS_PER_SECOND).unwrap()[0].normalized_values(),
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn label_edits_invalidate_only_fits_and_cancelled_refits_keep_completed_evidence() {
    let (root, mut project, audio) = fixture();
    let mut cache = AcousticEvidenceCache::default();
    let first = cache.analyze(&project, &audio, None).unwrap();
    let speaker = project.sources[0].speaker_id;
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 3 / 4, TICKS_PER_SECOND * 5 / 4).unwrap(),
            vec![speaker],
            &[],
        )
        .unwrap();
    let cancelled = std::sync::atomic::AtomicBool::new(true);
    assert!(cache.analyze(&project, &audio, Some(&cancelled)).is_err());
    assert_eq!(cache.stats().reference_fits, 1);
    let second = cache.analyze(&project, &audio, None).unwrap();
    assert_ne!(first.cache_key, second.cache_key);
    assert_eq!(cache.stats().fixed_preparations, 1);
    assert_eq!(cache.stats().reference_fits, 2);
    assert!(
        first
            .features_at(TICKS_PER_SECOND)
            .unwrap()
            .iter()
            .flat_map(|frame| frame.normalized_values())
            .all(f32::is_finite)
    );
    let unchanged = cache.analyze(&project, &audio, None).unwrap();
    assert_eq!(unchanged.cache_key, second.cache_key);
    assert_eq!(cache.stats().reference_fits, 2);
    let replacement = root.join("changed.wav");
    std::fs::copy(&project.sources[1].path, &replacement).unwrap();
    project.relink(0, replacement).unwrap();
    assert!(cache.analyze(&project, &audio, None).is_err());
    assert_eq!(cache.stats().fixed_preparations, 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn same_length_analysis_payload_tampering_cannot_reuse_accepted_acoustic_fits() {
    use std::io::{Seek, SeekFrom, Write};
    let (root, project, audio) = fixture();
    let mut cache = AcousticEvidenceCache::default();
    let accepted = cache.analyze(&project, &audio, None).unwrap();
    let mut payload = std::fs::OpenOptions::new()
        .write(true)
        .open(audio[0].path())
        .unwrap();
    payload.seek(SeekFrom::Start(60_000 * 4)).unwrap();
    payload.write_all(&0.125_f32.to_le_bytes()).unwrap();
    drop(payload);
    assert!(
        cache.analyze(&project, &audio, None).is_err(),
        "A finite same-length changed copy must not satisfy the fixed evidence key"
    );
    assert_eq!(cache.stats().reference_fits, 1);
    assert!(accepted.features_at(TICKS_PER_SECOND).is_ok());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn large_finite_audio_has_finite_normalized_features_without_clipping_sources() {
    let (root, project, audio) = fixture_scale(1e20);
    let mut cache = AcousticEvidenceCache::default();
    let evidence = cache.analyze(&project, &audio, None).unwrap();
    assert!(
        evidence
            .features_at(TICKS_PER_SECOND)
            .unwrap()
            .iter()
            .flat_map(|frame| frame.normalized_values())
            .all(f32::is_finite)
    );
    assert!(
        (evidence
            .transfer(project.sources[0].track_id, project.sources[1].track_id)
            .unwrap()
            .level_db
            .unwrap()
            + 13.9794)
            .abs()
            < 1.0
    );
    assert!(
        audio[0]
            .read_frames(16_000..16_010)
            .unwrap()
            .iter()
            .any(|sample| sample.abs() > 1e15)
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn development_transfer_comparison_handles_color_without_explaining_quiet_independent_speech() {
    let (root, mut project, audio) = fixture_profile(1.0, "colored");
    let mut cache = AcousticEvidenceCache::default();
    let evidence = cache.analyze(&project, &audio, None).unwrap();
    let comparison = evidence
        .development_comparisons
        .iter()
        .find(|comparison| {
            comparison.source == project.sources[0].track_id
                && comparison.target == project.sources[1].track_id
        })
        .unwrap();
    assert!(
        comparison.fir_explained > comparison.scalar_explained + 0.1,
        "Colored development leakage should benefit from its short transfer candidate: {comparison:?}"
    );
    let solo = evidence.features_at(TICKS_PER_SECOND).unwrap();
    let overlap = evidence.features_at(TICKS_PER_SECOND * 13 / 4).unwrap();
    let solo = solo[1]
        .bleed
        .iter()
        .find(|feature| feature.source == project.sources[0].track_id)
        .unwrap();
    let overlap = overlap[1]
        .bleed
        .iter()
        .find(|feature| feature.source == project.sources[0].track_id)
        .unwrap();
    assert!(
        overlap.excess_residual_db.unwrap() > solo.excess_residual_db.unwrap() + 6.0,
        "Quiet independent speech must add unexplained residual, not be treated as a copied voice"
    );
    let expected_solo = evidence
        .features_at(TICKS_PER_SECOND)
        .unwrap()
        .iter()
        .map(|frame| frame.normalized_values())
        .collect::<Vec<_>>();
    let expected_overlap = evidence
        .features_at(TICKS_PER_SECOND * 13 / 4)
        .unwrap()
        .iter()
        .map(|frame| frame.normalized_values())
        .collect::<Vec<_>>();
    project
        .add_reference_label(
            TimeRange::new(3 * TICKS_PER_SECOND, TICKS_PER_SECOND * 7 / 2).unwrap(),
            project
                .sources
                .iter()
                .map(|source| source.speaker_id)
                .collect(),
            &[],
        )
        .unwrap();
    let after = cache.analyze(&project, &audio, None).unwrap();
    assert_ne!(after.cache_key, evidence.cache_key);
    assert_eq!(
        after
            .features_at(TICKS_PER_SECOND)
            .unwrap()
            .iter()
            .map(|frame| frame.normalized_values())
            .collect::<Vec<_>>(),
        expected_solo
    );
    assert_eq!(
        after
            .features_at(TICKS_PER_SECOND * 13 / 4)
            .unwrap()
            .iter()
            .map(|frame| frame.normalized_values())
            .collect::<Vec<_>>(),
        expected_overlap
    );
    assert_eq!(
        cache.stats().fixed_preparations,
        1,
        "Overlap truth cannot train a fit to explain independent speech away"
    );
    eprintln!(
        "development comparison: {comparison:?}; solo/overlap residual excess {:?}/{:?}",
        solo.excess_residual_db, overlap.excess_residual_db
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn zero_noisy_and_moving_references_are_explicit_without_nonfinite_or_forced_bleed() {
    let (root, project, audio) = fixture_scale(0.0);
    assert!(
        AcousticEvidenceCache::default()
            .analyze(&project, &audio, None)
            .is_err(),
        "Zero solo energy is unusable, not a zero-leakage fit"
    );
    std::fs::remove_dir_all(root).unwrap();
    for profile in ["noisy", "movement", "bad_movement"] {
        let (root, project, audio) = fixture_profile(1.0, profile);
        let evidence = AcousticEvidenceCache::default()
            .analyze(&project, &audio, None)
            .unwrap();
        let fit = evidence
            .transfer(project.sources[0].track_id, project.sources[1].track_id)
            .unwrap();
        if profile == "movement" {
            assert!(
                fit.level_db.is_some() && fit.reliability >= 0.8,
                "Small movement should retain qualified scalar evidence: {fit:?}"
            );
            assert_eq!(fit.estimator, "scalar-with-delayed-pair-evidence");
        } else {
            assert!(
                fit.level_db.is_none() && fit.reliability == 0.0,
                "Poor or changing fits must be missing evidence: {fit:?}"
            );
            let features = evidence.features_at(TICKS_PER_SECOND).unwrap();
            let bleed = features[1]
                .bleed
                .iter()
                .find(|feature| feature.source == project.sources[0].track_id)
                .unwrap();
            assert!(bleed.excess_residual_db.is_none());
        }
        assert!(
            evidence
                .features_at(TICKS_PER_SECOND)
                .unwrap()
                .iter()
                .flat_map(|frame| frame.normalized_values())
                .all(f32::is_finite)
        );
        eprintln!("development profile {profile}: {fit:?}");
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn apparently_perfect_sub_floor_transfer_does_not_become_identified_bleed() {
    let (root, project, audio) = fixture_profile(1.0, "below_floor");
    let evidence = AcousticEvidenceCache::default()
        .analyze(&project, &audio, None)
        .unwrap();
    let fit = evidence
        .transfer(project.sources[0].track_id, project.sources[1].track_id)
        .unwrap();
    assert!(matches!(
        fit.status,
        rust_app::TransferStatus::BelowRoomFloor
    ));
    assert!(
        fit.level_db.is_none(),
        "A mathematically fitted signal below room tone is not an identified coefficient"
    );
    assert!(
        !evidence
            .development_comparisons
            .iter()
            .find(|comparison| comparison.source == fit.source && comparison.target == fit.target)
            .unwrap()
            .fir_selected
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn acoustic_analysis_requires_explicit_synchronized_input_validation() {
    let (root, mut project, audio) = fixture();
    project.set_supplied_alignment(false).unwrap();
    assert!(
        AcousticEvidenceCache::default()
            .analyze(&project, &audio, None)
            .is_err(),
        "Missing timing confirmation must not fit a cross-microphone model"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn mixed_native_rate_roundoff_tail_is_unavailable_at_each_source_eof() {
    let (root, original, audio) = fixture();
    let samples = audio[1].read_frames(0..64_000).unwrap();
    let path = root.join("speaker-native-44100.wav");
    let mut format = bwavfile::WaveFmt::new_pcm_mono(44_100, 32);
    format.tag = 3;
    let mut writer = bwavfile::WaveWriter::create(&path, format)
        .unwrap()
        .audio_frame_writer()
        .unwrap();
    let resampled: Vec<_> = (0..176_401)
        .map(|frame| {
            let position = frame as f64 * 16_000.0 / 44_100.0;
            let left = position.floor() as usize;
            let a = samples[left.min(samples.len() - 1)];
            let b = samples[(left + 1).min(samples.len() - 1)];
            a + (b - a) * (position - position.floor()) as f32
        })
        .collect();
    writer.write_frames(&resampled).unwrap();
    writer.end().unwrap();
    let mut project = Project::create(
        root.join("mixed.cleanslate"),
        vec![original.sources[0].path.clone(), path],
    )
    .unwrap();
    project.set_supplied_alignment(true).unwrap();
    for label in original.reference_labels() {
        project
            .add_reference_label(label.range, label.active_speakers.clone(), &[])
            .unwrap();
    }
    let copies: Vec<_> = project
        .source_paths()
        .iter()
        .map(|path| {
            rust_app::prepare_analysis_audio_in(path, &root.join("mixed-cache"), None).unwrap()
        })
        .collect();
    assert!(
        copies[1].identity.frame_count > copies[0].identity.frame_count,
        "Fixture must retain the longer source's ceil analysis tail"
    );
    let evidence = AcousticEvidenceCache::default()
        .analyze(&project, &copies, None)
        .unwrap();
    assert!(evidence.features_at(4 * TICKS_PER_SECOND - 1).is_ok());
    assert!(
        evidence.features_at(4 * TICKS_PER_SECOND).is_err(),
        "The first source has ended; ceil-padded analysis cannot create extra ownership evidence"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn overlapping_solo_labels_cannot_duplicate_or_leak_held_out_samples() {
    let (root, mut project, audio) = fixture_profile(1.0, "colored");
    let mut cache = AcousticEvidenceCache::default();
    let before = cache.analyze(&project, &audio, None).unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 3 / 4, TICKS_PER_SECOND * 5 / 4).unwrap(),
            vec![project.sources[0].speaker_id],
            &[],
        )
        .unwrap();
    let after = cache.analyze(&project, &audio, None).unwrap();
    assert_ne!(before.cache_key, after.cache_key);
    for (a, b) in before
        .development_comparisons
        .iter()
        .zip(&after.development_comparisons)
    {
        assert_eq!(
            a.validation_samples, b.validation_samples,
            "Repeated physical reference audio cannot add held-out evidence"
        );
        assert_eq!(a.fir_explained, b.fir_explained);
    }
    assert_eq!(cache.stats().fixed_preparations, 1);
    std::fs::remove_dir_all(root).unwrap();
}
