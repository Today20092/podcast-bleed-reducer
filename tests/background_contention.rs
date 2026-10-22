#[path = "common/supervised_fixture.rs"]
#[allow(dead_code)]
mod fixture;

#[test]
fn project_contention_report_observes_active_analysis_without_changing_the_project() {
    let (root, project, _) = fixture::session_at_rates([44_100, 48_000]);
    let saved = std::fs::read(project.path()).unwrap();
    let report_path = root.join("contention.json");
    let args = [
        "--contention".into(),
        project.path().as_os_str().to_owned(),
        report_path.as_os_str().to_owned(),
        root.join("contention-cache").into_os_string(),
    ];
    rust_app::benchmark_editor_project_command(&args).unwrap();
    let bytes = std::fs::read(&report_path).unwrap();
    let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        report["measurement_kind"],
        "headless-editor-background-contention"
    );
    assert_eq!(report["source_count"], 2);
    assert_eq!(report["source_rates"], serde_json::json!([44_100, 48_000]));
    assert_eq!(report["saved_project_unchanged"], true);
    assert_eq!(report["analysis_result_matches_baseline"], true);
    assert!(report["analysis_ms"].as_f64().unwrap() > 0.0);
    for phase in ["foreground_baseline", "foreground_during_analysis"] {
        assert!(report[phase]["observed"].as_u64().unwrap() > 0);
        assert!(report[phase]["retained"].as_u64().unwrap() <= 4096);
        assert!(report[phase]["p99_ms"].as_f64().unwrap().is_finite());
    }
    assert!(report["native_device_underruns"].is_null());
    assert!(report["peak_process_memory_bytes"].is_null());
    assert_eq!(std::fs::read(project.path()).unwrap(), saved);
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(!text.contains(&root.display().to_string()));
    assert!(!text.contains("speaker-0.wav"));
    assert!(rust_app::benchmark_editor_project_command(&args).is_err());
    assert_eq!(std::fs::read(report_path).unwrap(), bytes);
    eprintln!("CONTENTION_FIXTURE_REPORT {report}");
    std::fs::remove_dir_all(root).unwrap();
}
