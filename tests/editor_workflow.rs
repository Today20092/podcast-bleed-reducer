use rust_app::{EditorAnalysisIdentity, Project, TICKS_PER_SECOND, TimeRange};
#[path = "common/supervised_fixture.rs"]
mod fixture;

#[test]
fn acoustic_analysis_trains_every_track_and_applies_beyond_reference_ranges() {
    let (root, mut project, audio, _) = fixture::session_gain(0.0);
    project.set_nemotron_enabled(false).unwrap();
    let mut parameters = project.parameters();
    parameters.close_ms = 50.0;
    project.set_parameters(parameters).unwrap();
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let progress = std::sync::atomic::AtomicU8::new(0);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let result = worker
        .analyze_acoustic_evidence(&project, &audio, &progress, &cancel)
        .unwrap();
    assert!(result.runtime.is_none());
    assert!(result.fixed_evidence.is_none());
    assert_eq!(
        result.ownership.models.as_ref().unwrap().models.len(),
        project.sources.len()
    );
    let probe = TICKS_PER_SECOND * 31 / 10;
    assert!(
        project
            .reference_labels()
            .iter()
            .all(|label| !(label.range.start <= probe && probe < label.range.end))
    );
    assert_eq!(
        result.temporal.states_at(probe).unwrap()[1],
        rust_app::TemporalTrackState::Absent
    );
    assert_eq!(
        result.temporal.states_at(probe).unwrap()[0],
        rust_app::TemporalTrackState::Owned
    );
    let stats = worker.stats();
    let repeated = worker
        .analyze_acoustic_evidence(&project, &audio, &progress, &cancel)
        .unwrap();
    assert_eq!(worker.stats(), stats);
    assert_eq!(repeated.ownership.cache_key, result.ownership.cache_key);
    let mut envelopes = rust_app::EffectiveEnvelopeCache::default();
    let effective = result
        .install(&mut project, &mut envelopes, None, None)
        .unwrap();
    assert_eq!(effective.envelopes[0].gain_at_time(probe), 1.0);
    assert!(effective.envelopes[1].gain_at_time(probe) < 0.5);
    let identity = rust_app::EditorAnalysisIdentity::capture(&project).unwrap();
    project.set_nemotron_enabled(true).unwrap();
    assert!(!identity.matches(&project));
    assert!(
        result
            .install(&mut project, &mut envelopes, None, None)
            .is_err()
    );
    project.save().unwrap();
    assert!(
        rust_app::Project::open(project.path())
            .unwrap()
            .nemotron_enabled()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn acoustic_analysis_preserves_explicit_quiet_overlap_and_rejects_canceled_install() {
    let (root, mut project, audio, _) = fixture::session_gain(0.03);
    project.set_nemotron_enabled(false).unwrap();
    let range = TimeRange::new(TICKS_PER_SECOND * 33 / 10, TICKS_PER_SECOND * 36 / 10).unwrap();
    project
        .add_reference_label(
            range,
            project.sources.iter().map(|s| s.speaker_id).collect(),
            &[],
        )
        .unwrap();
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let progress = std::sync::atomic::AtomicU8::new(0);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let result = worker
        .analyze_acoustic_evidence(&project, &audio, &progress, &cancel)
        .unwrap();
    let mut cache = rust_app::EffectiveEnvelopeCache::default();
    let effective = result
        .install(&mut project, &mut cache, None, None)
        .unwrap();
    for time in [range.start, TICKS_PER_SECOND * 7 / 2, range.end - 1] {
        assert!(
            effective
                .envelopes
                .iter()
                .all(|envelope| envelope.gain_at_time(time) == 1.0)
        );
    }
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(
        worker
            .analyze_acoustic_evidence(&project, &audio, &progress, &cancel)
            .is_err()
    );
    assert!(std::sync::Arc::ptr_eq(
        worker.last_completed().unwrap(),
        &result
    ));
    assert!(
        result
            .install(&mut project, &mut cache, None, Some(&cancel))
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn completed_job_wakes_long_cancellation_timer_without_claiming_a_cancel() {
    let (root, project, audio, evidence) = fixture::session();
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let progress = std::sync::atomic::AtomicU8::new(95);
    let (result, timing) = rust_app::measure_editor_cancellation(
        &progress,
        55,
        std::time::Duration::from_secs(600),
        |cancel| worker.analyze_verified_evidence(&project, &audio, &evidence, &progress, cancel),
    )
    .unwrap();
    assert!(result.is_ok());
    assert!(!timing.cancelled);
    assert!(!timing.job_active_at_request);
    assert!(timing.requested_progress.is_none());
    assert!(timing.return_after_request_ms.is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn timed_public_job_cancellation_retains_previous_completed_analysis() {
    let (root, mut project, audio, evidence) = fixture::session_at_rates([96000, 96000]);
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let progress = std::sync::atomic::AtomicU8::new(0);
    let previous = worker
        .analyze_verified_evidence(
            &project,
            &audio,
            &evidence,
            &progress,
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 33 / 10, TICKS_PER_SECOND * 36 / 10).unwrap(),
            project.sources.iter().map(|s| s.speaker_id).collect(),
            &[],
        )
        .unwrap();
    progress.store(0, std::sync::atomic::Ordering::Relaxed);
    let (result, timing) = rust_app::measure_editor_cancellation(
        &progress,
        55,
        std::time::Duration::from_millis(1),
        |cancel| worker.analyze_verified_evidence(&project, &audio, &evidence, &progress, cancel),
    )
    .unwrap();
    assert!(result.is_err());
    assert!(timing.cancelled);
    assert!(timing.requested_progress.is_some_and(|p| p >= 55));
    assert!(
        timing
            .return_after_request_ms
            .is_some_and(|ms| ms.is_finite() && ms >= 0.0)
    );
    assert!(std::sync::Arc::ptr_eq(
        worker.last_completed().unwrap(),
        &previous
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gain_benchmark_does_not_install_project_edits_or_reanalyze() {
    let (root, project, audio, evidence) = fixture::session();
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let result = worker
        .analyze_verified_evidence(
            &project,
            &audio,
            &evidence,
            &std::sync::atomic::AtomicU8::new(0),
            &cancel,
        )
        .unwrap();
    let before = std::fs::read(project.path()).unwrap();
    let stats = worker.stats();
    let report = rust_app::benchmark_editor_gain_edits(&project, &result, 4, &cancel).unwrap();
    assert_eq!(report.runs_ms.len(), 4);
    assert_eq!(report.automatic_builds, 1);
    assert!(report.runs_ms.iter().all(|ms| ms.is_finite() && *ms >= 0.0));
    assert_eq!(worker.stats().model_fits, stats.model_fits);
    assert_eq!(
        worker.stats().fixed_vad_preparations,
        stats.fixed_vad_preparations
    );
    assert_eq!(worker.stats().probability_passes, stats.probability_passes);
    assert_eq!(std::fs::read(project.path()).unwrap(), before);
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(rust_app::benchmark_editor_gain_edits(&project, &result, 4, &cancel).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn analysis_stage_measurements_describe_the_shared_completed_worker() {
    let (root, project, audio, evidence) = fixture::session();
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let result = worker
        .analyze_verified_evidence(
            &project,
            &audio,
            &evidence,
            &std::sync::atomic::AtomicU8::new(0),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
    let stages = serde_json::to_value(&result.timings).unwrap();
    for value in stages.as_object().unwrap().values() {
        assert!(value.as_f64().is_some_and(|ms| ms.is_finite() && ms >= 0.0));
    }
    assert!(result.timings.ownership_ms > 0.0);
    assert!(result.timings.temporal_ms > 0.0);
    assert!(result.timings.overview_ms > 0.0);
    assert_eq!(
        result.timings.native_history_ms, 0.0,
        "trusted prepared fixture does not perform native inference"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn supervised_review_correction_batch_analysis_and_gain_undo_save_share_one_pipeline() {
    let (root, mut project, audio, evidence) = fixture::session();
    let progress = std::sync::atomic::AtomicU8::new(0);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let first = worker
        .analyze_verified_evidence(&project, &audio, &evidence, &progress, &cancel)
        .unwrap();
    let mut cache = rust_app::EffectiveEnvelopeCache::default();
    let prior = first
        .install(&mut project, &mut cache, Some(&evidence.runtime), None)
        .unwrap();
    let mut review = rust_app::ReviewWorkspace::from_ownership(&first.temporal);
    assert!(!review.warnings.is_empty());
    review.set_comparison(rust_app::Comparison::Processed);
    review.select_warning(0);
    assert_eq!(review.playhead(), review.warnings[0].start);
    assert!(!review.highlighted_tracks().is_empty());
    assert_eq!(review.comparison(), rust_app::Comparison::Processed);
    project
        .set_review_state(
            review.selected_key().unwrap(),
            rust_app::ReviewState::Reviewed,
        )
        .unwrap();
    let before = worker.stats();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 3, TICKS_PER_SECOND * 13 / 4).unwrap(),
            project
                .sources
                .iter()
                .map(|source| source.speaker_id)
                .collect(),
            &[],
        )
        .unwrap();
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 7 / 2, TICKS_PER_SECOND * 15 / 4).unwrap(),
            project
                .sources
                .iter()
                .map(|source| source.speaker_id)
                .collect(),
            &[],
        )
        .unwrap();
    assert_eq!(
        worker.stats().model_fits,
        before.model_fits,
        "Label batches never automatically run inference"
    );
    assert!(matches!(
        project.result(),
        rust_app::ResultState::Stale { .. }
    ));
    assert!(std::sync::Arc::ptr_eq(
        &prior,
        cache.last_completed().unwrap()
    ));
    let next = worker
        .analyze_verified_evidence(&project, &audio, &evidence, &progress, &cancel)
        .unwrap();
    assert_eq!(
        worker.stats().fixed_vad_preparations,
        before.fixed_vad_preparations
    );
    let current = next
        .install(&mut project, &mut cache, Some(&evidence.runtime), None)
        .unwrap();
    let stats = worker.stats();
    let draft = project
        .region_gain_edit(
            project.sources[0].track_id,
            TimeRange::new(TICKS_PER_SECOND * 3, TICKS_PER_SECOND * 13 / 4).unwrap(),
            rust_app::GainRegionAction::Attenuate(0.5),
        )
        .unwrap();
    project.commit_gain_edit(draft).unwrap();
    let id = project.manual_gain_edits()[0].id;
    let original = project.gain_constraints();
    let mut drag =
        rust_app::GainPointDrag::begin(&project, project.sources[0].track_id, id, 0).unwrap();
    drag.move_to(TICKS_PER_SECOND * 3, 0.25).unwrap();
    project.commit_gain_edit(drag.finish()).unwrap();
    assert!(project.undo_gain_edit().unwrap());
    assert_eq!(project.gain_constraints(), original);
    let edited = cache
        .update(
            &project,
            &next.temporal,
            rust_app::project_fade_settings(&project).unwrap(),
            None,
        )
        .unwrap();
    let preview = next.preview(&project, &edited);
    for mode in [
        rust_app::AuditionMode::Original,
        rust_app::AuditionMode::Processed,
        rust_app::AuditionMode::Removed,
    ] {
        assert_eq!(
            rust_app::editor_audition_envelopes(&project, Some(&preview), mode).unwrap()[0]
                .gain_at_time(TICKS_PER_SECOND * 31 / 10),
            0.5
        );
    }
    project.save().unwrap();
    let loaded = Project::open(project.path()).unwrap();
    assert_eq!(loaded.gain_constraints(), original);
    assert_eq!(loaded.reference_labels(), project.reference_labels());
    assert_eq!(worker.stats().probability_passes, stats.probability_passes);
    assert!(current.analysis_current);
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(
        worker
            .analyze_verified_evidence(&project, &audio, &evidence, &progress, &cancel)
            .is_err()
    );
    assert!(std::sync::Arc::ptr_eq(
        worker.last_completed().unwrap(),
        &next
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "creates an editable local project from the owner-authorized synchronized files"]
fn prepare_owner_recording_without_inventing_reference_truth() {
    let root = std::path::Path::new(
        r"C:\Users\User\Documents\Codex\2026-08-14\podcast-automix-rust-gui\test-audio",
    );
    let destination = root.join("261005-supervised.cleanslate");
    assert!(!destination.exists(), "Keep any existing owner edits");
    let mut project = Project::create(
        &destination,
        (1..=3)
            .map(|i| root.join(format!("261005-channel-{i}.wav")))
            .collect(),
    )
    .unwrap();
    project.set_supplied_alignment(true).unwrap();
    assert!(project.reference_labels().is_empty());
    project.save().unwrap();
    eprintln!(
        "Prepared owner project: {}. Original playback is ready; solo and room references remain for the owner.",
        destination.display()
    );
}

#[test]
fn two_hour_overview_retains_brief_activity_and_bounds_visible_work() {
    let duration = TICKS_PER_SECOND * 7200;
    let frames = duration.div_ceil(rust_app::OWNERSHIP_FRAME_TICKS) as usize;
    let rows = (0..frames)
        .map(|i| rust_app::TemporalEvidence {
            probability: Some(if i == frames / 2 { 0.95 } else { 0.01 }),
            raw_speech: Some(if i == frames / 2 { 0.95 } else { 0.01 }),
            assigned_activity: Some(if i == frames / 2 { 0.95 } else { 0.01 }),
            preservation_reason: None,
        })
        .collect();
    let grid = rust_app::TemporalEvidenceGrid::new(
        vec![rust_app::TrackId(1)],
        duration,
        vec![rows],
        vec![0.01; frames],
    )
    .unwrap();
    let temporal = rust_app::TemporalOwnershipCache::default()
        .optimize_evidence(&grid, rust_app::TemporalSettings::default(), None)
        .unwrap();
    let overview = rust_app::OwnershipOverview::new(&temporal).unwrap();
    let viewport = rust_app::timeline::TimelineViewport::new(duration);
    let bands = overview.visible_bands(0, &viewport, 1000.0);
    assert!(bands.len() <= 1000);
    assert!(
        bands
            .iter()
            .any(|band| band.state == rust_app::OverviewState::MixedOrUncertain),
        "A brief utterance must remain visible inside an overview pixel"
    );
    let interaction = std::time::Instant::now();
    for rate in [44100, 48000, 96000, 47999] {
        let envelope =
            rust_app::GainEnvelope::constant_at_rate(7200 * rate as usize, rate, 0.25).unwrap();
        let curve = rust_app::visible_gain_curve(&viewport, 1000.0, &envelope);
        assert_eq!(curve.len(), 1001);
        assert_eq!(curve[0].time, 0);
        assert_eq!(curve[1000].time, duration);
        assert_eq!(curve[500].gain, 0.25);
        let mut navigated = viewport.clone();
        navigated.zoom_at(500.0, 1000.0, 5.0);
        navigated.pan_by_pixels(-100.0, 1000.0);
        let time = navigated.sample_at(500.0, 1000.0);
        let constraints = rust_app::GainConstraints {
            manual_edits: vec![rust_app::ManualGainEdit {
                id: 7,
                source: rust_app::SourceBinding {
                    track: rust_app::TrackId(1),
                    fingerprint: "test-only".into(),
                },
                points: vec![rust_app::EnvelopePoint { time, gain: 0.5 }],
                interpolation: rust_app::ManualInterpolation::Linear,
            }],
            protections: vec![],
        };
        assert_eq!(
            rust_app::hit_gain_point(
                &constraints,
                rust_app::TrackId(1),
                &navigated,
                rust_app::GainPointer {
                    x: navigated.x_for_sample(time, 1000.0),
                    y: 32.0,
                    width: 1000.0,
                    height: 64.0
                }
            ),
            Some((7, 0))
        );
        for _ in 0..25 {
            assert!(rust_app::visible_gain_buckets(&navigated, 1000.0, &envelope).len() <= 1000);
            assert!(overview.visible_bands(0, &navigated, 1000.0).len() <= 1000);
        }
    }
    eprintln!(
        "Two-hour mixed-rate visible interaction kernels (100 gain/overview queries plus navigation/hit checks): {:?}",
        interaction.elapsed()
    );
}

#[test]
fn offline_warm_preflight_reports_missing_runtime_without_inference_or_download() {
    let missing =
        std::env::temp_dir().join(format!("cleanslate-missing-runtime-{}", std::process::id()));
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_model-check"))
        .arg("--verify-only")
        .arg("--runtime")
        .arg(missing)
        .output()
        .unwrap();
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(
        !error.contains("Usage:"),
        "Verification-only must be a supported offline operation: {error}"
    );
    assert!(error.contains("runtime"));
    assert!(!String::from_utf8_lossy(&result.stdout).contains("Offline CPU inference passed"));
}

#[test]
fn supervised_completion_composes_latest_edits_and_rejects_obsolete_or_failed_installation() {
    let (root, mut project, audio, evidence) = fixture::session();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let progress = std::sync::atomic::AtomicU8::new(0);
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let completed = worker
        .analyze_verified_evidence(&project, &audio, &evidence, &progress, &cancel)
        .unwrap();
    let before = worker.stats();
    let mut cache = rust_app::EffectiveEnvelopeCache::default();
    let draft = project
        .region_gain_edit(
            project.sources[0].track_id,
            TimeRange::new(TICKS_PER_SECOND * 3, TICKS_PER_SECOND * 13 / 4).unwrap(),
            rust_app::GainRegionAction::Attenuate(0.4),
        )
        .unwrap();
    project.commit_gain_edit(draft).unwrap();
    let accepted = completed
        .install(&mut project, &mut cache, Some(&evidence.runtime), None)
        .unwrap();
    assert_eq!(
        accepted.envelopes[0].gain_at_time(TICKS_PER_SECOND * 31 / 10),
        0.4
    );
    assert!(matches!(
        project.result(),
        rust_app::ResultState::Current { .. }
    ));
    assert!(
        project
            .analysis_recipe()
            .unwrap()
            .feature_recipe
            .starts_with(rust_app::EDITOR_ANALYSIS_RECIPE)
    );
    assert_eq!(worker.stats().probability_passes, before.probability_passes);
    assert_eq!(worker.stats().model_fits, before.model_fits);
    let narrow = project
        .region_gain_edit(
            project.sources[0].track_id,
            TimeRange::new(
                TICKS_PER_SECOND * 1001 / 1000,
                TICKS_PER_SECOND * 1002 / 1000,
            )
            .unwrap(),
            rust_app::GainRegionAction::Attenuate(0.2),
        )
        .unwrap();
    project.commit_gain_edit(narrow).unwrap();
    let narrow_effective = cache
        .update(
            &project,
            &completed.temporal,
            rust_app::project_fade_settings(&project).unwrap(),
            None,
        )
        .unwrap();
    let viewport = rust_app::timeline::TimelineViewport::new(TICKS_PER_SECOND * 4);
    let buckets = rust_app::visible_gain_buckets(&viewport, 1000.0, &narrow_effective.envelopes[0]);
    assert!(
        buckets
            .iter()
            .any(|bucket| bucket.lower == 0.2 && bucket.upper == 1.0),
        "A sub-pixel manual attenuation must remain visible through shared bounds"
    );
    let accepted = narrow_effective;
    let mut changed_runtime = evidence.runtime.clone();
    changed_runtime.recipe_sha256 = "2".repeat(64);
    assert!(
        completed
            .install(&mut project, &mut cache, Some(&changed_runtime), None)
            .is_err()
    );
    assert!(std::sync::Arc::ptr_eq(
        cache.last_completed().unwrap(),
        &accepted
    ));
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let locked = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(project.path())
            .unwrap();
        assert!(
            completed
                .install(&mut project, &mut cache, Some(&evidence.runtime), None)
                .is_err()
        );
        assert!(std::sync::Arc::ptr_eq(
            cache.last_completed().unwrap(),
            &accepted
        ));
        drop(locked);
    }
    project
        .add_reference_label(
            TimeRange::new(TICKS_PER_SECOND * 7 / 2, TICKS_PER_SECOND * 15 / 4).unwrap(),
            project.sources.iter().map(|s| s.speaker_id).collect(),
            &[],
        )
        .unwrap();
    assert!(
        completed
            .install(&mut project, &mut cache, Some(&evidence.runtime), None)
            .is_err()
    );
    assert!(std::sync::Arc::ptr_eq(
        cache.last_completed().unwrap(),
        &accepted
    ));
    assert!(matches!(
        project.result(),
        rust_app::ResultState::Stale { .. }
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_gain_point_drag_is_transient_bounded_and_one_undo_command() {
    let (root, mut project, _, _) = fixture::session();
    let track = project.sources[0].track_id;
    let draft = project
        .region_gain_edit(
            track,
            TimeRange::new(TICKS_PER_SECOND, TICKS_PER_SECOND * 2).unwrap(),
            rust_app::GainRegionAction::Attenuate(0.4),
        )
        .unwrap();
    project.commit_gain_edit(draft).unwrap();
    let before = project.gain_constraints();
    let id = project.manual_gain_edits()[0].id;
    let mut drag = rust_app::GainPointDrag::begin(&project, track, id, 0).unwrap();
    drag.move_to(TICKS_PER_SECOND / 2, 0.75).unwrap();
    drag.move_to(TICKS_PER_SECOND / 4, 0.8).unwrap();
    assert_eq!(project.gain_constraints(), before);
    assert!(drag.move_to(TICKS_PER_SECOND, f32::NAN).is_err());
    project.commit_gain_edit(drag.finish()).unwrap();
    assert_eq!(project.manual_gain_edits()[0].points[0].gain, 0.8);
    assert_eq!(
        project.manual_gain_edits()[0].points[0].time,
        TICKS_PER_SECOND / 4
    );
    assert!(project.undo_gain_edit().unwrap());
    assert_eq!(project.gain_constraints(), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn analysis_commit_identity_ignores_gain_but_rejects_new_truth_and_alignment() {
    let directory =
        std::env::temp_dir().join(format!("cleanslate-editor-identity-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("source.wav");
    let data_bytes = 48000_u32 * 2;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&48000_u32.to_le_bytes());
    bytes.extend_from_slice(&96000_u32.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    bytes.resize(bytes.len() + data_bytes as usize, 0);
    std::fs::write(&source, bytes).unwrap();
    let mut project = Project::create(directory.join("session.cleanslate"), vec![source]).unwrap();
    assert!(project.require_alignment().is_err());
    let early_original =
        rust_app::editor_audition_envelopes(&project, None, rust_app::AuditionMode::Original)
            .expect("Original playback needs neither timing confirmation nor references/model");
    assert_eq!(early_original[0].gain_at(47999), 1.0);
    assert!(
        project.require_alignment().is_err(),
        "Audition must not declare synchronization"
    );
    project.set_supplied_alignment(true).unwrap();
    let original = rust_app::original_envelopes(&project).unwrap();
    assert_eq!(original[0].sample_rate(), 48000);
    assert_eq!(original[0].sample_count(), 48000);
    assert_eq!(original[0].gain_at(47999), 1.0);
    let preview = rust_app::AnalysisPreview {
        stems: project.source_paths(),
        regions: vec![],
        envelopes: vec![rust_app::GainEnvelope::constant_at_rate(48000, 48000, 0.25).unwrap()],
        protections: vec![],
        parameters: project.parameters(),
    };
    for mode in [
        rust_app::AuditionMode::Original,
        rust_app::AuditionMode::Processed,
        rust_app::AuditionMode::Removed,
    ] {
        let behind_mode =
            rust_app::editor_audition_envelopes(&project, Some(&preview), mode).unwrap();
        assert_eq!(
            behind_mode[0].gain_at(24000),
            0.25,
            "Comparison mode must retain the same effective curve"
        );
    }
    assert!(
        rust_app::editor_audition_envelopes(&project, None, rust_app::AuditionMode::Processed)
            .is_err()
    );
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let progress = std::sync::atomic::AtomicU8::new(0);
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let error = worker
        .analyze(&project, &directory.join("cache"), None, &progress, &cancel)
        .unwrap_err();
    assert!(error.to_string().contains("reference"));
    assert!(worker.last_completed().is_none());
    let snapshot = EditorAnalysisIdentity::capture(&project).unwrap();
    assert!(snapshot.matches(&project));
    let mut parameters = project.parameters();
    parameters.attenuation_db = -12.0;
    project.set_parameters(parameters).unwrap();
    assert!(
        snapshot.matches(&project),
        "Gain changes compose with completed analysis"
    );
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    assert!(
        !snapshot.matches(&project),
        "A delayed result cannot overwrite new human truth"
    );
    let snapshot = EditorAnalysisIdentity::capture(&project).unwrap();
    project.set_supplied_alignment(false).unwrap();
    assert!(
        !snapshot.matches(&project),
        "Withdrawing alignment blocks installation"
    );
    std::fs::remove_dir_all(directory).unwrap();
}
