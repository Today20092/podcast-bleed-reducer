use bwavfile::{Bext, WaveFmt, WaveReader, WaveWriter};
use rust_app::{
    AutomixParameters, Decision, Protection, Region, RunRequest, run, run_with_parameters,
    run_with_progress,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
};
use std::time::{Duration, Instant};
use std::{fs, path::Path};

fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        std::thread::yield_now();
    }
}

fn wav(path: &Path, format: u16, samples: &[f32], bext: Option<u64>) {
    let fmt = if format == 3 {
        WaveFmt {
            tag: 3,
            channel_count: 1,
            sample_rate: 48_000,
            bytes_per_second: 192_000,
            block_alignment: 4,
            bits_per_sample: 32,
            extended_format: None,
        }
    } else {
        WaveFmt::new_pcm_mono(48_000, 16)
    };
    let mut writer = WaveWriter::create(path, fmt).unwrap();
    if let Some(time_reference) = bext {
        writer
            .write_broadcast_metadata(&Bext {
                description: "fixture".into(),
                originator: "CleanSlate".into(),
                originator_reference: String::new(),
                origination_date: "2026-08-14".into(),
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
    frames.write_frames(samples).unwrap();
    frames.end().unwrap();
}

#[test]
fn renders_float_overlap_and_protection_and_round_trips_bext() {
    let root = std::env::temp_dir().join(format!("cleanslate-metadata-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let a = root.join("a.wav");
    let b = root.join("b.wav");
    wav(&a, 3, &vec![0.5; 3_000], Some(42));
    wav(&b, 3, &vec![0.5; 3_000], Some(42));
    let progress = Arc::new(AtomicU8::new(0));
    let result = run_with_progress(
        RunRequest {
            stems: vec![a, b],
            output_dir: root.join("rendered"),
            regions: vec![
                Region {
                    start: 0,
                    end: 1_000,
                    decision: Decision::Owned(vec![0]),
                },
                Region {
                    start: 1_000,
                    end: 2_000,
                    decision: Decision::Uncertain,
                },
                Region {
                    start: 2_000,
                    end: 3_000,
                    decision: Decision::NoSpeech,
                },
            ],
            protections: vec![Protection {
                track: Some(1),
                start: 2_500,
                end: 3_000,
            }],
            cancelled: None,
        },
        Some(Arc::clone(&progress)),
    )
    .unwrap();
    assert_eq!(progress.load(Ordering::Relaxed), 100);
    assert!(result.warnings.is_empty());
    for (track, output) in result.stems.into_iter().enumerate() {
        let mut reader = WaveReader::open(output).unwrap();
        assert_eq!(reader.format().unwrap().tag, 3);
        assert_eq!(reader.frame_length().unwrap(), 3_000);
        assert_eq!(
            reader
                .broadcast_extension()
                .unwrap()
                .unwrap()
                .time_reference,
            42
        );
        let mut samples = vec![0.0_f32; 3_000];
        reader
            .audio_frame_reader()
            .unwrap()
            .read_frames(&mut samples)
            .unwrap();
        if track == 0 {
            assert!(samples[1_999] > 0.49); // owned/uncertain overlap remains at unity
            let floor = 0.125_892_53;
            let expected = 0.5 * (floor + (1.0 - floor) * (-1_000.0_f32 / 24_000.0).exp());
            assert!((samples[2_999] - expected).abs() < 0.000_01); // one-pole close toward -18 dB
        } else {
            assert!((samples[999] - 0.5 * 0.125_892_53).abs() < 0.000_01); // non-owner -18 dB
            let floor = 0.125_892_53;
            let opening =
                |elapsed| 0.5 * (1.0 - (1.0 - floor) * (-(elapsed as f32) / 2_400.0).exp());
            assert!((samples[1_999] - opening(1_000)).abs() < 0.000_01); // uncertain opens toward unity
            let opened = opening(1_000) / 0.5;
            let closed = floor + (opened - floor) * (-500.0_f32 / 24_000.0).exp();
            let protected = 0.5 * (1.0 - (1.0 - closed) * (-500.0_f32 / 2_400.0).exp());
            assert!((samples[2_999] - protected).abs() < 0.000_01); // protection reverses the close smoothly
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn renders_one_pcm_stem_and_report_without_touching_source() {
    let root = std::env::temp_dir().join(format!("cleanslate-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let source = root.join("voice.wav");
    wav(&source, 1, &[0.5, -0.5], None);
    let original = fs::read(&source).unwrap();
    let result = run(RunRequest {
        stems: vec![source.clone()],
        output_dir: root.join("rendered"),
        regions: vec![Region {
            start: 0,
            end: 2,
            decision: Decision::NoSpeech,
        }],
        protections: vec![],
        cancelled: None,
    })
    .unwrap();
    assert_eq!(fs::read(source).unwrap(), original);
    assert_eq!(result.stems.len(), 1);
    assert!(result.report.exists());
    assert_eq!(
        result.warnings,
        vec!["alignment could not be verified: no stems contain BEXT"]
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn automixes_silence_and_records_inspectable_evidence() {
    let root = std::env::temp_dir().join(format!("cleanslate-automix-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let source = root.join("silence.wav");
    wav(&source, 3, &vec![0.000_01; 48_000], Some(42));
    let parameters = AutomixParameters {
        attenuation_db: -12.0,
        ..AutomixParameters::default()
    };
    let result = run_with_parameters(
        RunRequest {
            stems: vec![source],
            output_dir: root.join("rendered"),
            regions: vec![],
            protections: vec![],
            cancelled: None,
        },
        None,
        parameters,
    )
    .unwrap();

    let report = fs::read_to_string(result.report).unwrap();
    assert!(report.contains("\"detector\": \"silero-vad-v5\""));
    assert!(report.contains("\"speech_probability\": 0.5"));
    assert!(report.contains("\"energy_margin_db\": 6"));
    assert!(report.contains("\"attenuation_db\": -12"));
    assert!(report.contains("\"decision\": \"NoSpeech\""));
    assert!(report.contains("\"affected_track\": 0"));
    assert!(report.contains("\"evidence_reasons\""));
    assert!(report.contains("\"protections\": []"));

    let reader = WaveReader::open(&result.stems[0]).unwrap();
    let mut samples = vec![0.0_f32; 48_000];
    reader
        .audio_frame_reader()
        .unwrap()
        .read_frames(&mut samples)
        .unwrap();
    assert!((samples[47_999] - 0.000_01 * 10.0_f32.powf(-12.0 / 20.0)).abs() < 0.000_000_01);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn blocks_empty_stems_instead_of_panicking() {
    let root = std::env::temp_dir().join(format!("cleanslate-empty-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let source = root.join("empty.wav");
    wav(&source, 3, &[], Some(42));
    let error = run(RunRequest {
        stems: vec![source],
        output_dir: root.join("rendered"),
        regions: vec![],
        protections: vec![],
        cancelled: None,
    })
    .unwrap_err();
    assert!(error.to_string().contains("no audio samples"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn blocks_invalid_owner_sets() {
    let root = std::env::temp_dir().join(format!("cleanslate-owners-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let source = root.join("voice.wav");
    wav(&source, 3, &[0.25; 10], Some(42));
    let error = run(RunRequest {
        stems: vec![source],
        output_dir: root.join("rendered"),
        regions: vec![Region {
            start: 0,
            end: 10,
            decision: Decision::Owned(vec![1]),
        }],
        protections: vec![],
        cancelled: None,
    })
    .unwrap_err();
    assert!(error.to_string().contains("invalid owner set"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn blocks_bad_alignment_and_cleans_up_failure_and_cancellation() {
    let root = std::env::temp_dir().join(format!("cleanslate-errors-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let a = root.join("a.wav");
    let b = root.join("b.wav");
    wav(&a, 1, &[0.0; 10], Some(1));
    wav(&b, 1, &[0.0; 10], None);
    let output = root.join("rendered");
    let error = run(RunRequest {
        stems: vec![a.clone(), b.clone()],
        output_dir: output.clone(),
        regions: vec![],
        protections: vec![],
        cancelled: None,
    })
    .unwrap_err();
    assert!(error.to_string().contains("mixed BEXT"));
    assert!(!output.exists());

    wav(&a, 1, &vec![0.0; 1_000_000], Some(1));
    wav(&b, 1, &vec![0.0; 1_000_000], Some(1));
    let staging = root.join(format!(".rendered.{}.tmp", std::process::id()));
    let first_stem = staging.join("01-a.wav");
    let blocked_second_stem = staging.join("02-b.wav");
    let blocker = std::thread::spawn(move || {
        wait_for(&first_stem);
        fs::create_dir(blocked_second_stem).unwrap();
    });
    let error = run(RunRequest {
        stems: vec![a.clone(), b.clone()],
        output_dir: output.clone(),
        regions: vec![],
        protections: vec![],
        cancelled: None,
    })
    .unwrap_err();
    blocker.join().unwrap();
    assert!(!error.to_string().is_empty());
    assert!(!output.exists());

    let cancelled = Arc::new(AtomicBool::new(false));
    let cancel = cancelled.clone();
    let first_stem = staging.join("01-a.wav");
    let canceller = std::thread::spawn(move || {
        wait_for(&first_stem);
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let error = run(RunRequest {
        stems: vec![a, b],
        output_dir: output.clone(),
        regions: vec![],
        protections: vec![],
        cancelled: Some(cancelled),
    })
    .unwrap_err();
    canceller.join().unwrap();
    assert!(error.to_string().contains("cancelled"));
    assert!(!output.exists());
    fs::remove_dir_all(root).unwrap();
}
