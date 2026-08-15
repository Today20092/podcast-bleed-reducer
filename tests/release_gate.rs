use rust_app::release_gate::{Label, LabelKind, evaluate};

#[test]
fn release_gate_measures_preservation_overlap_and_bleed_independently() {
    let inputs = vec![vec![1.0; 4], vec![0.5; 4]];
    let outputs = vec![vec![1.0, 1.0, 1.0, 0.25], vec![0.5, 0.5, 0.25, 0.125]];
    let labels = vec![
        Label {
            track: 0,
            start: 0,
            end: 3,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 1,
            start: 1,
            end: 2,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 1,
            start: 2,
            end: 4,
            kind: LabelKind::Bleed,
        },
    ];

    let evidence = evaluate(&inputs, &outputs, &labels);
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
            end: 2,
            kind: LabelKind::Preserve,
        },
        Label {
            track: 0,
            start: 1,
            end: 2,
            kind: LabelKind::Preserve,
        },
    ];
    assert_eq!(evaluate(&audio, &audio, &labels).overlap_samples, 0);
}
