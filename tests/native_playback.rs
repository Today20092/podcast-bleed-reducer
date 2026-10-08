use rust_app::PreparedAudio;
use std::sync::atomic::AtomicBool;

#[test]
fn playback_modes_and_seek_share_source_time_on_the_device_grid() {
    use rust_app::{AuditionMode, GainEnvelope};
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/import/mono96000.wav");
    let prepared =
        PreparedAudio::prepare_for_rate(&[path], 44_100, &AtomicBool::new(false)).unwrap();
    // A native-grid envelope ending at 50 ms must expose unity at that physical
    // boundary, not at device frame 4,800 (approximately 109 ms).
    let envelope = GainEnvelope::constant_at_rate(4_800, 96_000, 0.25).unwrap();
    let pcm = prepared.read_frames(0..prepared.frame_count()).unwrap();
    let mut source = prepared
        .playback(
            vec![envelope],
            AuditionMode::Processed,
            0.5,
            0,
            prepared.duration(),
        )
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    let mut observed = 0;
    while observed < 200 {
        let sample = source.next().unwrap();
        if source.position_frame() != observed {
            let frame = source.position_frame() - 1;
            assert!((sample - pcm[0][frame] * 0.125).abs() < 1e-6);
            observed = source.position_frame();
        } else {
            std::thread::yield_now();
        }
        assert!(std::time::Instant::now() < deadline);
    }
    source.seek(28_224_000 * 3 / 40);
    source.set_mode(AuditionMode::Removed);
    loop {
        let sample = source.next().unwrap();
        if source.position_frame() == 3_308 {
            assert!(
                pcm[0][3_307].abs() > 1e-5,
                "fixture must expose an incorrectly retained attenuation"
            );
            assert_eq!(sample, 0.0);
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
}

#[test]
fn mixed_native_impulses_share_a_delay_compensated_playback_grid() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import");
    let paths = [44_100, 48_000, 96_000].map(|rate| root.join(format!("impulses{rate}.wav")));
    let audio = PreparedAudio::prepare_for_rate(&paths, 48_000, &AtomicBool::new(false)).unwrap();
    assert_eq!(audio.sample_rate(), 48_000);
    assert_eq!(audio.frame_count(), 4_800);
    let tracks = audio.read_frames(0..4_800).unwrap();
    for track in tracks {
        for (range, expected) in [(450..510, 480), (4_290..4_350, 4_320)] {
            let peak = range
                .max_by(|&a, &b| track[a].abs().total_cmp(&track[b].abs()))
                .unwrap();
            assert_eq!(peak, expected);
        }
    }
}

#[test]
fn bounded_audition_lifecycle_and_native_envelope_validation() {
    use rust_app::{AuditionMode, GainEnvelope};
    assert!(GainEnvelope::constant_at_rate(100, 0, 0.5).is_err());
    assert!(GainEnvelope::constant_at_rate(100, 96_000, f32::NAN).is_err());
    assert!(GainEnvelope::constant_at_rate(100, 96_000, 1.1).is_err());
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/import/mono96000.wav");
    let audio = PreparedAudio::prepare_for_rate(&[path], 48_000, &AtomicBool::new(false)).unwrap();
    assert!(audio.read_frames(0..65_537).is_err());
    let started = std::time::Instant::now();
    for _ in 0..20 {
        let source = audio
            .playback(vec![], AuditionMode::Original, 0.5, 0, audio.duration())
            .unwrap();
        assert!(source.set_monitor(f32::NAN).is_err());
        drop(source); // Cancels and joins the owned producer outside the callback.
    }
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
}
