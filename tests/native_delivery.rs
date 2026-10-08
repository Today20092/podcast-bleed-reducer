use rust_app::{
    AutomixParameters, Decision, Region, RunRequest, SourceMetadata, TICKS_PER_SECOND,
    run_with_parameters,
};

#[test]
fn mixed_rate_delivery_preserves_native_grids_and_shared_physical_gain() {
    let root =
        std::env::temp_dir().join(format!("cleanslate-native-delivery-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut sources = Vec::new();
    for rate in [44_100, 48_000, 96_000] {
        let path = root.join(format!("{rate}.wav"));
        let mut format = bwavfile::WaveFmt::new_pcm_mono(rate, 32);
        format.tag = 3;
        let writer = bwavfile::WaveWriter::create(&path, format).unwrap();
        let mut audio = writer.audio_frame_writer().unwrap();
        audio.write_frames(&vec![0.5; rate as usize / 10]).unwrap();
        audio.end().unwrap();
        sources.push(path);
    }
    let result = run_with_parameters(
        RunRequest {
            stems: sources.clone(),
            output_dir: root.join("render"),
            regions: vec![Region {
                start: 0,
                end: TICKS_PER_SECOND / 10,
                decision: Decision::NoSpeech,
            }],
            protections: vec![],
            cancelled: None,
            supplied_alignment: true,
        },
        None,
        AutomixParameters {
            attenuation_db: -20.0,
            ..Default::default()
        },
    )
    .unwrap();
    for (path, rate) in result.stems.iter().zip([44_100, 48_000, 96_000]) {
        let metadata = SourceMetadata::read(path).unwrap();
        assert_eq!(metadata.grid.sample_rate, rate);
        assert_eq!(metadata.grid.frame_count, u64::from(rate / 10));
        let mut reader = bwavfile::WaveReader::open(path)
            .unwrap()
            .audio_frame_reader()
            .unwrap();
        let mut samples = vec![0.0; rate as usize / 10];
        reader.read_frames(&mut samples).unwrap();
        assert!((samples[rate as usize / 20] - 0.05).abs() < 1e-6);
    }
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("render/report.json")).unwrap()).unwrap();
    assert_eq!(report["input"]["sample_count"], 4_410);
    for section in ["input", "output"] {
        for (track, rate) in [44_100, 48_000, 96_000].into_iter().enumerate() {
            assert_eq!(
                report[section]["track_grids"][track]["grid"]["sample_rate"],
                rate
            );
            assert_eq!(
                report[section]["track_grids"][track]["grid"]["frame_count"],
                rate / 10
            );
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unity_native_pcm_and_bwf_integrity_survive_delivery() {
    let root = std::env::temp_dir().join(format!("cleanslate-native-unity-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for (rate, bits) in [(44_100, 16), (96_000, 24)] {
        let path = root.join(format!("{rate}-{bits}.wav"));
        let mut writer =
            bwavfile::WaveWriter::create(&path, bwavfile::WaveFmt::new_pcm_mono(rate, bits))
                .unwrap();
        let bext = bwavfile::Bext {
            description: "native unity".into(),
            originator: "CleanSlate".into(),
            originator_reference: String::new(),
            origination_date: "2026-10-08".into(),
            origination_time: "00:00:01".into(),
            time_reference: u64::from(rate) * 2,
            version: 0,
            umid: None,
            loudness_value: None,
            loudness_range: None,
            max_true_peak_level: None,
            max_momentary_loudness: None,
            max_short_term_loudness: None,
            coding_history: String::new(),
        };
        writer.write_broadcast_metadata(&bext).unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(&[0.0, 0.25, -0.5, 0.75]).unwrap();
        frames.end().unwrap();
        let before = std::fs::read(&path).unwrap();
        let result = run_with_parameters(
            RunRequest {
                supplied_alignment: true,
                stems: vec![path.clone()],
                output_dir: root.join(format!("out-{rate}")),
                regions: vec![Region {
                    start: 0,
                    end: SourceMetadata::read(&path)
                        .unwrap()
                        .grid
                        .duration()
                        .unwrap(),
                    decision: Decision::Uncertain,
                }],
                protections: vec![],
                cancelled: None,
            },
            None,
            AutomixParameters::default(),
        )
        .unwrap();
        let mut output = bwavfile::WaveReader::open(&result.stems[0]).unwrap();
        assert_eq!(output.format().unwrap().sample_rate, rate);
        assert_eq!(output.format().unwrap().bits_per_sample, bits);
        assert_eq!(
            output
                .broadcast_extension()
                .unwrap()
                .unwrap()
                .time_reference,
            u64::from(rate) * 2
        );
        assert_eq!(output.frame_length().unwrap(), 4);
        let mut decoded = [0.0; 4];
        output
            .audio_frame_reader()
            .unwrap()
            .read_frames(&mut decoded)
            .unwrap();
        assert_eq!(decoded, [0.0, 0.25, -0.5, 0.75]);
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn synchronized_impulses_export_on_each_original_native_grid() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    let root =
        std::env::temp_dir().join(format!("cleanslate-native-impulse-{}", std::process::id()));
    let sources = [44_100, 48_000, 96_000]
        .map(|rate| fixture.join(format!("impulses{rate}.wav")))
        .to_vec();
    let result = run_with_parameters(
        RunRequest {
            supplied_alignment: true,
            stems: sources,
            output_dir: root.clone(),
            regions: vec![Region {
                start: 0,
                end: TICKS_PER_SECOND / 10,
                decision: Decision::Uncertain,
            }],
            protections: vec![],
            cancelled: None,
        },
        None,
        AutomixParameters::default(),
    )
    .unwrap();
    for (path, rate) in result.stems.iter().zip([44_100, 48_000, 96_000]) {
        let mut reader = bwavfile::WaveReader::open(path)
            .unwrap()
            .audio_frame_reader()
            .unwrap();
        let mut samples = vec![0.0; rate / 10];
        reader.read_frames(&mut samples).unwrap();
        for expected in [rate / 100, rate * 9 / 100] {
            let peak = (expected - 3..expected + 4)
                .max_by(|&a, &b| samples[a].abs().total_cmp(&samples[b].abs()))
                .unwrap();
            assert_eq!(peak, expected);
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
