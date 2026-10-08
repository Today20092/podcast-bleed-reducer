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
fn ninth_input_is_rejected_before_loading_native_runtime_or_creating_output() {
    use rust_app::model::{DiarizationInput, DiarizationRequest, diarize_in_helper};
    let directory =
        std::env::temp_dir().join(format!("cleanslate-native-capacity-{}", std::process::id()));
    let audio = rust_app::prepare_analysis_audio_in(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/import/mono48000.wav"),
        &directory.join("audio"),
        None,
    )
    .unwrap();
    let request = DiarizationRequest {
        inputs: vec![DiarizationInput::from(&audio); 9],
        output_directory: directory.join("result"),
        mix: Default::default(),
    };
    assert!(
        diarize_in_helper(&request, std::path::Path::new("missing-runtime"))
            .unwrap_err()
            .contains("one to eight")
    );
    assert!(!request.output_directory.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn skipped_compacted_prefix_is_rejected_without_damaging_retained_history() {
    let path =
        std::env::temp_dir().join(format!("cleanslate-native-gap-{}.f32", std::process::id()));
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
    assert!(
        writer
            .append(&ProbabilityFrames {
                start: 2,
                frames: 1,
                speakers: 8,
                seconds_per_frame: 0.01,
                probabilities: vec![0.2; 8]
            })
            .is_err()
    );
    let history = writer.finish().unwrap();
    assert_eq!(history.frames, 1);
    assert_eq!(history.read_frames(0..1).unwrap(), vec![0.1; 8]);
    history.verify().unwrap();
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
    let directory = std::env::temp_dir().join(format!(
        "cleanslate-diarization-reuse-{}",
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
    let runtime = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("native-runtime/bin");
    let provenance: RuntimeProvenance =
        serde_json::from_slice(&std::fs::read(runtime.join("runtime-manifest.json")).unwrap())
            .unwrap();
    let inputs = vec![DiarizationInput::from(&audio)];
    let cache = directory.join("evidence");
    let accepted = prepare_diarization(
        &inputs,
        &provenance,
        std::path::Path::new(env!("CARGO_BIN_EXE_model-check")),
        &runtime,
        &cache,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    let reused = prepare_diarization(
        &inputs,
        &provenance,
        std::path::Path::new("no-helper-no-inference"),
        &runtime,
        &cache,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(accepted.history.sha256, reused.history.sha256);
    assert_eq!(accepted.history.frames, 11);
    let prepared_bytes = std::fs::read(audio.path()).unwrap();
    std::fs::write(audio.path(), vec![0; prepared_bytes.len()]).unwrap();
    assert!(
        prepare_diarization(
            &inputs,
            &provenance,
            std::path::Path::new("missing-helper"),
            &runtime,
            &cache,
            &std::sync::atomic::AtomicBool::new(false)
        )
        .unwrap_err()
        .contains("prepared analysis")
    );
    std::fs::write(audio.path(), prepared_bytes).unwrap();
    assert!(
        prepare_diarization(
            &inputs,
            &provenance,
            std::path::Path::new("missing-helper"),
            &runtime,
            &cache,
            &std::sync::atomic::AtomicBool::new(true)
        )
        .unwrap_err()
        .contains("cancelled")
    );
    accepted.history.verify().unwrap();
    assert_eq!(
        std::fs::read_dir(&cache)
            .unwrap()
            .filter(|e| e.as_ref().unwrap().file_type().unwrap().is_dir())
            .count(),
        1
    );
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let locked = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(accepted.history.path())
            .unwrap();
        assert!(
            prepare_diarization(
                &inputs,
                &provenance,
                std::path::Path::new("missing-helper"),
                &runtime,
                &cache,
                &std::sync::atomic::AtomicBool::new(false)
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read_dir(&cache)
                .unwrap()
                .filter(|e| e.as_ref().unwrap().file_type().unwrap().is_dir())
                .count(),
            1
        );
        drop(locked);
        accepted.history.verify().unwrap();
    }
    std::fs::remove_file(accepted.history.path()).unwrap();
    let rebuilt = prepare_diarization(
        &inputs,
        &provenance,
        std::path::Path::new(env!("CARGO_BIN_EXE_model-check")),
        &runtime,
        &cache,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(rebuilt.history.sha256, accepted.history.sha256);
    rebuilt.history.verify().unwrap();
    assert!(std::fs::read_dir(&cache).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".invalid-")
    }));
    let invalid_bytes = vec![0; std::fs::metadata(rebuilt.history.path()).unwrap().len() as usize];
    std::fs::write(rebuilt.history.path(), &invalid_bytes).unwrap();
    let repaired = prepare_diarization(
        &inputs,
        &provenance,
        std::path::Path::new(env!("CARGO_BIN_EXE_model-check")),
        &runtime,
        &cache,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(repaired.history.sha256, accepted.history.sha256);
    repaired.history.verify().unwrap();
    let key = repaired
        .history
        .path()
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy();
    let lock_path = cache.join(format!(".{key}.lock"));
    let marker = directory.join("lock-ready");
    std::fs::remove_file(repaired.history.path()).unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "cache_lock_owner_fixture",
            "--nocapture",
        ])
        .env("CLEANSLATE_CACHE_LOCK", &lock_path)
        .env("CLEANSLATE_CACHE_LOCK_READY", &marker)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !marker.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let ready = marker.exists();
    let busy = prepare_diarization(
        &inputs,
        &provenance,
        std::path::Path::new("missing-helper"),
        &runtime,
        &cache,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .is_err();
    let _ = child.kill();
    child.wait().unwrap();
    assert!(ready && busy);
    let restarted = prepare_diarization(
        &inputs,
        &provenance,
        std::path::Path::new(env!("CARGO_BIN_EXE_model-check")),
        &runtime,
        &cache,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    restarted.history.verify().unwrap();
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "owned subprocess fixture for cache writer crash recovery"]
fn cache_lock_owner_fixture() {
    let Some(path) = std::env::var_os("CLEANSLATE_CACHE_LOCK") else {
        return;
    };
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .unwrap();
    file.try_lock().unwrap();
    std::fs::write(
        std::env::var_os("CLEANSLATE_CACHE_LOCK_READY").unwrap(),
        b"locked",
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_secs(20));
}

#[test]
#[ignore = "requires scripted development fixture and pinned CPU runtime; serialized native lane"]
fn continuous_stream_repeat_and_push_boundaries_preserve_probabilities() {
    use rust_app::model::{ModelCache, NativeRuntime};
    let fixture = std::path::PathBuf::from(
        std::env::var_os("CLEANSLATE_DIARIZATION_FIXTURE").expect("fixture directory"),
    );
    let decode = |name: &str| {
        std::fs::read(fixture.join(name))
            .unwrap()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect::<Vec<_>>()
    };
    let first = decode("track-1.f32");
    let second = decode("track-2.f32");
    assert_eq!(first.len(), 120 * 16_000);
    assert_eq!(first.len(), second.len());
    let mixed: Vec<_> = first
        .iter()
        .zip(&second)
        .map(|(a, b)| (a + b) * 0.5)
        .collect();
    let runtime = NativeRuntime::open(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("native-runtime/bin"),
    )
    .unwrap();
    let model = runtime
        .load_model(&ModelCache::default_location().unwrap())
        .unwrap();
    let infer = |chunk| {
        let mut stream = model.stream().unwrap();
        for samples in mixed.chunks(chunk) {
            stream.push(samples).unwrap();
        }
        stream.finish().unwrap();
        let probabilities = stream.probabilities().unwrap();
        assert_eq!(probabilities.start, 0);
        assert_eq!(probabilities.frames, 12_001);
        assert_eq!(probabilities.speakers, 8);
        probabilities.probabilities
    };
    let reference = infer(16_000);
    assert_eq!(reference, infer(16_000), "fresh continuous stream repeat");
    assert_eq!(
        reference,
        infer(512),
        "push boundaries must not restart speaker identity"
    );
}
