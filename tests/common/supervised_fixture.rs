use rust_app::model::{
    DiarizationEvidence, ProbabilityFrames, ProbabilityHistoryWriter, RuntimeProvenance,
};
use rust_app::{PreparedAnalysisAudio, Project, TICKS_PER_SECOND, TimeRange};
use std::path::PathBuf;
pub fn session() -> (
    PathBuf,
    Project,
    Vec<PreparedAnalysisAudio>,
    DiarizationEvidence,
) {
    session_gain(1.0)
}
pub fn session_gain(
    quiet_gain: f32,
) -> (
    PathBuf,
    Project,
    Vec<PreparedAnalysisAudio>,
    DiarizationEvidence,
) {
    let root = std::env::temp_dir().join(format!(
        "cleanslate-ownership-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut state = 7654321_u32;
    let noise: Vec<f32> = (0..64_000)
        .map(|_| {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            ((state >> 8) as f32 / 16777216.0 - 0.5) * 0.2
        })
        .collect();
    let mut paths = vec![];
    for track in 0..2 {
        let path = root.join(format!("speaker-{track}.wav"));
        let mut format = bwavfile::WaveFmt::new_pcm_mono(16000, 32);
        format.tag = 3;
        let mut writer = bwavfile::WaveWriter::create(&path, format)
            .unwrap()
            .audio_frame_writer()
            .unwrap();
        let samples: Vec<_> = noise
            .iter()
            .enumerate()
            .map(|(i, &a)| {
                let b = noise[(i + 700) % noise.len()];
                let active_a = (8000..24000).contains(&i) || (48000..60000).contains(&i);
                let active_b = (32000..44000).contains(&i) || (52000..60000).contains(&i);
                let (a, b) = (
                    if active_a { a } else { a * 0.0001 },
                    if active_b {
                        b * if i >= 52000 { quiet_gain } else { 1.0 }
                    } else {
                        b * 0.0001
                    },
                );
                if track == 0 { a + 0.1 * b } else { b + 0.2 * a }
            })
            .collect();
        writer.write_frames(&samples).unwrap();
        writer.end().unwrap();
        paths.push(path);
    }
    let mut project = Project::create(root.join("project.cleanslate"), paths).unwrap();
    project.set_supplied_alignment(true).unwrap();
    project
        .add_reference_label(
            TimeRange::new(0, TICKS_PER_SECOND / 4).unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    for (track, (start, end)) in [
        (TICKS_PER_SECOND / 2, TICKS_PER_SECOND * 3 / 2),
        (2 * TICKS_PER_SECOND, TICKS_PER_SECOND * 11 / 4),
    ]
    .into_iter()
    .enumerate()
    {
        project
            .add_reference_label(
                TimeRange::new(start, end).unwrap(),
                vec![project.sources[track].speaker_id],
                &[],
            )
            .unwrap();
    }
    let audio: Vec<_> = project
        .source_paths()
        .iter()
        .map(|path| rust_app::prepare_analysis_audio_in(path, &root.join("copies"), None).unwrap())
        .collect();
    let mut writer = ProbabilityHistoryWriter::create(&root.join("neural.f32")).unwrap();
    let probabilities: Vec<_> = (0..401)
        .flat_map(|frame| {
            (0..8).map(move |channel| {
                let active = match channel {
                    0 => (50..150).contains(&frame) || (300..375).contains(&frame),
                    1 => (200..275).contains(&frame) || (325..375).contains(&frame),
                    _ => false,
                };
                if active { 0.95 } else { 0.01 }
            })
        })
        .collect();
    writer
        .append(&ProbabilityFrames {
            start: 0,
            frames: 401,
            speakers: 8,
            seconds_per_frame: 0.01,
            probabilities,
        })
        .unwrap();
    let evidence = DiarizationEvidence {
        schema: 2,
        sources: audio.iter().map(|a| a.identity.clone()).collect(),
        input_sha256: audio.iter().map(|a| a.audio_sha256().to_string()).collect(),
        history: writer.finish().unwrap(),
        runtime: RuntimeProvenance {
            revision: rust_app::model::RUNTIME_REVISION.into(),
            recipe_sha256: "1".repeat(64),
            binaries: Default::default(),
        },
        model_revision: rust_app::model::MODEL_REVISION.into(),
        model_sha256: rust_app::model::NEMOTRON.sha256.into(),
        preset: "v3-offline".into(),
        mix_recipe: "mean-global-peak-floor-one-v1".into(),
        mix_peak: 1.0,
        mix_gains: vec![0.5; 2],
        sample_count: 64000,
        reported_seconds_per_frame: 0.01,
        compactions: 0,
        maximum_retained_frames: 401,
    };
    (root, project, audio, evidence)
}
