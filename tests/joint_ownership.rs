use bwavfile::{WaveFmt, WaveWriter};
use rust_app::{
    AutomixParameters, CalibrationExample, CalibrationKind, OwnershipSettings, PreviewCache,
    SourceStamp,
};
use std::sync::atomic::AtomicU8;

#[test]
fn calibration_edits_reuse_prepared_audio_and_reject_stale_sources() {
    let root = std::env::temp_dir().join(format!("joint-cache-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("source.wav");
    let writer = WaveWriter::create(&path, WaveFmt::new_pcm_mono(48_000, 16)).unwrap();
    let mut audio = writer.audio_frame_writer().unwrap();
    audio.write_frames(&vec![0.0_f32; 48_000]).unwrap();
    audio.end().unwrap();
    let paths = vec![path];
    let mut cache = PreviewCache::default();
    let progress = AtomicU8::new(0);
    let parameters = AutomixParameters::default();
    cache
        .analyze(&paths, &[], parameters, &progress, None)
        .unwrap();
    let settings = OwnershipSettings {
        enabled: true,
        examples: vec![CalibrationExample {
            kind: CalibrationKind::RoomTone,
            track: 0,
            start: rust_app::SourceTimebase::new(48_000, (0) as u64)
                .unwrap()
                .duration()
                .unwrap(),
            end: rust_app::SourceTimebase::new(48_000, (24_000) as u64)
                .unwrap()
                .duration()
                .unwrap(),
        }],
        sources: paths
            .iter()
            .map(|p| SourceStamp::read(p).unwrap())
            .collect(),
    };
    let warm = cache
        .analyze_with_ownership(&paths, &[], parameters, &settings, &progress, None)
        .unwrap();
    let cold = PreviewCache::default()
        .analyze_with_ownership(&paths, &[], parameters, &settings, &progress, None)
        .unwrap();
    assert_eq!(warm, cold);
    assert_eq!(cache.stats().preparations, 1);
    assert_eq!(cache.stats().ownership_derivations, 2);
    let mut stale = settings.clone();
    stale.sources.clear();
    assert!(
        cache
            .analyze_with_ownership(&paths, &[], parameters, &stale, &progress, None)
            .unwrap_err()
            .to_string()
            .contains("source")
    );
    let mut invalid = settings.clone();
    invalid.examples[0].end = 100;
    assert!(
        cache
            .analyze_with_ownership(&paths, &[], parameters, &invalid, &progress, None)
            .is_err()
    );
    assert_eq!(
        cache
            .analyze_with_ownership(&paths, &[], parameters, &settings, &progress, None)
            .unwrap(),
        warm
    );
    let bad = AutomixParameters {
        attenuation_db: f32::NAN,
        ..parameters
    };
    assert!(
        cache
            .analyze_with_ownership(&paths, &[], bad, &settings, &progress, None)
            .is_err()
    );
    assert!(cache.joint_diagnostics().unwrap().retained_feature_bytes < 1000);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn joint_render_matches_preview_gain_and_reports_settings() {
    use rust_app::{Protection, RunRequest, run_with_ownership};
    let root = std::env::temp_dir().join(format!("joint-render-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let paths = vec![root.join("direct.wav"), root.join("copy.wav")];
    let source: Vec<f32> = (0..96_000)
        .map(|i| {
            if i < 24_000 {
                0.0
            } else {
                0.15 * (i as f32 * 0.071).sin()
            }
        })
        .collect();
    let copy: Vec<_> = (0..source.len())
        .map(|i| -0.2 * source.get(i.wrapping_sub(120)).copied().unwrap_or(0.0))
        .collect();
    for (path, samples) in paths.iter().zip([&source, &copy]) {
        let format = WaveFmt {
            tag: 3,
            channel_count: 1,
            sample_rate: 48_000,
            bytes_per_second: 192_000,
            block_alignment: 4,
            bits_per_sample: 32,
            extended_format: None,
        };
        let mut writer = WaveWriter::create(path, format)
            .unwrap()
            .audio_frame_writer()
            .unwrap();
        writer.write_frames(samples).unwrap();
        writer.end().unwrap();
    }
    let parameters = AutomixParameters {
        speech_sensitivity: 0.0,
        speech_padding_ms: 0.0,
        attenuation_db: -9.0,
        ..Default::default()
    };
    let settings = OwnershipSettings {
        enabled: true,
        ..Default::default()
    };
    let protections = vec![Protection {
        track: Some(0),
        start: rust_app::SourceTimebase::new(48_000, (0) as u64)
            .unwrap()
            .duration()
            .unwrap(),
        end: u64::MAX,
    }];
    let preview = PreviewCache::default()
        .analyze_with_ownership(
            &paths,
            &protections,
            parameters,
            &settings,
            &AtomicU8::new(0),
            None,
        )
        .unwrap();
    assert!(preview.envelopes[1].gain_at(72_000) < 0.8);
    let rendered = run_with_ownership(
        RunRequest {
            stems: paths.clone(),
            output_dir: root.join("render"),
            regions: vec![],
            protections,
            cancelled: None,
        },
        None,
        parameters,
        &settings,
    )
    .unwrap();
    for (track, original) in [&source, &copy].iter().enumerate() {
        let reader = bwavfile::WaveReader::open(&rendered.stems[track]).unwrap();
        let mut audio = reader.audio_frame_reader().unwrap();
        let mut samples = vec![0.0_f32; source.len()];
        audio.read_frames(&mut samples).unwrap();
        for i in (0..source.len()).step_by(97) {
            assert!((samples[i] - original[i] * preview.envelopes[track].gain_at(i)).abs() < 1e-7);
        }
    }
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(rendered.report).unwrap()).unwrap();
    assert_eq!(report["detector"], "joint-speech-band-copy-v1");
    assert_eq!(report["ownership_settings"]["enabled"], true);
    assert!(
        report["regions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["applied_attenuation_db"] == -9.0)
    );
    std::fs::remove_dir_all(root).unwrap();
}
