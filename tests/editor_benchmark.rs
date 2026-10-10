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

#[test]
#[ignore = "measures acoustic trained two-hour components using controlled track sources"]
fn measure_controlled_trained_eight_track_components() {
    use std::sync::atomic::{AtomicBool, AtomicU8};
    let directory =
        std::path::PathBuf::from(std::env::var("CLEANSLATE_CAPACITY_DIRECTORY").unwrap());
    let project = rust_app::Project::open(directory.join("capacity.cleanslate")).unwrap();
    let saved = std::fs::read(project.path()).unwrap();
    let output = directory.join("trained-components.json");
    assert!(!output.exists());
    let cancel = AtomicBool::new(false);
    let audio: Vec<_> = project
        .source_paths()
        .iter()
        .map(|path| {
            rust_app::prepare_analysis_audio_in(
                path,
                &directory.join("cache-optimized-v1/audio"),
                Some(&cancel),
            )
            .unwrap()
        })
        .collect();
    let progress = AtomicU8::new(0);
    let mut worker = rust_app::EditorAnalysisWorker::default();
    let started = std::time::Instant::now();
    let cold = worker
        .analyze_acoustic_evidence(&project, &audio, &progress, &cancel)
        .unwrap();
    let cold_ms = started.elapsed().as_secs_f64() * 1000.0;
    let cold_stats = worker.stats();
    assert_eq!(cold.ownership.models.as_ref().unwrap().models.len(), 8);
    assert_eq!(cold_stats.model_fits, 8);
    assert_eq!(cold_stats.probability_passes, 1);
    let gain = rust_app::benchmark_editor_gain_edits(&project, &cold, 10, &cancel).unwrap();
    assert_eq!(worker.stats(), cold_stats);
    let started = std::time::Instant::now();
    let warm = worker
        .analyze_acoustic_evidence(&project, &audio, &progress, &cancel)
        .unwrap();
    let warm_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(worker.stats(), cold_stats);
    assert!(std::sync::Arc::ptr_eq(&warm.ownership, &cold.ownership));
    assert_eq!(std::fs::read(project.path()).unwrap(), saved);
    let report = serde_json::json!({"version":1,"measurement_kind":"controlled-acoustic-trained-components","quality_claim":"eight dedicated track roles repeat two scripted voices; NOT eight-speaker quality","cold_ms":cold_ms,"warm_ms":warm_ms,"cold_stages":cold.timings,"warm_stages":warm.timings,"cold_stats":cold_stats,"warm_stats":worker.stats(),"fitted_models":8,"gain_edits":gain,"sources":audio.iter().map(|a|&a.identity).collect::<Vec<_>>(),"saved_project_unchanged":true});
    use std::io::Write;
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&report).unwrap())
        .unwrap();
}
