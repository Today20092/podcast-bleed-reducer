use rust_app::release_gate::{Label, LabelKind, evaluate};

#[path = "common/supervised_fixture.rs"]
mod fixture;

#[test]
fn baseline_comparison_requires_matching_truth_and_automatic_bleed_improvement() {
    use rust_app::release_gate::{
        AppliedGainEvidence, EditorGainEvidence, FrozenBaselineComparison,
    };
    let automatic = AppliedGainEvidence {
        tracks: vec![],
        preservation_samples: 48,
        attenuated_preservation_samples: 0,
        overlap_ticks: 0,
        attenuated_overlap_ticks: 0,
        bleed_samples: 48,
        bleed_input_rms: 0.2,
        bleed_output_rms: 0.1,
    };
    let mut score = EditorGainEvidence {
        manifest_sha256: "controlled-manifest".into(),
        analysis_identity: "controlled".into(),
        effective: automatic.clone(),
        automatic,
        ownership: vec![],
    };
    let mut baseline = FrozenBaselineComparison {
        manifest_sha256: "controlled-manifest".into(),
        variant: "legacy_default".into(),
        receipt_sha256: "controlled".into(),
        raw_report_sha256: "controlled".into(),
        raw_report_path: "controlled.json".into(),
        grid: rust_app::SourceTimebase::new(48000, 48).unwrap(),
        preservation_samples: 48,
        attenuated_preservation_samples: 0,
        bleed_samples: 48,
        bleed_input_rms: 0.2,
        bleed_output_rms: 0.15,
    };
    assert_eq!(
        score.compare_baseline(&baseline).unwrap().release_status,
        "incomplete"
    );
    baseline.bleed_output_rms = 0.1;
    assert_eq!(
        score.compare_baseline(&baseline).unwrap().release_status,
        "failed"
    );
    baseline.bleed_output_rms = 0.15;
    score.automatic.attenuated_preservation_samples = 1;
    assert_eq!(
        score.compare_baseline(&baseline).unwrap().release_status,
        "failed",
        "effective manual unity cannot excuse automatic loss"
    );
    baseline.bleed_samples = 49;
    assert!(score.compare_baseline(&baseline).is_err());
    baseline.bleed_samples = 48;
    baseline.bleed_input_rms = 0.2;
    baseline.manifest_sha256 = "different-session-same-counts-and-energy".into();
    assert!(score.compare_baseline(&baseline).is_err());
    baseline.manifest_sha256 = score.manifest_sha256.clone();
    baseline.bleed_samples = 48;
    baseline.bleed_input_rms = 0.3;
    assert!(score.compare_baseline(&baseline).is_err());
}

