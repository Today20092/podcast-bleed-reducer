#[test]
#[ignore = "creates a local controlled capacity project from eight scripted two-hour WAVs"]
fn prepare_controlled_eight_track_capacity_project() {
    let directory =
        std::path::PathBuf::from(std::env::var("CLEANSLATE_CAPACITY_DIRECTORY").unwrap());
    let paths = (1..=8)
        .map(|track| directory.join(format!("track-{track}.wav")))
        .collect();
    let started = std::time::Instant::now();
    let mut project =
        rust_app::Project::create(directory.join("capacity.cleanslate"), paths).unwrap();
    project.set_supplied_alignment(true).unwrap();
    project
        .add_reference_label(
            rust_app::TimeRange::new(
                601 * rust_app::TICKS_PER_SECOND,
                602 * rust_app::TICKS_PER_SECOND,
            )
            .unwrap(),
            vec![],
            &[],
        )
        .unwrap();
    for index in 0..8 {
        let start = if index == 0 {
            2
        } else if index == 1 {
            22
        } else {
            122 + (index - 2) * 20
        };
        project
            .add_reference_label(
                rust_app::TimeRange::new(
                    start as u64 * rust_app::TICKS_PER_SECOND,
                    (start + 6) as u64 * rust_app::TICKS_PER_SECOND,
                )
                .unwrap(),
                vec![project.sources[index].speaker_id],
                &[],
            )
            .unwrap();
    }
    let receipt = serde_json::json!({"version":1,"purpose":"controlled capacity only; eight dedicated track roles repeat two scripted voices","project_create_and_reference_save_ms":started.elapsed().as_secs_f64()*1000.0,"source_count":project.sources.len(),"project":project.path(),"independent_quality_claim":false});
    std::fs::write(
        directory.join("project-preparation.json"),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    assert_eq!(project.sources.len(), 8);
    assert!(
        project
            .sources
            .iter()
            .all(|s| s.metadata.grid.frame_count == 7200 * 16000)
    );
}
