//! Performance measurements reuse the accepted editor and effective-envelope pipeline.
use crate::{EditorAnalysisResult, Project, RenderBlocker};
use std::{sync::atomic::AtomicBool, time::Instant};

#[derive(Debug, serde::Serialize)]
pub struct CancellationTiming {
    pub requested_progress: Option<u8>,
    pub returned_progress: u8,
    pub request_after_start_ms: Option<f64>,
    pub return_after_request_ms: Option<f64>,
    pub job_active_at_request: bool,
    pub cancelled: bool,
}

/// Request the existing public job flag after an observed stage; completion wakes the timer.
pub fn measure_editor_cancellation<T>(
    progress: &std::sync::atomic::AtomicU8,
    after_progress: u8,
    delay: std::time::Duration,
    work: impl FnOnce(&AtomicBool) -> Result<T, RenderBlocker>,
) -> Result<(Result<T, RenderBlocker>, CancellationTiming), RenderBlocker> {
    use std::sync::{Condvar, Mutex, atomic::Ordering};
    progress.store(0, Ordering::Relaxed);
    let cancel = AtomicBool::new(false);
    let state = (Mutex::new((false, None::<(Instant, u8)>)), Condvar::new());
    type TimerState = (Mutex<(bool, Option<(Instant, u8)>)>, Condvar);
    struct Finish<'a>(&'a TimerState);
    impl Drop for Finish<'_> {
        fn drop(&mut self) {
            self.0.0.lock().unwrap_or_else(|e| e.into_inner()).0 = true;
            self.0.1.notify_all();
        }
    }
    let started = Instant::now();
    std::thread::scope(|scope| {
        let timer = scope.spawn(|| {
            let mut reached = None;
            let mut current = state.0.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if current.0 {
                    break;
                }
                let stage = progress.load(Ordering::Relaxed);
                if stage >= after_progress {
                    let eligible = reached.get_or_insert_with(Instant::now);
                    if eligible.elapsed() >= delay {
                        current.1 = Some((Instant::now(), stage));
                        cancel.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                current = state
                    .1
                    .wait_timeout(current, std::time::Duration::from_millis(1))
                    .unwrap_or_else(|e| e.into_inner())
                    .0;
            }
        });
        let finish = Finish(&state);
        let result = work(&cancel);
        let returned = Instant::now();
        drop(finish);
        timer
            .join()
            .map_err(|_| RenderBlocker("Cancellation timer failed.".into()))?;
        let request = state.0.lock().unwrap_or_else(|e| e.into_inner()).1;
        let active = request.is_some_and(|(at, _)| at <= returned);
        let cancelled = active
            && result
                .as_ref()
                .is_err_and(|error| error.to_string().to_lowercase().contains("cancel"));
        Ok((
            result,
            CancellationTiming {
                requested_progress: request.map(|(_, stage)| stage),
                returned_progress: progress.load(Ordering::Relaxed),
                request_after_start_ms: request
                    .map(|(at, _)| at.duration_since(started).as_secs_f64() * 1000.0),
                return_after_request_ms: request
                    .filter(|_| active)
                    .map(|(at, _)| returned.saturating_duration_since(at).as_secs_f64() * 1000.0),
                job_active_at_request: active,
                cancelled,
            },
        ))
    })
}

fn cache_bytes(path: &std::path::Path) -> Result<u64, RenderBlocker> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0_u64;
    let mut directories = vec![path.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                directories.push(entry.path());
            } else if kind.is_file() {
                total = total
                    .checked_add(entry.metadata()?.len())
                    .ok_or_else(|| RenderBlocker("Cache size overflow.".into()))?;
            }
        }
    }
    Ok(total)
}

fn link_prepared_audio_cache(
    from: &std::path::Path,
    to: &std::path::Path,
) -> Result<(), RenderBlocker> {
    std::fs::create_dir(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            link_prepared_audio_cache(&entry.path(), &target)?;
        } else if kind.is_file() {
            std::fs::hard_link(entry.path(), target)?;
        } else {
            return Err(RenderBlocker(
                "Cancellation cache requires regular immutable prepared files.".into(),
            ));
        }
    }
    Ok(())
}

