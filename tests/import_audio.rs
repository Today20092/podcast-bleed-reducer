use rust_app::SourceMetadata;
use std::path::Path;

#[test]
fn mono_aiff_retains_its_native_rate_and_exact_decoded_frame_count() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import/mono44100.aiff");
    let metadata = SourceMetadata::read(&source).unwrap();
    assert_eq!(metadata.channels, 1);
    assert_eq!(metadata.grid.sample_rate, 44_100);
    assert_eq!(metadata.grid.frame_count, 4_410);
}

#[test]
fn advertised_encoded_profiles_retain_native_decoded_lengths() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    for (name, expected_frames) in [
        ("mono44100.flac", 4_410),
        ("pcm8.aiff", 4_410),
        ("pcm24.aiff", 4_410),
        ("pcm32.aiff", 4_410),
        ("mono44100.mp3", 4_410),
        // ffprobe confirms six AAC-LC packets, each decoding 1024 frames. Its edit-list
        // timing is not a gapless guarantee; every decoded frame remains in the source.
        ("mono44100-aac.m4a", 6_144),
        ("mono44100-aac.mp4", 6_144),
        ("mono44100-alac.m4a", 4_410),
        ("mono44100-alac.mp4", 4_410),
        ("mono44100.ogg", 4_410),
    ] {
        let metadata = SourceMetadata::read(&directory.join(name))
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(metadata.channels, 1, "{name}");
        assert_eq!(metadata.grid.sample_rate, 44_100, "{name}");
        assert_eq!(metadata.grid.frame_count, expected_frames, "{name}");
    }
}

#[test]
fn advertised_wav_pcm_and_float_profiles_decode_without_changing_the_sources() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    for name in [
        "pcm16.wav",
        "pcm24.wav",
        "pcm32.wav",
        "mono48000.wav",
        "float64.wav",
    ] {
        let path = directory.join(name);
        let bytes = std::fs::read(&path).unwrap();
        let audio = rust_app::prepare_analysis_audio(&path, None).unwrap();
        assert_eq!(audio.identity.source.grid.frame_count, 4_800, "{name}");
        assert_eq!(audio.identity.frame_count, 1_600, "{name}");
        assert_eq!(std::fs::read(path).unwrap(), bytes, "{name}");
    }
}

#[test]
fn synchronized_mixed_native_rates_validate_in_physical_time_without_changing_sources() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    let paths = [
        directory.join("mono44100.aiff"),
        directory.join("mono48000.wav"),
        directory.join("mono96000.wav"),
    ];
    let validation = rust_app::validate(&paths).unwrap();
    assert!(
        validation
            .warnings
            .iter()
            .any(|warning| warning.contains("alignment"))
    );
    assert_eq!(
        SourceMetadata::read(&paths[0]).unwrap().grid.frame_count,
        4_410
    );
    assert_eq!(
        SourceMetadata::read(&paths[1]).unwrap().grid.frame_count,
        4_800
    );
    assert_eq!(
        SourceMetadata::read(&paths[2]).unwrap().grid.frame_count,
        9_600
    );
}

#[test]
fn analysis_copies_are_antialiased_and_delay_compensated_on_the_common_grid() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    for rate in [44_100, 48_000, 96_000] {
        let audio =
            rust_app::prepare_analysis_audio(&directory.join(format!("impulses{rate}.wav")), None)
                .unwrap();
        assert_eq!(audio.identity.source.grid.sample_rate, rate);
        let samples = audio.read_frames(0..1_600).unwrap();
        assert_eq!(samples.len(), 1_600);
        for expected in [160, 1_440] {
            let peak = (expected - 8..expected + 9)
                .max_by(|&left, &right| samples[left].abs().total_cmp(&samples[right].abs()))
                .unwrap();
            assert_eq!(peak, expected, "rate {rate}, endpoint {expected}");
        }
    }
}

