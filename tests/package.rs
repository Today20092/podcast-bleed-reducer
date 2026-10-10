use bwavfile::{WaveFmt, WaveWriter};
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn packaged_gate_exercises_safe_cancellation() {
    let dir = std::env::temp_dir().join(format!(
        "cleanslate-package-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let mut stems = vec![];
    for name in ["a.wav", "b.wav"] {
        let path = dir.join(name);
        let writer = WaveWriter::create(&path, WaveFmt::new_pcm_mono(48_000, 16)).unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(&[0.0_f32; 32]).unwrap();
        frames.end().unwrap();
        stems.push(path);
    }
    let output = dir.join("cancelled");
    let result = Command::new(env!("CARGO_BIN_EXE_release_gate"))
        .arg("--cancel")
        .arg(&output)
        .args(&stems)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!output.exists());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn shipped_executable_exercises_offline_analysis_and_delivery() {
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_rust-app"))
        .arg("--package-self-check")
        .status()
        .unwrap();
    assert!(status.success());
}