/// Existing preview_benchmark project mode, using the required installed model and editor worker.
pub fn benchmark_editor_project_command(
    args: &[std::ffi::OsString],
) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() != 5 || args[0] != "--project" {
        return Err(
            "--project <iterations> <session.cleanslate> <new-report.json> <cache-directory>"
                .into(),
        );
    }
    let iterations: usize = args[1].to_str().ok_or("Invalid iteration count")?.parse()?;
    if iterations == 0 || iterations > 10000 {
        return Err("Iterations must be 1–10000".into());
    }
    let output = std::path::Path::new(&args[3]);
    if output.exists() {
        return Err("Benchmark output already exists; choose a new path.".into());
    }
    let start = Instant::now();
    let project = Project::open(std::path::PathBuf::from(&args[2]))?;
    let project_load_ms = start.elapsed().as_secs_f64() * 1000.0;
    let saved_project = std::fs::read(project.path())?;
    let cache = std::path::Path::new(&args[4]);
    let cache_before_bytes = cache_bytes(cache)?;
    let cancel = AtomicBool::new(false);
    let helper = std::env::current_exe()?.with_file_name(if cfg!(windows) {
        "model-check.exe"
    } else {
        "model-check"
    });
    let mut verification = std::process::Command::new(helper);
    verification
        .arg("--verify-only")
        .arg("--runtime")
        .arg(crate::model::runtime_directory().map_err(RenderBlocker)?);
    let start = Instant::now();
    let runtime =
        crate::model::verify_native_runtime(verification, &cancel).map_err(RenderBlocker)?;
    let initial_runtime_verification_ms = start.elapsed().as_secs_f64() * 1000.0;
    let mut worker = crate::EditorAnalysisWorker::default();
    let progress = std::sync::atomic::AtomicU8::new(0);
    let start = Instant::now();
    let cold = worker.analyze(&project, cache, Some(&runtime), &progress, &cancel)?;
    let cold_analysis_ms = start.elapsed().as_secs_f64() * 1000.0;
    let cold_stats = worker.stats();
    let gain = benchmark_editor_gain_edits(&project, &cold, iterations, &cancel)?;
    let gain_stats = worker.stats();
    if gain_stats != cold_stats {
        return Err("Gain benchmark unexpectedly reanalyzed.".into());
    }
    let history = &cold.fixed_evidence.history;
    let first = history
        .read_frames(0..history.frames.min(1))
        .map_err(RenderBlocker)?;
    let tail = history
        .read_frames(history.frames.saturating_sub(1)..history.frames)
        .map_err(RenderBlocker)?;
    let start = Instant::now();
    let warm = worker.analyze(&project, cache, Some(&runtime), &progress, &cancel)?;
    let warm_analysis_ms = start.elapsed().as_secs_f64() * 1000.0;
    let warm_stats = worker.stats();
    if warm_stats != cold_stats {
        return Err("Warm identical analysis unexpectedly repeated model work.".into());
    }
    // Reuse immutable PCM only; a fresh native namespace exercises an actual history job.
    let cancellation_cache = cache.join(format!(
        "cancellation-probe-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir(&cancellation_cache)?;
    link_prepared_audio_cache(&cache.join("audio"), &cancellation_cache.join("audio"))?;
    let (cancel_result, inflight_cancellation) =
        measure_editor_cancellation(&progress, 20, std::time::Duration::from_secs(20), |flag| {
            worker.analyze(
                &project,
                &cancellation_cache,
                Some(&runtime),
                &progress,
                flag,
            )
        })?;
    let inflight_retained = worker
        .last_completed()
        .is_some_and(|last| std::sync::Arc::ptr_eq(last, &warm));
    warm.fixed_evidence
        .history
        .verify()
        .map_err(RenderBlocker)?;
    let probe_completed = cancel_result.is_ok();
    let completed_before_request = probe_completed && !inflight_cancellation.job_active_at_request;
    let cancellation_measured = inflight_cancellation.cancelled && inflight_retained;
    let inflight_error = cancel_result.as_ref().err().map(ToString::to_string);
    let inflight_failed = !probe_completed && !cancellation_measured;
    let accepted_after_probe = worker
        .last_completed()
        .cloned()
        .ok_or("Cancellation probe lost accepted analysis.")?;
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    let start = Instant::now();
    let canceled = worker
        .analyze(&project, cache, Some(&runtime), &progress, &cancel)
        .is_err();
    let cancellation_ms = start.elapsed().as_secs_f64() * 1000.0;
    let retained = worker
        .last_completed()
        .is_some_and(|last| std::sync::Arc::ptr_eq(last, &accepted_after_probe));
    if !canceled || !retained {
        return Err("Cancellation did not retain accepted result.".into());
    }
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    project.validate_sources_cancellable(Some(&cancel))?;
    crate::verify_installed_editor_runtime(&runtime, &cancel)?;
    if std::fs::read(project.path())? != saved_project {
        return Err("Project changed during benchmark.".into());
    }
    let report = serde_json::json!({"version":1,"measurement_kind":"actual-required-native-editor-pipeline","quality_claim":"performance only; no independent quality or listening approval","project_load_ms":project_load_ms,"initial_runtime_verification_ms":initial_runtime_verification_ms,"cold_analysis_ms":cold_analysis_ms,"cold_stages_ms":cold.timings,"warm_analysis_ms":warm_analysis_ms,"warm_stages_ms":warm.timings,"gain_edits":gain,"cold_stats":cold_stats,"warm_stats":warm_stats,"fitted_models":cold.ownership.models.as_ref().map_or(0,|m|m.models.len()),"native_history":{"frames":history.frames,"first_probabilities":first,"last_probabilities":tail,"sha256":history.sha256,"compactions":cold.fixed_evidence.compactions,"maximum_retained_frames":cold.fixed_evidence.maximum_retained_frames,"sample_count":cold.fixed_evidence.sample_count},"cache_before_bytes":cache_before_bytes,"cache_after_bytes":cache_bytes(cache)?,"pre_canceled_job_ms":cancellation_ms,"pre_canceled_job_retained_accepted":retained,"sources":project.sources.iter().map(|s|serde_json::json!({"track":s.track_id,"fingerprint":s.fingerprint,"grid":s.metadata.grid})).collect::<Vec<_>>(),"runtime":runtime,"recipe":cold.recipe,"saved_project_unchanged":true,"inflight_cancellation_measured":false,"peak_process_memory_measured":false});
    let mut report = report;
    report["cold_application_cache"] = serde_json::json!(cache_before_bytes == 0);
    report["first_run_cache_state"] = serde_json::json!(if cache_before_bytes == 0 {
        "empty"
    } else {
        "existing"
    });
    report["first_analysis_ms"] = report["cold_analysis_ms"].clone();
    report["first_stages_ms"] = report["cold_stages_ms"].clone();
    if cache_before_bytes != 0 {
        report["cold_analysis_ms"] = serde_json::Value::Null;
        report["cold_stages_ms"] = serde_json::Value::Null;
    }
    report["debug_assertions_enabled"] = serde_json::json!(cfg!(debug_assertions));
    report["os_cache_state"] = serde_json::json!("not flushed; application-cache cold state only");
    report["optimization_claim"] =
        serde_json::json!("see independently recorded binary hash and exact build flags");
    report["inflight_cancellation"] = serde_json::to_value(&inflight_cancellation)?;
    report["inflight_cancellation_measured"] = serde_json::json!(cancellation_measured);
    report["cancellation_cache"] = serde_json::json!(cancellation_cache);
    report["cancellation_cache_state"] =
        serde_json::json!("reused immutable prepared audio; fresh native history namespace");
    report["cancellation_target_phase"] = serde_json::json!(
        "progress 20: audio preparation and reference readiness complete; before runtime verification/native history"
    );
    report["inflight_cancellation_outcome"] = serde_json::json!(if cancellation_measured {
        "cancelled while active; previous accepted analysis retained"
    } else if completed_before_request {
        "completed before cancellation request; cancellation not measured"
    } else if probe_completed {
        "completed despite active cancellation request; cancellation not measured"
    } else {
        "probe failed; cancellation not measured"
    });
    report["inflight_probe_error"] = serde_json::json!(inflight_error);
    report["inflight_cancellation_retained_accepted"] = serde_json::json!(inflight_retained);
    report["accepted_native_history_verified_after_cancel"] = serde_json::json!(true);
    report["cache_size_measurement"] = serde_json::json!(
        "regular-file logical bytes; immutable PCM hard links counted per path; physical disk usage recorded separately"
    );
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?
        .write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
    if inflight_failed {
        return Err("Cancellation probe failed; completed benchmark measurements were retained in the report.".into());
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct GainEditTimings {
    pub initial_composition_ms: f64,
    pub runs_ms: Vec<f64>,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub automatic_builds: u64,
}

/// Exercise transient region gain drafts without saving edits or invoking an analysis worker.
pub fn benchmark_editor_gain_edits(
    project: &Project,
    analysis: &EditorAnalysisResult,
    iterations: usize,
    cancel: &AtomicBool,
) -> Result<GainEditTimings, RenderBlocker> {
    if iterations == 0 || iterations > 10000 || !analysis.snapshot.matches(project) {
        return Err(RenderBlocker(
            "Benchmark requires current analysis and 1–10000 iterations.".into(),
        ));
    }
    crate::prepared_analysis::check_cancelled(Some(cancel))?;
    let mut cache = crate::EffectiveEnvelopeCache::default();
    let settings = crate::project_fade_settings(project)?;
    let start = Instant::now();
    cache.update(project, &analysis.temporal, settings, Some(cancel))?;
    let initial_composition_ms = start.elapsed().as_secs_f64() * 1000.0;
    let source = project
        .sources
        .first()
        .ok_or_else(|| RenderBlocker("No benchmark sources.".into()))?;
    let end = source
        .metadata
        .grid
        .duration()?
        .min(crate::TICKS_PER_SECOND);
    let range = crate::TimeRange::new(0, end)?;
    let mut runs_ms = Vec::with_capacity(iterations);
    for iteration in 0..iterations {
        crate::prepared_analysis::check_cancelled(Some(cancel))?;
        let start = Instant::now();
        let draft = project.region_gain_edit(
            source.track_id,
            range,
            crate::GainRegionAction::Attenuate(if iteration % 2 == 0 { 0.25 } else { 0.5 }),
        )?;
        let preview =
            cache.preview_edit(project, &analysis.temporal, settings, &draft, Some(cancel))?;
        // Evaluate the same source-time curve used by drawing, playback and delivery.
        let _ = preview.envelopes[0].gain_at_time(end / 2);
        runs_ms.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let mut sorted = runs_ms.clone();
    sorted.sort_by(f64::total_cmp);
    Ok(GainEditTimings {
        initial_composition_ms,
        median_ms: sorted[sorted.len() / 2],
        p95_ms: sorted[(sorted.len() * 95).div_ceil(100) - 1],
        runs_ms,
        automatic_builds: cache.automatic_builds(),
    })
}
