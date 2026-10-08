use rust_app::model::{ProbabilityFrames, ProbabilityHistoryWriter};

#[test]
fn copied_global_windows_keep_early_boundary_and_final_frames_once() {
    let path = std::env::temp_dir().join(format!(
        "cleanslate-diarization-history-{}.f32",
        std::process::id()
    ));
    let mut writer = ProbabilityHistoryWriter::create(&path).unwrap();
    writer
        .append(&ProbabilityFrames {
            start: 0,
            frames: 2,
            speakers: 8,
            seconds_per_frame: 0.01,
            probabilities: [vec![0.1; 8], vec![0.2; 8]].concat(),
        })
        .unwrap();
    writer
        .append(&ProbabilityFrames {
            start: 1,
            frames: 2,
            speakers: 8,
            seconds_per_frame: 0.01,
            probabilities: [vec![0.2; 8], vec![0.3; 8]].concat(),
        })
        .unwrap();
    let history = writer.finish().unwrap();
    assert_eq!(history.frames, 3);
    assert_eq!(
        history.read_frames(0..3).unwrap(),
        [vec![0.1; 8], vec![0.2; 8], vec![0.3; 8]].concat()
    );
    assert_eq!(std::fs::metadata(&path).unwrap().len(), 96);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn cancelled_evidence_preparation_cannot_install_or_replace_results() {
    use rust_app::model::{RuntimeProvenance, prepare_diarization};
    let directory =
        std::env::temp_dir().join(format!("cleanslate-native-cancel-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let accepted = directory.join("accepted.json");
    std::fs::write(&accepted, b"accepted project result").unwrap();
    let runtime = RuntimeProvenance {
        revision: rust_app::model::RUNTIME_REVISION.into(),
        recipe_sha256: "0".repeat(64),
        binaries: Default::default(),
    };
    let error = prepare_diarization(
        &[],
        &runtime,
        std::path::Path::new("missing-helper"),
        std::path::Path::new("missing-runtime"),
        &directory,
        &std::sync::atomic::AtomicBool::new(true),
    )
    .unwrap_err();
    assert!(error.contains("cancelled"));
    assert_eq!(
        std::fs::read(&accepted).unwrap(),
        b"accepted project result"
    );
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn accepted_history_rejects_modified_payload_before_reuse() {
    let path = std::env::temp_dir().join(format!(
        "cleanslate-diarization-corrupt-{}.f32",
        std::process::id()
    ));
    let mut writer = ProbabilityHistoryWriter::create(&path).unwrap();
    writer
        .append(&ProbabilityFrames {
            start: 0,
            frames: 1,
            speakers: 8,
            seconds_per_frame: 0.01,
            probabilities: vec![0.1; 8],
        })
        .unwrap();
    let history = writer.finish().unwrap();
    std::fs::write(&path, [0u8; 32]).unwrap();
    assert!(history.verify().is_err());
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires pinned CPU native runtime and model; run explicitly in the serialized native lane"]
fn native_helper_retains_complete_tail_with_verified_provenance() {
    use rust_app::model::{DiarizationEvidence, DiarizationInput, DiarizationRequest};
    let directory = std::env::temp_dir().join(format!(
        "cleanslate-native-diarization-{}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).unwrap();
    let audio = rust_app::prepare_analysis_audio_in(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/import/mono48000.wav"),
        &directory.join("audio"),
        None,
    )
    .unwrap();
    let request = DiarizationRequest {
        inputs: vec![DiarizationInput::from(&audio)],
        output_directory: directory.join("result"),
        mix: Default::default(),
    };
    let path = directory.join("request.json");
    std::fs::write(&path, serde_json::to_vec(&request).unwrap()).unwrap();
    let runtime = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("native-runtime/bin");
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_model-check"))
        .arg("--diarize")
        .arg(&path)
        .arg("--runtime")
        .arg(runtime)
        .status()
        .unwrap();
    assert!(status.success());
    let evidence: DiarizationEvidence = serde_json::from_slice(
        &std::fs::read(request.output_directory.join("result.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(evidence.history.frames, 11);
    assert_eq!(evidence.sample_count, 1600);
    assert_eq!(evidence.sources, vec![audio.identity]);
    assert_eq!(evidence.preset, "v3-offline");
    evidence.history.verify().unwrap();
    assert_eq!(evidence.history.read_frames(0..11).unwrap().len(), 88);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "requires pinned CPU native runtime and model; run explicitly in the serialized native lane"]
fn completed_fixed_evidence_reuses_without_inference_and_cancellation_keeps_it() {
    use rust_app::model::{DiarizationInput, RuntimeProvenance, prepare_diarization};
    let directory = std::env::temp_dir().join(format!("cleanslate-diarization-reuse-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let audio = rust_app::prepare_analysis_audio_in(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import/mono48000.wav"),
        &directory.join("audio"), None).unwrap();
    let runtime = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("native-runtime/bin");
    let provenance: RuntimeProvenance = serde_json::from_slice(&std::fs::read(runtime.join("runtime-manifest.json")).unwrap()).unwrap();
    let inputs = vec![DiarizationInput::from(&audio)];
    let cache = directory.join("evidence");
    let accepted = prepare_diarization(&inputs, &provenance, std::path::Path::new(env!("CARGO_BIN_EXE_model-check")),
        &runtime, &cache, &std::sync::atomic::AtomicBool::new(false)).unwrap();
    let reused = prepare_diarization(&inputs, &provenance, std::path::Path::new("no-helper-no-inference"),
        &runtime, &cache, &std::sync::atomic::AtomicBool::new(false)).unwrap();
    assert_eq!(accepted.history.sha256, reused.history.sha256);
    assert_eq!(accepted.history.frames, 11);
    assert!(prepare_diarization(&inputs, &provenance, std::path::Path::new("missing-helper"), &runtime, &cache,
        &std::sync::atomic::AtomicBool::new(true)).unwrap_err().contains("cancelled"));
    accepted.history.verify().unwrap();
    assert_eq!(std::fs::read_dir(cache).unwrap().count(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}
