use rust_app::{Rounding, SourceTimebase, TICKS_PER_SECOND, TimeRange};

#[test]
fn native_sources_share_physical_time_without_losing_sample_boundaries() {
    for rate in [44_100, 48_000, 96_000] {
        let grid = SourceTimebase::new(rate, u64::from(rate) * 7_200).unwrap();
        assert_eq!(grid.time_at_frame(0, Rounding::Floor).unwrap(), 0);
        let first = grid.time_at_frame(1, Rounding::Floor).unwrap();
        assert_eq!(grid.frame_at_time(first, Rounding::Floor).unwrap(), 1);
        let last = grid.frame_count - 1;
        let last_time = grid.time_at_frame(last, Rounding::Floor).unwrap();
        assert_eq!(grid.frame_at_time(last_time, Rounding::Ceil).unwrap(), last);
        assert_eq!(grid.duration().unwrap(), 7_200 * TICKS_PER_SECOND);
        assert_eq!(
            grid.time_at_frame(u64::from(rate) * 90, Rounding::Floor)
                .unwrap(),
            90 * TICKS_PER_SECOND
        );
    }
}

#[test]
fn preserved_edges_round_outward_on_arbitrary_native_grids() {
    let grid = SourceTimebase::new(12_345, 12_345).unwrap();
    let start = grid.time_at_frame(1, Rounding::Floor).unwrap();
    let end = grid.time_at_frame(2, Rounding::Ceil).unwrap();
    assert_eq!(
        TimeRange::new(start, end)
            .unwrap()
            .native_frames(grid)
            .unwrap(),
        0..3
    );
    let grid = SourceTimebase::new(96_000, 96_000).unwrap();
    assert_eq!(
        TimeRange::new(295, 587)
            .unwrap()
            .native_frames(grid)
            .unwrap(),
        1..2
    );
}

#[test]
fn invalid_grids_bounds_and_unrepresentable_time_fail_safely() {
    assert!(SourceTimebase::new(0, 1).is_err());
    assert!(SourceTimebase::new(1, u64::MAX).is_err());
    let grid = SourceTimebase::new(48_000, 48_000).unwrap();
    assert!(grid.time_at_frame(48_001, Rounding::Floor).is_err());
    assert!(
        grid.frame_at_time(TICKS_PER_SECOND + 1, Rounding::Floor)
            .is_err()
    );
    assert!(TimeRange::new(10, 10).is_err());
    for value in [f64::NAN, f64::INFINITY, -1.0, f64::MAX] {
        assert!(rust_app::time_from_seconds(value, Rounding::Floor).is_err());
    }
}
