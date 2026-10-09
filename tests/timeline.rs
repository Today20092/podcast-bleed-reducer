use rust_app::{SourceTimebase, TICKS_PER_SECOND, timeline::TimelineViewport};

#[test]
fn captured_lock_survives_seeks_zoom_resize_and_session_edges() {
    use rust_app::timeline::PlaybackScroll;
    let mut viewport = TimelineViewport::new(100 * TICKS_PER_SECOND);
    for _ in 0..6 {
        viewport.zoom_at(0.0, 1000.0, 1.0);
    }
    viewport.seek(viewport.sample_at(300.0, 1000.0));
    let start = viewport.start();
    let visible = viewport.visible_samples();
    viewport.lock_playhead();
    viewport.follow_playhead(PlaybackScroll::Locked);
    assert!((viewport.start() - start).abs() < 1.0);
    viewport.seek(50 * TICKS_PER_SECOND);
    viewport.follow_playhead(PlaybackScroll::Locked);
    assert!((viewport.x_for_sample(viewport.playhead(), 1000.0) - 300.0).abs() < 0.01);
    assert!((viewport.x_for_sample(viewport.playhead(), 2000.0) - 600.0).abs() < 0.01);
    assert_eq!(viewport.visible_samples(), visible);
    viewport.zoom_at(800.0, 1000.0, 1.0);
    viewport.follow_playhead(PlaybackScroll::Locked);
    assert!((viewport.x_for_sample(viewport.playhead(), 1000.0) - 300.0).abs() < 0.01);
    viewport.center_playhead();
    assert!((viewport.x_for_sample(viewport.playhead(), 1000.0) - 500.0).abs() < 0.01);
    viewport.seek(0);
    viewport.follow_playhead(PlaybackScroll::Locked);
    assert_eq!(viewport.start(), 0.0);
    viewport.seek(100 * TICKS_PER_SECOND);
    viewport.follow_playhead(PlaybackScroll::Locked);
    assert_eq!(
        viewport.start(),
        100.0 * TICKS_PER_SECOND as f64 - viewport.visible_samples()
    );
}

#[test]
fn playback_follow_pages_at_the_edge_and_locks_without_changing_zoom_or_selection() {
    use rust_app::timeline::PlaybackScroll;
    let mut viewport = TimelineViewport::new(100 * TICKS_PER_SECOND);
    for _ in 0..5 {
        viewport.zoom_at(0.0, 1000.0, 1.0);
    }
    let visible = viewport.visible_samples();
    viewport.set_selection(Some((TICKS_PER_SECOND, 2 * TICKS_PER_SECOND)));
    viewport.seek(visible.floor() as u64 - 1);
    assert!(!viewport.follow_playhead(PlaybackScroll::Page));
    viewport.seek(visible.ceil() as u64);
    assert!(viewport.follow_playhead(PlaybackScroll::Page));
    assert_eq!(viewport.start(), viewport.playhead() as f64);
    assert!(!viewport.follow_playhead(PlaybackScroll::Page));
    viewport.seek(0);
    assert!(viewport.follow_playhead(PlaybackScroll::Page));
    assert_eq!(viewport.start(), 0.0);
    viewport.seek(50 * TICKS_PER_SECOND);
    assert!(viewport.follow_playhead(PlaybackScroll::Locked));
    assert!((viewport.x_for_sample(viewport.playhead(), 1000.0) - 500.0).abs() < 0.01);
    let start = viewport.start();
    viewport.seek(80 * TICKS_PER_SECOND);
    assert!(!viewport.follow_playhead(PlaybackScroll::Off));
    assert_eq!(viewport.start(), start);
    viewport.seek(100 * TICKS_PER_SECOND);
    assert!(viewport.follow_playhead(PlaybackScroll::Locked));
    assert_eq!(viewport.start(), 100.0 * TICKS_PER_SECOND as f64 - visible);
    viewport.seek(0);
    viewport.follow_playhead(PlaybackScroll::Locked);
    assert_eq!(viewport.start(), 0.0);
    assert_eq!(viewport.visible_samples(), visible);
    assert_eq!(
        viewport.selection(),
        Some((TICKS_PER_SECOND, 2 * TICKS_PER_SECOND))
    );
}