#[test]
fn disk_backed_analysis_reuses_a_complete_artifact_and_cancellation_keeps_it_intact() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import/impulses48000.wav");
    let directory =
        std::env::temp_dir().join(format!("cleanslate-analysis-cache-{}", std::process::id()));
    let audio = rust_app::prepare_analysis_audio_in(&source, &directory, None).unwrap();
    assert_eq!(audio.read_frames(0..1_600).unwrap().len(), 1_600);
    let reused = rust_app::prepare_analysis_audio_in(&source, &directory, None).unwrap();
    assert_eq!(reused.identity, audio.identity);
    assert_eq!(reused.path(), audio.path());
    let cancelled = std::sync::atomic::AtomicBool::new(true);
    assert!(rust_app::prepare_analysis_audio_in(&source, &directory, Some(&cancelled)).is_err());
    assert_eq!(
        audio.read_frames(160..161).unwrap(),
        reused.read_frames(160..161).unwrap()
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn missing_timing_requires_a_saved_supplied_alignment_declaration_before_analysis() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import/mono48000.wav");
    let directory =
        std::env::temp_dir().join(format!("cleanslate-alignment-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let mut project =
        rust_app::Project::create(directory.join("session.cleanslate"), vec![source]).unwrap();
    assert!(!project.supplied_alignment());
    assert!(project.require_alignment().is_err());
    project.set_supplied_alignment(true).unwrap();
    assert!(project.require_alignment().is_ok());
    assert!(
        rust_app::Project::open(project.path())
            .unwrap()
            .supplied_alignment()
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn mixed_rate_preview_uses_prepared_evidence_on_the_same_physical_timeline() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    let paths = vec![
        directory.join("mono48000.wav"),
        directory.join("mono96000.wav"),
    ];
    let mut cache = rust_app::PreviewCache::default();
    cache.set_supplied_alignment(true);
    let preview = cache
        .analyze(
            &paths,
            &[],
            rust_app::AutomixParameters::default(),
            &std::sync::atomic::AtomicU8::new(0),
            None,
        )
        .unwrap();
    assert_eq!(preview.regions.last().unwrap().end, 2_822_400);
    cache.set_supplied_alignment(false);
    assert!(
        cache
            .analyze(
                &paths,
                &[],
                rust_app::AutomixParameters::default(),
                &std::sync::atomic::AtomicU8::new(0),
                None
            )
            .is_err()
    );
}

#[test]
fn truncated_advertised_inputs_never_become_shorter_valid_sources() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    let temporary =
        std::env::temp_dir().join(format!("cleanslate-truncated-{}", std::process::id()));
    std::fs::create_dir_all(&temporary).unwrap();
    for filename in [
        "mono48000.wav",
        "mono44100.aiff",
        "mono44100.flac",
        "mono44100.mp3",
        "mono44100-aac.m4a",
        "mono44100-alac.m4a",
        "mono44100.ogg",
    ] {
        let bytes = std::fs::read(directory.join(filename)).unwrap();
        let path = temporary.join(filename);
        std::fs::write(&path, &bytes[..bytes.len() - 17]).unwrap();
        assert!(
            rust_app::validate(&[path]).is_err(),
            "accepted truncated {filename}"
        );
    }
    std::fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn analysis_resampling_rejects_energy_above_the_16khz_nyquist_limit() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    for rate in [48_000, 96_000] {
        let audio =
            rust_app::prepare_analysis_audio(&directory.join(format!("outofband{rate}.wav")), None)
                .unwrap();
        let samples = audio.read_frames(4_000..12_000).unwrap();
        let rms =
            (samples.iter().map(|value| value * value).sum::<f32>() / samples.len() as f32).sqrt();
        assert!(
            rms < 0.0001,
            "aliased 12 kHz RMS {rms} at source rate {rate}"
        );
    }
}

#[test]
fn unsupported_profiles_and_multichannel_inputs_are_rejected() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    for filename in [
        "unsupported-opus.ogg",
        "unsupported-stereo.wav",
        "unsupported-pcm8.wav",
    ] {
        assert!(
            rust_app::validate(&[directory.join(filename)]).is_err(),
            "accepted {filename}"
        );
    }
}

#[test]
fn float64_import_renders_explicit_native_rate_float32_output_without_panicking() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import/float64.wav");
    let directory =
        std::env::temp_dir().join(format!("cleanslate-f64-render-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let result = rust_app::run(rust_app::RunRequest {
        supplied_alignment: true,
        stems: vec![source],
        output_dir: directory.clone(),
        regions: vec![rust_app::Region {
            start: 0,
            end: 2_822_400,
            decision: rust_app::Decision::Owned(vec![0]),
        }],
        protections: vec![],
        cancelled: None,
    })
    .unwrap();
    let mut reader = bwavfile::WaveReader::open(&result.stems[0]).unwrap();
    let format = reader.format().unwrap();
    assert_eq!(format.sample_rate, 48_000);
    assert_eq!(format.bits_per_sample, 32);
    assert_eq!(format.tag, 3);
    assert_eq!(reader.frame_length().unwrap(), 4_800);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn multiple_mono_audio_streams_are_rejected_instead_of_selecting_one() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/import/unsupported-multiple-audio.mp4");
    assert!(rust_app::validate(&[source]).is_err());
}

