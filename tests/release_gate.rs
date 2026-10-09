use rust_app::release_gate::{Label, LabelKind, evaluate};

#[path = "common/supervised_fixture.rs"]
mod fixture;

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
