use std::path::Path;

fn source(path: &Path, rate: u32, frames: usize) {
    let writer =
        bwavfile::WaveWriter::create(path, bwavfile::WaveFmt::new_pcm_mono(rate, 16)).unwrap();
    let mut audio = writer.audio_frame_writer().unwrap();
    audio.write_frames(&vec![0.0; frames]).unwrap();
    audio.end().unwrap();
}

#[test]
fn unsafe_native_rate_geometry_fails_without_installing_evidence() {
    let root =
        std::env::temp_dir().join(format!("cleanslate-unsafe-geometry-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("unsafe.wav");
    source(&path, 1_048_579, 1);
    let original = std::fs::read(&path).unwrap();
    let cache = root.join("cache");
    let error = rust_app::prepare_analysis_audio_in(&path, &cache, None)
        .err()
        .expect("unsafe geometry must be rejected");
    assert!(error.to_string().contains("FFT geometry"), "{error}");
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read_dir(cache).unwrap().count(), 0);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn safe_low_high_and_odd_native_rates_have_checked_nonzero_conversion_geometry() {
    let root = std::env::temp_dir().join(format!("cleanslate-geometry-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for rate in [1, 7_000, 48_001, 768_000] {
        let path = root.join(format!("{rate}.wav"));
        let count = (rate as usize / 100).max(1);
        source(&path, rate, count);
        let prepared =
            rust_app::prepare_analysis_audio_in(&path, &root.join("cache"), None).unwrap();
        assert_eq!(prepared.identity.source.grid.sample_rate, rate);
        assert_eq!(prepared.identity.source.grid.frame_count, count as u64);
        assert_eq!(
            prepared.identity.frame_count,
            (count as u64 * 16_000).div_ceil(u64::from(rate))
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