#[cfg(windows)]
#[test]
fn ordinary_cache_io_failure_preserves_the_accepted_artifact() {
    use std::os::windows::fs::OpenOptionsExt;
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import/mono48000.wav");
    let directory =
        std::env::temp_dir().join(format!("cleanslate-cache-io-{}", std::process::id()));
    let audio = rust_app::prepare_analysis_audio_in(&source, &directory, None).unwrap();
    let accepted = std::fs::read(audio.path()).unwrap();
    let exclusive = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(audio.path())
        .unwrap();
    assert!(rust_app::prepare_analysis_audio_in(&source, &directory, None).is_err());
    assert!(audio.path().exists());
    drop(exclusive);
    assert_eq!(std::fs::read(audio.path()).unwrap(), accepted);
    assert_eq!(
        rust_app::prepare_analysis_audio_in(&source, &directory, None)
            .unwrap()
            .identity,
        audio.identity
    );
    std::fs::remove_dir_all(directory).unwrap();
}

fn write_bwf(path: &Path, rate: u32, reference: Option<u64>) {
    let mut writer =
        bwavfile::WaveWriter::create(path, bwavfile::WaveFmt::new_pcm_mono(rate, 16)).unwrap();
    if let Some(time_reference) = reference {
        writer
            .write_broadcast_metadata(&bwavfile::Bext {
                description: "Synthetic timing fixture".into(),
                originator: "CleanSlate".into(),
                originator_reference: String::new(),
                origination_date: "2026-10-08".into(),
                origination_time: "00:00:00".into(),
                time_reference,
                version: 0,
                umid: None,
                loudness_value: None,
                loudness_range: None,
                max_true_peak_level: None,
                max_momentary_loudness: None,
                max_short_term_loudness: None,
                coding_history: String::new(),
            })
            .unwrap();
    }
    let mut frames = writer.audio_frame_writer().unwrap();
    frames.write_frames(&vec![0.0; rate as usize / 10]).unwrap();
    frames.end().unwrap();
}

