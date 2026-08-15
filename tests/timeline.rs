use rust_app::timeline::TimelineViewport;

#[test]
fn click_and_continuous_drag_seek_in_session_time() {
    let mut viewport = TimelineViewport::new(48_000);
    for x in [750.0, 600.0, 425.0, 250.0] {
        viewport.seek(viewport.sample_at(x, 1_000.0));
    }
    assert_eq!(viewport.playhead(), 12_000);
}

#[test]
fn wheel_zoom_keeps_the_time_beneath_the_cursor() {
    let mut viewport = TimelineViewport::new(480_000);
    let before = viewport.sample_at(725.0, 1_000.0);
    viewport.zoom_at(725.0, 1_000.0, 1.0);
    let after = viewport.sample_at(725.0, 1_000.0);
    assert!(before.abs_diff(after) <= 1, "{before} != {after}");
}

#[test]
fn ruler_waveforms_playhead_and_selection_share_one_transform() {
    let mut viewport = TimelineViewport::new(480_000);
    viewport.zoom_at(500.0, 1_000.0, 1.0);
    let sample = viewport.sample_at(300.0, 1_000.0);
    viewport.seek(sample);
    viewport.set_selection(Some((sample, sample + 48_000)));
    assert!((viewport.x_for_sample(viewport.playhead(), 1_000.0) - 300.0).abs() < 0.01);
    assert_eq!(viewport.selection(), Some((sample, sample + 48_000)));
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
    let mut viewport = TimelineViewport::new(((sizes[0] - 44) / 2) as usize);
    let started = std::time::Instant::now();
    for step in 0..10_000 {
        let x = (step % 1_280) as f32;
        viewport.seek(viewport.sample_at(x, 1_280.0));
        viewport.zoom_at(x, 1_280.0, if step % 2 == 0 { 1.0 } else { -1.0 });
    }
    assert!(started.elapsed() < std::time::Duration::from_millis(100));
}
