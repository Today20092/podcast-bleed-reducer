use bwavfile::{WaveFmt, WaveWriter};
use rust_app::{AutomixParameters, PreviewCache, Protection, analyze_preview};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicU8},
};

fn wav(path: &Path, count: usize) {
    let writer = WaveWriter::create(path, WaveFmt::new_pcm_mono(48_000, 16)).unwrap();
    let mut frames = writer.audio_frame_writer().unwrap();
    let samples: Vec<f32> = (0..count)
        .map(|i| {
            if i < count / 2 {
                0.0
            } else {
                (i as f32 * 0.03).sin() * 0.2
            }
        })
        .collect();
    frames.write_frames(&samples).unwrap();
    frames.end().unwrap();
}

#[test]
fn warm_edits_match_cold_and_reuse_detector_and_ownership() {
    let root = std::env::temp_dir().join(format!("prepared-equivalence-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let paths = vec![root.join("a.wav"), root.join("b.wav")];
    for path in &paths {
        wav(path, 12_001);
    }
    let mut cache = PreviewCache::default();
    let mut parameters = AutomixParameters::default();
    let progress = Arc::new(AtomicU8::new(0));
    cache
        .analyze(&paths, &[], parameters, &progress, None)
        .unwrap();
    parameters.attenuation_db = -9.0;
    parameters.open_ms = 10.0;
    parameters.close_ms = 100.0;
    let protections = vec![Protection {
        track: Some(0),
        start: rust_app::SourceTimebase::new(48_000, 23)
            .unwrap()
            .duration()
            .unwrap(),
        end: rust_app::SourceTimebase::new(48_000, 901)
            .unwrap()
            .duration()
            .unwrap(),
    }];
    let warm = cache
        .analyze(&paths, &protections, parameters, &progress, None)
        .unwrap();
    assert_eq!(
        warm,
        analyze_preview(&paths, &protections, parameters, progress.clone()).unwrap()
    );
    assert_eq!(cache.stats().preparations, 1);
    assert_eq!(cache.stats().ownership_derivations, 1);
    parameters.speech_sensitivity = 0.7;
    parameters.bleed_discrimination_db = 3.0;
    parameters.speech_padding_ms = 32.0;
    let warm = cache
        .analyze(&paths, &protections, parameters, &progress, None)
        .unwrap();
    assert_eq!(
        warm,
        analyze_preview(&paths, &protections, parameters, progress).unwrap()
    );
    assert_eq!(cache.stats().preparations, 1);
    assert_eq!(cache.stats().ownership_derivations, 2);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn ordered_relinked_and_modified_sources_invalidate_preparation() {
    let root = std::env::temp_dir().join(format!("prepared-identity-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let a = root.join("a.wav");
    let b = root.join("b.wav");
    let c = root.join("c.wav");
    for path in [&a, &b, &c] {
        wav(path, 3073);
    }
    let mut cache = PreviewCache::default();
    let parameters = AutomixParameters::default();
    let progress = AtomicU8::new(0);
    cache
        .analyze(&[a.clone(), b.clone()], &[], parameters, &progress, None)
        .unwrap();
    cache
        .analyze(&[b.clone(), a.clone()], &[], parameters, &progress, None)
        .unwrap();
    assert_eq!(cache.stats().preparations, 2);
    cache
        .analyze(&[b.clone(), c.clone()], &[], parameters, &progress, None)
        .unwrap();
    assert_eq!(cache.stats().preparations, 3);
    // Same byte length; a guaranteed different timestamp avoids clock-resolution assumptions.
    let modified = std::fs::metadata(&c).unwrap().modified().unwrap();
    wav(&c, 3073);
    std::fs::File::options()
        .write(true)
        .open(&c)
        .unwrap()
        .set_times(
            std::fs::FileTimes::new().set_modified(modified + std::time::Duration::from_secs(2)),
        )
        .unwrap();
    cache
        .analyze(&[b.clone(), c.clone()], &[], parameters, &progress, None)
        .unwrap();
    assert_eq!(cache.stats().preparations, 4);
    std::fs::write(&c, b"invalid replacement").unwrap();
    assert!(
        cache
            .analyze(&[b, c], &[], parameters, &progress, None)
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cancellation_preserves_completed_cache_and_invalid_protections_are_blocked() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let root = std::env::temp_dir().join(format!("prepared-cancel-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let paths = vec![root.join("a.wav")];
    wav(&paths[0], 3073);
    let mut cache = PreviewCache::default();
    let parameters = AutomixParameters::default();
    let progress = AtomicU8::new(0);
    let cancelled = AtomicBool::new(true);
    assert!(
        cache
            .analyze(&paths, &[], parameters, &progress, Some(&cancelled))
            .unwrap_err()
            .to_string()
            .contains("cancelled")
    );
    assert_eq!(cache.stats().preparations, 0);
    cancelled.store(false, Ordering::Relaxed);
    let expected = cache
        .analyze(&paths, &[], parameters, &progress, Some(&cancelled))
        .unwrap();
    cancelled.store(true, Ordering::Relaxed);
    assert!(
        cache
            .analyze(&paths, &[], parameters, &progress, Some(&cancelled))
            .is_err()
    );
    assert_eq!(
        cache
            .analyze(&paths, &[], parameters, &progress, None)
            .unwrap(),
        expected
    );
    assert_eq!(cache.stats().preparations, 1);
    let invalid = [Protection {
        track: Some(1),
        start: rust_app::SourceTimebase::new(48_000, 0)
            .unwrap()
            .duration()
            .unwrap(),
        end: u64::MAX,
    }];
    assert!(
        cache
            .analyze(&paths, &invalid, parameters, &progress, None)
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}
