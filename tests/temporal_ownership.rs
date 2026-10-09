use rust_app::{
    OWNERSHIP_FRAME_TICKS, TemporalEvidence, TemporalEvidenceGrid, TemporalOwnershipCache,
    TemporalSettings, TemporalTrackState, TrackId,
};

fn row(owner: f32, activity: f32, raw: f32) -> TemporalEvidence {
    TemporalEvidence {
        probability: Some(owner),
        raw_speech: Some(raw),
        assigned_activity: Some(activity),
        preservation_reason: None,
    }
}

#[test]
fn brief_overlap_survives_and_contradictions_never_establish_silence() {
    let rows = vec![
        vec![row(0.01, 0.01, 0.01); 8],
        vec![row(0.01, 0.01, 0.01); 8],
    ];
    let mut rows = rows;
    rows[0][3] = row(0.95, 0.95, 0.95);
    rows[1][3] = row(0.91, 0.91, 0.8);
    rows[0][5] = row(0.01, 0.9, 0.8);
    rows[1][6] = row(0.95, 0.01, 0.01);
    let grid = TemporalEvidenceGrid::new(
        vec![TrackId(1), TrackId(2)],
        OWNERSHIP_FRAME_TICKS * 8,
        rows,
        vec![0.01, 0.01, 0.01, 0.95, 0.01, 0.9, 0.01, 0.01],
    )
    .unwrap();
    let mut cache = TemporalOwnershipCache::default();
    let result = cache
        .optimize_evidence(&grid, TemporalSettings::default(), None)
        .unwrap();
    assert_eq!(
        result.states_at(OWNERSHIP_FRAME_TICKS * 3).unwrap(),
        vec![TemporalTrackState::Owned, TemporalTrackState::Owned]
    );
    assert_eq!(
        result.states_at(OWNERSHIP_FRAME_TICKS * 5).unwrap()[0],
        TemporalTrackState::UncertainUnity
    );
    assert_eq!(
        result.states_at(OWNERSHIP_FRAME_TICKS * 6).unwrap()[1],
        TemporalTrackState::UncertainUnity
    );
    assert_eq!(
        result.states_at(0).unwrap(),
        vec![TemporalTrackState::Absent; 2]
    );
    assert!(result.states_at(grid.duration()).is_err());
    assert!(!result.warnings.is_empty());
    assert_eq!(
        result
            .evidence()
            .evidence_at(OWNERSHIP_FRAME_TICKS * 5)
            .unwrap()[0]
            .probability,
        Some(0.01)
    );
    assert!(matches!(
        result.engine_regions()[5].decision,
        rust_app::Decision::Uncertain
    ));
    let mut review = rust_app::ReviewWorkspace::from_ownership(&result);
    review.select_warning(0);
    assert_eq!(review.playhead(), review.warnings[0].start);
    assert!(review.warnings[0].key().contains(&result.cache_key));
}

#[test]
fn gaps_can_be_preserved_but_long_silence_is_not_filled_and_cache_is_atomic() {
    use std::sync::{Arc, atomic::AtomicBool};
    let mut rows = vec![row(0.01, 0.01, 0.01); 20];
    rows[2] = row(0.9, 0.9, 0.9);
    rows[5] = row(0.9, 0.9, 0.9);
    rows[18] = row(0.9, 0.9, 0.9);
    let grid = TemporalEvidenceGrid::new(
        vec![TrackId(1)],
        OWNERSHIP_FRAME_TICKS * 20,
        vec![rows],
        vec![0.01; 20],
    )
    .unwrap();
    let mut cache = TemporalOwnershipCache::default();
    let accepted = cache
        .optimize_evidence(&grid, TemporalSettings::default(), None)
        .unwrap();
    assert_eq!(
        accepted.states_at(OWNERSHIP_FRAME_TICKS * 3).unwrap(),
        vec![TemporalTrackState::UncertainUnity]
    );
    assert_eq!(
        accepted.states_at(OWNERSHIP_FRAME_TICKS * 10).unwrap(),
        vec![TemporalTrackState::Absent]
    );
    let same = cache
        .optimize_evidence(&grid, TemporalSettings::default(), None)
        .unwrap();
    assert!(Arc::ptr_eq(&same, &accepted));
    assert!(
        cache
            .optimize_evidence(
                &grid,
                TemporalSettings::default(),
                Some(&AtomicBool::new(true))
            )
            .is_err()
    );
    assert!(Arc::ptr_eq(cache.last_completed().unwrap(), &accepted));
    let changed = cache
        .optimize_evidence(
            &grid,
            TemporalSettings {
                preserved_gap_frames: 0,
                ..TemporalSettings::default()
            },
            None,
        )
        .unwrap();
    assert_ne!(changed.cache_key, accepted.cache_key);
    assert_eq!(
        changed.states_at(OWNERSHIP_FRAME_TICKS * 3).unwrap(),
        vec![TemporalTrackState::Absent]
    );
    assert!(
        TemporalEvidenceGrid::new(
            vec![TrackId(1)],
            1,
            vec![vec![row(f32::NAN, 0.1, 0.1)]],
            vec![0.1]
        )
        .is_err()
    );
}