#[test]
fn minimum_zoom_is_one_tenth_second_in_common_time() {
    let mut viewport = TimelineViewport::new(28_224_000);
    for _ in 0..100 {
        viewport.zoom_at(500.0, 1_000.0, 1.0);
    }
    assert_eq!(viewport.visible_samples(), 2_822_400.0);
}

#[test]
fn click_and_continuous_drag_seek_in_session_time() {
    let mut viewport = TimelineViewport::new(TICKS_PER_SECOND);
    for x in [750.0, 600.0, 425.0, 250.0] {
        viewport.seek(viewport.sample_at(x, 1_000.0));
    }
    assert_eq!(viewport.playhead(), TICKS_PER_SECOND / 4);
}

#[test]
fn wheel_zoom_keeps_the_time_beneath_the_cursor() {
    let mut viewport = TimelineViewport::new(10 * TICKS_PER_SECOND);
    let before = viewport.sample_at(725.0, 1_000.0);
    viewport.zoom_at(725.0, 1_000.0, 1.0);
    let after = viewport.sample_at(725.0, 1_000.0);
    assert!(before.abs_diff(after) <= 1, "{before} != {after}");
}

#[test]
fn horizontal_pan_preserves_zoom_and_clamps_to_session() {
    let mut viewport = TimelineViewport::new(10 * TICKS_PER_SECOND);
    viewport.zoom_at(500.0, 1_000.0, 1.0);
    let visible = viewport.visible_samples();

    viewport.pan_by_pixels(-250.0, 1_000.0);
    assert_eq!(viewport.visible_samples(), visible);
    assert_eq!(viewport.start(), visible / 4.0);

    viewport.pan_by_pixels(f32::MAX, 1_000.0);
    assert_eq!(viewport.start(), 0.0);
    viewport.pan_by_pixels(-f32::MAX, 1_000.0);
    assert_eq!(viewport.start(), 10.0 * TICKS_PER_SECOND as f64 - visible);
}

#[test]
fn ruler_waveforms_playhead_and_selection_share_one_transform() {
    let mut viewport = TimelineViewport::new(10 * TICKS_PER_SECOND);
    viewport.zoom_at(500.0, 1_000.0, 1.0);
    let sample = viewport.sample_at(300.0, 1_000.0);
    viewport.seek(sample);
    viewport.set_selection(Some((sample, sample + TICKS_PER_SECOND)));
    assert!((viewport.x_for_sample(viewport.playhead(), 1_000.0) - 300.0).abs() < 0.01);
    assert_eq!(
        viewport.selection(),
        Some((sample, sample + TICKS_PER_SECOND))
    );
}

#[test]
#[ignore = "responsiveness evidence uses the local canonical reference session"]
fn canonical_session_navigation_stays_responsive() {
    let sources = [
        r"C:\Users\User\Downloads\00005603_A01.wav",
        r"C:\Users\User\Downloads\00005603_A02.wav",
        r"C:\Users\User\Downloads\00005603_A03.wav",
    ];
    let sizes = sources.map(|path| std::fs::metadata(path).unwrap().len());
    assert_eq!(sizes, [sizes[0]; 3]);
    let mut viewport = TimelineViewport::new(
        SourceTimebase::new(48_000, (sizes[0] - 44) / 2)
            .unwrap()
            .duration()
            .unwrap(),
    );
    let started = std::time::Instant::now();
    for step in 0..10_000 {
        let x = (step % 1_280) as f32;
        viewport.seek(viewport.sample_at(x, 1_280.0));
        viewport.zoom_at(x, 1_280.0, if step % 2 == 0 { 1.0 } else { -1.0 });
    }
    assert!(started.elapsed() < std::time::Duration::from_millis(100));
}
