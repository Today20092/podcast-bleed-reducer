use std::time::Duration;

use rust_app::performance::{Capture, Metric};

#[test]
fn report_names_the_observed_boundaries_and_uses_empirical_percentiles() {
    let mut capture = Capture::new("zoom");
    for ms in [1, 2, 3, 4, 100] {
        capture.record_duration(Metric::AppUpdate, Duration::from_millis(ms));
    }
    let report = serde_json::to_value(capture.report()).unwrap();
    let update = &report["observations"]["application_update_elapsed_ms"];
    assert_eq!(update["observations"], 5);
    assert_eq!(update["median_ms"], 3.0);
    assert_eq!(update["p95_ms"], 100.0);
    assert_eq!(update["p99_ms"], 100.0);
    assert_eq!(report["scenario"], "zoom");
    assert!(
        report["unavailable"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| { v.as_str().unwrap().contains("GPU presentation") })
    );
    assert!(report.get("fps").is_none());
    assert!(report.get("project_path").is_none());
}

#[test]
fn report_retains_only_the_latest_bounded_measurement_window() {
    let mut capture = Capture::new("pan\n\t");
    for ms in 0..4100 {
        capture.record_duration(Metric::WaveformRebuild, Duration::from_millis(ms));
    }
    let report = serde_json::to_value(capture.report()).unwrap();
    let geometry = &report["observations"]["waveform_cache_build_closure_elapsed_ms"];
    assert_eq!(geometry["observations"], 4100);
    assert_eq!(geometry["retained_samples"], 4096);
    assert_eq!(geometry["overwritten_samples"], 4);
    assert_eq!(geometry["median_ms"], 2051.0);
    assert_eq!(geometry["max_ms"], 4099.0);
    assert_eq!(report["scenario"], "pan");
}

#[test]
fn capture_never_replaces_an_existing_file() {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!(
        "cleanslate-perf-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut original = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .unwrap();
    original.write_all(b"existing project bytes").unwrap();
    drop(original);
    let capture = Capture::new("idle");
    assert!(capture.write_report(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"existing project bytes");
    std::fs::remove_file(path).unwrap();
}