#[test]
fn human_constraints_are_exact_and_all_labels_apply_without_gain_or_alignment_leaks() {
    use rust_app::{AutomixParameters, Project, TimeRange};
    use std::sync::Arc;
    let root = std::env::temp_dir().join(format!(
        "temporal-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut paths = vec![];
    for track in 0..2 {
        let path = root.join(format!("{track}.wav"));
        let mut writer =
            bwavfile::WaveWriter::create(&path, bwavfile::WaveFmt::new_pcm_mono(16000, 16))
                .unwrap()
                .audio_frame_writer()
                .unwrap();
        writer.write_frames(&vec![0_i16; 512 * 4096]).unwrap();
        writer.end().unwrap();
        paths.push(path);
    }
    let mut project = Project::create(root.join("project.cleanslate"), paths).unwrap();
    project.set_supplied_alignment(true).unwrap();
    let tracks: Vec<_> = project.sources.iter().map(|s| s.track_id).collect();
    let speakers: Vec<_> = project.sources.iter().map(|s| s.speaker_id).collect();
    let rows = vec![
        (0..4096)
            .map(|i| if i % 2 == 0 {
                row(0.5, 0.5, 0.5)
            } else {
                row(0.01, 0.01, 0.01)
            })
            .collect();
        2
    ];
    let grid =
        TemporalEvidenceGrid::new(tracks, OWNERSHIP_FRAME_TICKS * 4096, rows, vec![0.01; 4096])
            .unwrap();
    let mut cache = TemporalOwnershipCache::default();
    let start = OWNERSHIP_FRAME_TICKS * 3 + 1;
    let end = start + rust_app::TICKS_PER_SECOND / 4;
    project
        .add_reference_label(TimeRange::new(start, end).unwrap(), speakers, &[])
        .unwrap();
    let result = cache
        .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
        .unwrap();
    assert_eq!(
        result.states_at(start).unwrap(),
        vec![TemporalTrackState::Owned; 2]
    );
    assert_eq!(
        result.states_at(end - 1).unwrap(),
        vec![TemporalTrackState::Owned; 2]
    );
    assert_eq!(
        result.states_at(start - 1).unwrap(),
        vec![TemporalTrackState::Absent; 2]
    );
    assert_eq!(
        result.states_at(end).unwrap(),
        vec![TemporalTrackState::UncertainUnity; 2]
    );
    assert!(result.warnings.len() > 4080 && result.warnings.len() <= 4096);
    assert!(
        result
            .warnings
            .iter()
            .all(|warning| warning.end <= start || warning.start >= end)
    );
    let original_evidence = result.evidence().duration();
    project
        .set_parameters(AutomixParameters {
            attenuation_db: -18.0,
            ..AutomixParameters::default()
        })
        .unwrap();
    let same = cache
        .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
        .unwrap();
    assert!(Arc::ptr_eq(&same, &result));
    assert_eq!(same.evidence().duration(), original_evidence);
    project
        .add_reference_label(
            TimeRange::new(
                OWNERSHIP_FRAME_TICKS * 30 + 1,
                OWNERSHIP_FRAME_TICKS * 30 + 1 + rust_app::TICKS_PER_SECOND / 4,
            )
            .unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    let changed = cache
        .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
        .unwrap();
    assert_ne!(changed.cache_key, result.cache_key);
    assert_eq!(
        changed.states_at(OWNERSHIP_FRAME_TICKS * 30 + 1).unwrap(),
        vec![TemporalTrackState::Absent; 2]
    );
    assert_eq!(
        changed.states_at(OWNERSHIP_FRAME_TICKS * 30).unwrap(),
        vec![TemporalTrackState::UncertainUnity; 2]
    );
    project.set_supplied_alignment(false).unwrap();
    assert!(
        cache
            .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
            .is_err()
    );
    assert!(Arc::ptr_eq(cache.last_completed().unwrap(), &changed));
    project.set_supplied_alignment(true).unwrap();
    let path = &project.sources[0].path;
    let mut bytes = std::fs::read(path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    std::fs::write(path, bytes).unwrap();
    assert!(
        cache
            .optimize_project_evidence(&project, &grid, TemporalSettings::default(), None)
            .is_err()
    );
    assert!(Arc::ptr_eq(cache.last_completed().unwrap(), &changed));
    std::fs::remove_dir_all(root).unwrap();
}