#[test]
fn bwf_origins_are_compared_in_time_and_conflicts_cannot_be_declared_away() {
    let directory =
        std::env::temp_dir().join(format!("cleanslate-bwf-time-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let paths: Vec<_> = [44_100, 48_000, 96_000]
        .into_iter()
        .map(|rate| {
            let path = directory.join(format!("{rate}.wav"));
            write_bwf(&path, rate, Some(u64::from(rate) * 10));
            path
        })
        .collect();
    let validation = rust_app::validate(&paths).unwrap();
    assert!(validation.alignment_verified);
    assert!(validation.require_alignment(false).is_ok());
    write_bwf(&paths[2], 96_000, None);
    assert!(!rust_app::validate(&paths).unwrap().alignment_verified);
    write_bwf(&paths[1], 48_000, Some(48_000 * 11));
    assert!(rust_app::validate(&paths).is_err());
    let mut project =
        rust_app::Project::create(directory.join("conflict.cleanslate"), paths).unwrap();
    project.set_supplied_alignment(true).unwrap();
    assert!(project.require_alignment().is_err());
    let error =
        rust_app::editor_audition_envelopes(&project, None, rust_app::AuditionMode::Original)
            .unwrap_err();
    assert!(error.to_string().contains("conflicting BEXT"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn aac_delay_is_visible_and_native_duration_mismatch_is_never_trimmed() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    let aac = directory.join("mono44100-aac.m4a");
    let validation = rust_app::validate(std::slice::from_ref(&aac)).unwrap();
    assert!(
        validation
            .warnings
            .iter()
            .any(|warning| warning.contains("AAC"))
    );
    assert!(validation.require_alignment(false).is_err());
    assert!(validation.require_alignment(true).is_ok());
    assert_eq!(
        rust_app::SourceMetadata::read(&aac)
            .unwrap()
            .grid
            .frame_count,
        6_144
    );
    assert!(rust_app::validate(&[aac, directory.join("mono44100.aiff")]).is_err());
}

#[test]
fn deleted_derived_audio_is_regenerated_without_changing_the_source() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import/mono48000.wav");
    let source_bytes = std::fs::read(&source).unwrap();
    let directory =
        std::env::temp_dir().join(format!("cleanslate-cache-missing-{}", std::process::id()));
    let audio = rust_app::prepare_analysis_audio_in(&source, &directory, None).unwrap();
    let accepted = std::fs::read(audio.path()).unwrap();
    std::fs::remove_file(audio.path()).unwrap();
    let rebuilt = rust_app::prepare_analysis_audio_in(&source, &directory, None).unwrap();
    assert_eq!(rebuilt.identity, audio.identity);
    assert_eq!(std::fs::read(rebuilt.path()).unwrap(), accepted);
    assert_eq!(std::fs::read(&source).unwrap(), source_bytes);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "requires the owner's Git-ignored three-track recording"]
fn owner_recording_prepares_sequentially_with_native_identity_and_bounded_reads() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("CLEANSLATE_IMPORT_TEST_AUDIO")
            .expect("set CLEANSLATE_IMPORT_TEST_AUDIO to the local test-audio directory"),
    );
    let cache = std::env::temp_dir().join("cleanslate-owner-import-acceptance");
    let tracks = std::env::var("CLEANSLATE_IMPORT_TEST_TRACKS")
        .ok()
        .map(|value| value.parse::<u8>().unwrap())
        .unwrap_or(3);
    assert!((1..=3).contains(&tracks));
    for track in 1..=tracks {
        let source = directory.join(format!("261005-channel-{track}.wav"));
        let audio = rust_app::prepare_analysis_audio_in(&source, &cache, None).unwrap();
        assert_eq!(audio.identity.source.grid.sample_rate, 48_000);
        assert_eq!(audio.identity.source.grid.frame_count, 154_128_376);
        assert_eq!(audio.identity.frame_count, 51_376_126);
        assert_eq!(audio.read_frames(0..512).unwrap().len(), 512);
        let end = audio.identity.frame_count as usize;
        assert_eq!(audio.read_frames(end - 512..end).unwrap().len(), 512);
        assert_eq!(
            rust_app::SourceMetadata::read(&source).unwrap(),
            audio.identity.source
        );
        println!(
            "track {track}: native_frames={}, analysis_frames={}, source_sha256={}",
            audio.identity.source.grid.frame_count,
            audio.identity.frame_count,
            audio.identity.source_fingerprint
        );
    }
}
