use rust_app::release_gate::{Label, LabelKind, evaluate};

#[test]
fn release_gate_measures_preservation_overlap_and_bleed_independently() {
    let inputs = vec![vec![1.0; 4], vec![0.5; 4]];
    let outputs = vec![vec![1.0, 1.0, 1.0, 0.25], vec![0.5, 0.5, 0.25, 0.125]];
    let labels = vec![
        Label {
            track: 0,
            start: 0,
            end: 3 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 1,
            start: rust_app::TICKS_PER_SECOND / 48_000,
            end: 2 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 1,
            start: 2 * rust_app::TICKS_PER_SECOND / 48_000,
            end: 4 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Bleed,
        },
    ];

    let evidence = evaluate(
        &inputs,
        &outputs,
        &labels,
        &[rust_app::SourceTimebase::new(48_000, 4).unwrap(); 2],
    )
    .unwrap();
    assert_eq!(evidence.preservation_samples, 4);
    assert_eq!(evidence.changed_preservation_samples, 0);
    assert_eq!(evidence.overlap_samples, 1);
    assert_eq!(evidence.changed_overlap_samples, 0);
    assert!(evidence.bleed_output_rms < evidence.bleed_input_rms);
}

#[test]
fn duplicate_labels_on_one_track_are_not_genuine_overlap() {
    let audio = vec![vec![1.0; 2], vec![1.0; 2]];
    let labels = vec![
        Label {
            track: 0,
            start: 0,
            end: 2 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 0,
            start: rust_app::TICKS_PER_SECOND / 48_000,
            end: 2 * rust_app::TICKS_PER_SECOND / 48_000,
            kind: LabelKind::Preserve,
        },
    ];
    assert_eq!(
        evaluate(
            &audio,
            &audio,
            &labels,
            &[rust_app::SourceTimebase::new(48_000, 2).unwrap(); 2]
        )
        .unwrap()
        .overlap_samples,
        0
    );
}

#[test]
fn mixed_native_rates_measure_each_tracks_samples_on_the_shared_physical_timeline() {
    use rust_app::{SourceTimebase, TICKS_PER_SECOND};
    let inputs = vec![vec![1.0; 2], vec![0.5; 4]];
    let outputs = vec![vec![1.0; 2], vec![0.5, 0.5, 0.25, 0.5]];
    let labels = vec![
        Label {
            track: 0,
            start: 0,
            end: TICKS_PER_SECOND,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 1,
            start: TICKS_PER_SECOND / 4,
            end: 3 * TICKS_PER_SECOND / 4,
            kind: LabelKind::Preserve,
        },
    ];
    let evidence = evaluate(
        &inputs,
        &outputs,
        &labels,
        &[
            SourceTimebase::new(2, 2).unwrap(),
            SourceTimebase::new(4, 4).unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(evidence.preservation_samples, 4);
    assert_eq!(evidence.changed_preservation_samples, 1);
    assert_eq!(evidence.overlap_samples, 2);
    assert_eq!(evidence.changed_overlap_samples, 1);
}
