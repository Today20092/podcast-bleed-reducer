use rust_app::{PreparedAnalysisAudio, Project, TICKS_PER_SECOND, TimeRange};
use std::path::PathBuf;
pub fn session() -> (PathBuf, Project, Vec<PreparedAnalysisAudio>) {
    session_gain(1.0)
}
pub fn session_gain(quiet_gain: f32) -> (PathBuf, Project, Vec<PreparedAnalysisAudio>) {
    session_rates_gain([16000, 16000], quiet_gain)
}
// Shared by several integration binaries; only delivery tests need mixed rates.
#[allow(dead_code)]
pub fn session_at_rates(rates: [u32; 2]) -> (PathBuf, Project, Vec<PreparedAnalysisAudio>) {
    session_rates_gain(rates, 1.0)
}
fn session_rates_gain(
    rates: [u32; 2],
    quiet_gain: f32,
) -> (PathBuf, Project, Vec<PreparedAnalysisAudio>) {
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
    for (track, &rate) in rates.iter().enumerate() {
        let path = root.join(format!("speaker-{track}.wav"));
        let mut format = bwavfile::WaveFmt::new_pcm_mono(rate, 32);
        format.tag = 3;
        let mut writer = bwavfile::WaveWriter::create(&path, format)
            .unwrap()
            .audio_frame_writer()
            .unwrap();
        let samples: Vec<_> = (0..rate as usize * 4)
            .map(|frame| {
                let i = frame * 16000 / rate as usize;
                let a = noise[i];
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
    (root, project, audio)
}