#[test]
fn heldout_truth_cannot_share_native_sample_with_adjacent_development_reference() {
    let (root, mut project, _, _) = fixture::session_at_rates([48000, 48000]);
    let mut references = project.reference_labels().to_vec();
    let boundary = references[1].range.end + 1;
    references[1].range.end = boundary;
    project.replace_reference_labels(references).unwrap();
    let manifest = serde_json::json!({"version":1,"claim":"controlled","provenance":"adjacent native calibration boundary mechanics","prior_use":["controlled"],"reference_identity":rust_app::release_gate::FrozenEvaluationManifest::reference_identity(&project),"sources":project.sources.iter().map(|s|serde_json::json!({"track":s.track_id,"fingerprint":s.fingerprint,"grid":s.metadata.grid})).collect::<Vec<_>>(),"labels":[{"track":project.sources[0].track_id,"start":boundary,"end":boundary+10000,"kind":"preserve","case":"quiet_speech"}]});
    assert!(
        rust_app::release_gate::FrozenEvaluationManifest::read(
            &project,
            &manifest.to_string(),
            None
        )
        .is_err(),
        "tick-disjoint truth must not reuse an outward-rounded calibration sample"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires pinned frozen baseline executable closure"]
fn actual_frozen_comparison_scores_only_preselected_native_truth() {
    use sha2::{Digest, Sha256};
    let (root, project, _, _) = fixture::session_at_rates([48000, 48000]);
    let receipt = std::env::var("CLEANSLATE_FROZEN_RECEIPT").unwrap();
    let receipt_sha256 = std::env::var("CLEANSLATE_FROZEN_RECEIPT_SHA256").unwrap();
    let settings = serde_json::json!({"enabled":true,"examples":[{"kind":"SoloSpeech","track":0,"start":24000,"end":72000},{"kind":"SoloSpeech","track":1,"start":96000,"end":132000},{"kind":"RoomTone","track":0,"start":0,"end":12000}],"sources":project.sources.iter().map(|s|rust_app::SourceStamp::read(&s.path).unwrap()).collect::<Vec<_>>()});
    let bytes = serde_json::to_vec(&settings).unwrap();
    let settings_path = root.join("development.json");
    std::fs::write(&settings_path, &bytes).unwrap();
    let manifest = serde_json::json!({"version":1,"claim":"controlled","provenance":"actual frozen comparator mechanics","prior_use":["controlled"],"reference_identity":rust_app::release_gate::FrozenEvaluationManifest::reference_identity(&project),"sources":project.sources.iter().map(|s|serde_json::json!({"track":s.track_id,"fingerprint":s.fingerprint,"grid":s.metadata.grid})).collect::<Vec<_>>(),"labels":[{"track":project.sources[0].track_id,"start":3*rust_app::TICKS_PER_SECOND,"end":4*rust_app::TICKS_PER_SECOND,"kind":"preserve","case":"quiet_speech"},{"track":project.sources[1].track_id,"start":3*rust_app::TICKS_PER_SECOND,"end":4*rust_app::TICKS_PER_SECOND,"kind":"bleed","case":"changing_delay"}],"baseline":{"receipt":receipt,"receipt_sha256":receipt_sha256,"variant":"legacy_default","development_settings":settings_path,"development_settings_sha256":format!("{:x}",Sha256::digest(&bytes))}});
    let frozen = rust_app::release_gate::FrozenEvaluationManifest::read(
        &project,
        &manifest.to_string(),
        None,
    )
    .unwrap();
    let before = std::fs::read(project.path()).unwrap();
    let result = rust_app::release_gate::run_frozen_comparison(
        &project,
        &frozen,
        &root.join("comparison"),
        None,
    )
    .unwrap();
    assert_eq!(result.variant, "legacy_default");
    assert_eq!(result.manifest_sha256, frozen.digest());
    assert_eq!(result.preservation_samples, 48000);
    assert_eq!(result.bleed_samples, 48000);
    assert!(result.bleed_output_rms.is_finite());
    assert_eq!(std::fs::read(project.path()).unwrap(), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn frozen_csv_uses_unique_physical_native_intervals_without_fixed_rate_reinterpretation() {
    let (root, project, _, _) = fixture::session_at_rates([48000, 48000]);
    let label = serde_json::json!({"track":project.sources[0].track_id,"start":3*rust_app::TICKS_PER_SECOND,"end":4*rust_app::TICKS_PER_SECOND,"kind":"preserve","case":"quiet_speech"});
    let json = serde_json::json!({"version":1,"claim":"controlled","provenance":"CSV unit adapter mechanics","prior_use":["controlled"],"reference_identity":rust_app::release_gate::FrozenEvaluationManifest::reference_identity(&project),"sources":project.sources.iter().map(|s|serde_json::json!({"track":s.track_id,"fingerprint":s.fingerprint,"grid":s.metadata.grid})).collect::<Vec<_>>(),"labels":[label.clone(),label]});
    let manifest =
        rust_app::release_gate::FrozenEvaluationManifest::read(&project, &json.to_string(), None)
            .unwrap();
    let csv = rust_app::release_gate::frozen_baseline_csv(&project, &manifest, None).unwrap();
    assert_eq!(csv.lines().count(), 2);
    assert!(
        csv.lines()
            .nth(1)
            .unwrap()
            .contains(",3.00000000000000000,4.00000000000000000,direct-speech,")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires independently pinned external frozen baseline receipt and binaries"]
fn actual_frozen_baseline_checkout_and_artifacts_verify_without_modification() {
    let path = std::env::var_os("CLEANSLATE_FROZEN_RECEIPT").expect("CLEANSLATE_FROZEN_RECEIPT");
    let digest = std::env::var("CLEANSLATE_FROZEN_RECEIPT_SHA256")
        .expect("CLEANSLATE_FROZEN_RECEIPT_SHA256");
    let bytes = std::fs::read(path).unwrap();
    let receipt = rust_app::release_gate::verify_frozen_artifacts(&bytes, &digest, None).unwrap();
    receipt.verify_checkout(None).unwrap();
    assert!(receipt.executable("legacy_default").unwrap().is_file());
    assert!(receipt.executable("guided_joint").unwrap().is_file());
    assert!(receipt.executable("legacy_0_20").is_err());
}

#[test]
fn frozen_guidance_cannot_use_held_out_intervals_as_calibration() {
    let (root, mut project, _, _) = fixture::session_at_rates([48000, 48000]);
    let mut settings = serde_json::json!({"enabled":true,"examples":[{"kind":"SoloSpeech","track":0,"start":24000,"end":48000}],"sources":project.sources.iter().map(|s|rust_app::SourceStamp::read(&s.path).unwrap()).collect::<Vec<_>>()});
    rust_app::release_gate::validate_frozen_development_settings(
        &project,
        &settings.to_string(),
        None,
    )
    .unwrap();
    settings["examples"][0]["start"] = serde_json::json!(144000);
    settings["examples"][0]["end"] = serde_json::json!(192000);
    assert!(
        rust_app::release_gate::validate_frozen_development_settings(
            &project,
            &settings.to_string(),
            None
        )
        .is_err()
    );
    let mut labels = project.reference_labels().to_vec();
    for label in labels.iter_mut().filter(|l| l.active_speakers.is_empty()) {
        label.reference_track = Some(project.sources[1].track_id);
    }
    project.replace_reference_labels(labels).unwrap();
    settings["examples"] = serde_json::json!([{"kind":"RoomTone","track":0,"start":0,"end":12000}]);
    assert!(
        rust_app::release_gate::validate_frozen_development_settings(
            &project,
            &settings.to_string(),
            None
        )
        .is_err(),
        "track-scoped room truth cannot calibrate another microphone"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn independent_manifest_requires_frozen_sound_model_and_baseline_declarations() {
    let (root, project, _, _) = fixture::session();
    let manifest = serde_json::json!({"version":1,"claim":"independent","provenance":"independent contract validation","prior_use":[],"reference_identity":rust_app::release_gate::FrozenEvaluationManifest::reference_identity(&project),"sources":project.sources.iter().map(|s|serde_json::json!({"track":s.track_id,"fingerprint":s.fingerprint,"grid":s.metadata.grid})).collect::<Vec<_>>(),"labels":[{"track":project.sources[0].track_id,"start":3*rust_app::TICKS_PER_SECOND,"end":4*rust_app::TICKS_PER_SECOND,"kind":"preserve","case":"quiet_speech"}]});
    assert!(
        rust_app::release_gate::FrozenEvaluationManifest::read(
            &project,
            &manifest.to_string(),
            None
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn baseline_artifact_receipt_rejects_changed_executable_bytes() {
    use sha2::{Digest, Sha256};
    let (root, _, _, _) = fixture::session();
    let executable = root.join("ownership_evaluate.exe");
    std::fs::write(&executable, b"controlled executable bytes").unwrap();
    let receipt = serde_json::json!({"version":1,"commit":"6a60cc2bfe187ea889fd2d8bcc3344311cfd706b","checkout":root,"checkout_clean":true,"lock_sha256":"0".repeat(64),"silero_model_sha256":"1".repeat(64),"rustc":"controlled test","build_command":["cargo","build","--locked"],"artifacts":[{"name":"ownership_evaluate.exe","path":executable,"sha256":format!("{:x}",Sha256::digest(b"controlled executable bytes"))}],"allowed_variants":["legacy_default"],"comparison_requires_identical_48khz_sources":true,"quality_claim":"controlled artifact integrity only"});
    let bytes = serde_json::to_vec(&receipt).unwrap();
    let digest = format!("{:x}", Sha256::digest(&bytes));
    rust_app::release_gate::verify_frozen_artifacts(&bytes, &digest, None).unwrap();
    std::fs::write(&executable, b"altered executable bytes").unwrap();
    assert!(rust_app::release_gate::verify_frozen_artifacts(&bytes, &digest, None).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn accepted_evaluation_reuses_editor_analysis_without_installing_or_training_on_truth() {
    let (root, mut project, audio, neural) = fixture::session();
    let frozen = serde_json::json!({"version":1,"claim":"controlled","provenance":"scripted held-out mechanics","prior_use":["controlled"],"reference_identity":rust_app::release_gate::FrozenEvaluationManifest::reference_identity(&project),"sources":project.sources.iter().map(|s|serde_json::json!({"track":s.track_id,"fingerprint":s.fingerprint,"grid":s.metadata.grid})).collect::<Vec<_>>(),"labels":[{"track":project.sources[0].track_id,"start":3*rust_app::TICKS_PER_SECOND,"end":4*rust_app::TICKS_PER_SECOND,"kind":"preserve","case":"quiet_speech"}]});
    let manifest =
        rust_app::release_gate::FrozenEvaluationManifest::read(&project, &frozen.to_string(), None)
            .unwrap();
    let before = std::fs::read(project.path()).unwrap();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let analysis = worker
        .analyze_verified_evidence(
            &project,
            &audio,
            &neural,
            &std::sync::atomic::AtomicU8::new(0),
            &cancel,
        )
        .unwrap();
    let stats = worker.stats();
    let report =
        rust_app::release_gate::measure_editor_result(&project, &manifest, &analysis, &cancel)
            .unwrap();
    assert_eq!(report.manifest_sha256, manifest.digest());
    assert_eq!(report.effective.preservation_samples, 16000);
    assert_eq!(
        report.ownership.iter().map(|s| s.samples).sum::<u64>(),
        16000
    );
    assert_eq!(worker.stats().model_fits, stats.model_fits);
    assert_eq!(
        worker.stats().fixed_vad_preparations,
        stats.fixed_vad_preparations
    );
    assert_eq!(worker.stats().probability_passes, stats.probability_passes);
    assert_eq!(std::fs::read(project.path()).unwrap(), before);
    project
        .replace_manual_gain_edits(vec![rust_app::ManualGainEdit {
            id: 1,
            source: rust_app::SourceBinding {
                track: project.sources[0].track_id,
                fingerprint: project.sources[0].fingerprint.clone(),
            },
            interpolation: rust_app::ManualInterpolation::Linear,
            points: vec![
                rust_app::EnvelopePoint {
                    time: 3 * rust_app::TICKS_PER_SECOND,
                    gain: 0.5,
                },
                rust_app::EnvelopePoint {
                    time: 4 * rust_app::TICKS_PER_SECOND,
                    gain: 0.5,
                },
            ],
        }])
        .unwrap();
    let edited =
        rust_app::release_gate::measure_editor_result(&project, &manifest, &analysis, &cancel)
            .unwrap();
    assert_eq!(
        edited.automatic.attenuated_preservation_samples,
        report.automatic.attenuated_preservation_samples
    );
    assert_eq!(edited.effective.attenuated_preservation_samples, 16000);
    assert_eq!(worker.stats().model_fits, stats.model_fits);
    assert_eq!(worker.stats().probability_passes, stats.probability_passes);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn streaming_native_scores_keep_track_denominators_and_common_time_overlap() {
    let (root, _, _, _) = fixture::session();
    let paths: Vec<_> = [2, 4]
        .into_iter()
        .enumerate()
        .map(|(index, rate)| {
            let path = root.join(format!("native-{index}.wav"));
            let mut writer =
                bwavfile::WaveWriter::create(&path, bwavfile::WaveFmt::new_pcm_mono(rate, 16))
                    .unwrap()
                    .audio_frame_writer()
                    .unwrap();
            writer.write_frames(&vec![0.5; rate as usize]).unwrap();
            writer.end().unwrap();
            path
        })
        .collect();
    let mut project = rust_app::Project::create(root.join("mixed.cleanslate"), paths).unwrap();
    project.set_supplied_alignment(true).unwrap();
    let t = rust_app::TICKS_PER_SECOND;
    let labels = serde_json::json!([{ "track":project.sources[0].track_id,"start":0,"end":t,"kind":"preserve","case":"quiet_speech" },{ "track":project.sources[1].track_id,"start":t/4,"end":3*t/4,"kind":"preserve","case":"overlap" },{ "track":project.sources[1].track_id,"start":3*t/4,"end":t,"kind":"bleed","case":"changing_delay" }]);
    let json = serde_json::json!({"version":1,"claim":"controlled","provenance":"mixed native clock worked example","prior_use":["controlled"],"reference_identity":rust_app::release_gate::FrozenEvaluationManifest::reference_identity(&project),"sources":project.sources.iter().map(|s|serde_json::json!({"track":s.track_id,"fingerprint":s.fingerprint,"grid":s.metadata.grid})).collect::<Vec<_>>(),"labels":labels});
    let manifest =
        rust_app::release_gate::FrozenEvaluationManifest::read(&project, &json.to_string(), None)
            .unwrap();
    let measured = rust_app::release_gate::measure_applied_gain(
        &project,
        &manifest,
        &[
            rust_app::GainEnvelope::constant_at_rate(2, 2, 1.0).unwrap(),
            rust_app::GainEnvelope::constant_at_rate(4, 4, 0.5).unwrap(),
        ],
        None,
    )
    .unwrap();
    assert_eq!(measured.tracks[0].preservation_samples, 2);
    assert_eq!(measured.tracks[1].preservation_samples, 2);
    assert_eq!(measured.tracks[1].attenuated_preservation_samples, 2);
    assert_eq!(measured.overlap_ticks, t / 2);
    assert_eq!(measured.attenuated_overlap_ticks, t / 2);
    assert_eq!(measured.bleed_samples, 1);
    assert_eq!(measured.bleed_input_rms, 0.5);
    assert_eq!(measured.bleed_output_rms, 0.25);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn frozen_truth_rejects_contradictions_that_round_to_one_native_sample() {
    let (root, project, _, _) = fixture::session();
    let track = project.sources[0].track_id;
    let start = 3 * rust_app::TICKS_PER_SECOND;
    let manifest = serde_json::json!({"version":1,"claim":"controlled","provenance":"synthetic sub-sample geometry check","prior_use":["controlled"],"reference_identity":rust_app::release_gate::FrozenEvaluationManifest::reference_identity(&project),"sources":project.sources.iter().map(|source|serde_json::json!({"track":source.track_id,"fingerprint":source.fingerprint,"grid":source.metadata.grid})).collect::<Vec<_>>(),"labels":[{"track":track,"start":start,"end":start+1,"kind":"preserve","case":"quiet_speech"},{"track":track,"start":start+1,"end":start+2,"kind":"bleed","case":"changing_delay"}]});
    assert!(
        rust_app::release_gate::FrozenEvaluationManifest::read(
            &project,
            &manifest.to_string(),
            None
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn applied_gain_counts_attenuated_zero_samples_and_unique_truth() {
    let (root, _, _, _) = fixture::session();
    let source = root.join("zero.wav");
    let writer =
        bwavfile::WaveWriter::create(&source, bwavfile::WaveFmt::new_pcm_mono(4, 16)).unwrap();
    let mut frames = writer.audio_frame_writer().unwrap();
    frames.write_frames(&[0.0; 4]).unwrap();
    frames.end().unwrap();
    let mut project =
        rust_app::Project::create(root.join("zero.cleanslate"), vec![source]).unwrap();
    project.set_supplied_alignment(true).unwrap();
    let label = serde_json::json!({"track":project.sources[0].track_id,"start":0,"end":rust_app::TICKS_PER_SECOND,"kind":"preserve","case":"quiet_speech"});
    let json = serde_json::json!({"version":1,"claim":"controlled","provenance":"synthetic zero-valued wanted samples, mechanical gain check","prior_use":["controlled"],"reference_identity":rust_app::release_gate::FrozenEvaluationManifest::reference_identity(&project),"sources":[{"track":project.sources[0].track_id,"fingerprint":project.sources[0].fingerprint,"grid":project.sources[0].metadata.grid}],"labels":[label.clone(),label]});
    let manifest =
        rust_app::release_gate::FrozenEvaluationManifest::read(&project, &json.to_string(), None)
            .unwrap();
    let gain = rust_app::GainEnvelope::constant_at_rate(4, 4, 0.5).unwrap();
    let measured =
        rust_app::release_gate::measure_applied_gain(&project, &manifest, &[gain], None).unwrap();
    assert_eq!(measured.preservation_samples, 4);
    assert_eq!(measured.attenuated_preservation_samples, 4);
    assert_eq!(measured.overlap_ticks, 0);
    let preserved = rust_app::release_gate::measure_applied_gain(
        &project,
        &manifest,
        &[rust_app::GainEnvelope::constant_at_rate(4, 4, 1.0).unwrap()],
        None,
    )
    .unwrap();
    let report = rust_app::release_gate::EditorGainEvidence {
        manifest_sha256: "controlled-manifest".into(),
        analysis_identity: "controlled".into(),
        automatic: measured,
        effective: preserved,
        ownership: vec![],
    };
    assert_eq!(report.effective.attenuated_preservation_samples, 0);
    assert_eq!(
        report.release_status(),
        "failed",
        "manual preservation cannot hide automatic loss"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn frozen_evaluation_rejects_training_leakage_and_unions_physical_truth() {
    let (root, project, _, _) = fixture::session();
    let track = project.sources[0].track_id;
    let label = serde_json::json!({"track":track,"start":3*rust_app::TICKS_PER_SECOND,"end":4*rust_app::TICKS_PER_SECOND,"kind":"preserve","case":"quiet_speech"});
    let mut manifest = serde_json::json!({"version":1,"claim":"controlled","provenance":"synthetic mechanical fixture, not release truth","prior_use":["controlled development"],"reference_identity":rust_app::release_gate::FrozenEvaluationManifest::reference_identity(&project),"sources":project.sources.iter().map(|source|serde_json::json!({"track":source.track_id,"fingerprint":source.fingerprint,"grid":source.metadata.grid})).collect::<Vec<_>>(),"labels":[label.clone(),label]});
    let accepted = rust_app::release_gate::FrozenEvaluationManifest::read(
        &project,
        &manifest.to_string(),
        None,
    )
    .unwrap();
    assert_eq!(accepted.labels().len(), 1);
    manifest["labels"][1]["kind"] = "bleed".into();
    assert!(
        rust_app::release_gate::FrozenEvaluationManifest::read(
            &project,
            &manifest.to_string(),
            None
        )
        .is_err()
    );
    manifest["labels"] = serde_json::json!([{ "track":track,"start":rust_app::TICKS_PER_SECOND,"end":2*rust_app::TICKS_PER_SECOND,"kind":"preserve","case":"quiet_speech" }]);
    assert!(
        rust_app::release_gate::FrozenEvaluationManifest::read(
            &project,
            &manifest.to_string(),
            None
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn release_gate_measures_preservation_overlap_and_bleed_independently() {
    let inputs = vec![vec![1.0; 4], vec![0.5; 4]];
    let outputs = vec![vec![1.0, 1.0, 1.0, 0.25], vec![0.5, 0.5, 0.25, 0.125]];
    let labels = vec![
        Label {
            track: 0,
            start: 0,
            end: 3 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 1,
            start: rust_app::TICKS_PER_SECOND / 48_000,
            end: 2 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 1,
            start: 2 * rust_app::TICKS_PER_SECOND / 48_000,
            end: 4 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Bleed,
        },
    ];

    let evidence = evaluate(
        &inputs,
        &outputs,
        &labels,
        &[rust_app::SourceTimebase::new(48_000, 4).unwrap(); 2],
    )
    .unwrap();
    assert_eq!(evidence.preservation_samples, 4);
    assert_eq!(evidence.changed_preservation_samples, 0);
    assert_eq!(evidence.overlap_samples, 1);
    assert_eq!(evidence.changed_overlap_samples, 0);
    assert!(evidence.bleed_output_rms < evidence.bleed_input_rms);
}

#[test]
fn duplicate_labels_on_one_track_are_not_genuine_overlap() {
    let audio = vec![vec![1.0; 2], vec![1.0; 2]];
    let labels = vec![
        Label {
            track: 0,
            start: 0,
            end: 2 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 0,
            start: rust_app::TICKS_PER_SECOND / 48_000,
            end: 2 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Preserve,
        },
    ];
    assert_eq!(
        evaluate(
            &audio,
            &audio,
            &labels,
            &[rust_app::SourceTimebase::new(48_000, 2).unwrap(); 2]
        )
        .unwrap()
        .overlap_samples,
        0
    );
}

#[test]
fn mixed_native_rates_measure_each_tracks_samples_on_the_shared_physical_timeline() {
    use rust_app::{SourceTimebase, TICKS_PER_SECOND};
    let inputs = vec![vec![1.0; 2], vec![0.5; 4]];
    let outputs = vec![vec![1.0; 2], vec![0.5, 0.5, 0.25, 0.5]];
    let labels = vec![
        Label {
            track: 0,
            start: 0,
            end: TICKS_PER_SECOND,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 1,
            start: TICKS_PER_SECOND / 4,
            end: 3 * TICKS_PER_SECOND / 4,
            kind: LabelKind::Preserve,
        },
    ];
    let evidence = evaluate(
        &inputs,
        &outputs,
        &labels,
        &[
            SourceTimebase::new(2, 2).unwrap(),
            SourceTimebase::new(4, 4).unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(evidence.preservation_samples, 4);
    assert_eq!(evidence.changed_preservation_samples, 1);
    assert_eq!(evidence.overlap_samples, 2);
    assert_eq!(evidence.changed_overlap_samples, 1);
}
