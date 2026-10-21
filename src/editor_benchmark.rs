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
    if args.first().is_some_and(|arg| arg == "--contention") {
        return benchmark_editor_contention(args);
    }
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
    let mut worker = crate::EditorAnalysisWorker::default();
    let progress = std::sync::atomic::AtomicU8::new(0);
    let start = Instant::now();
    let cold = worker.analyze(&project, cache, &progress, &cancel)?;
    let cold_analysis_ms = start.elapsed().as_secs_f64() * 1000.0;
    let cold_stats = worker.stats();
    let gain = benchmark_editor_gain_edits(&project, &cold, iterations, &cancel)?;
    let gain_stats = worker.stats();
    if gain_stats != cold_stats {
        return Err("Gain benchmark unexpectedly reanalyzed.".into());
    }
    let start = Instant::now();
    let warm = worker.analyze(&project, cache, &progress, &cancel)?;
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
        measure_editor_cancellation(&progress, 55, std::time::Duration::from_secs(20), |flag| {
            worker.analyze(&project, &cancellation_cache, &progress, flag)
        })?;
    let inflight_retained = worker
        .last_completed()
        .is_some_and(|last| std::sync::Arc::ptr_eq(last, &warm));
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
    let canceled = worker.analyze(&project, cache, &progress, &cancel).is_err();
    let cancellation_ms = start.elapsed().as_secs_f64() * 1000.0;
    let retained = worker
        .last_completed()
        .is_some_and(|last| std::sync::Arc::ptr_eq(last, &accepted_after_probe));
    if !canceled || !retained {
        return Err("Cancellation did not retain accepted result.".into());
    }
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    project.validate_sources_cancellable(Some(&cancel))?;
    if std::fs::read(project.path())? != saved_project {
        return Err("Project changed during benchmark.".into());
    }
    let report = serde_json::json!({"version":1,"measurement_kind":"actual-acoustic-editor-pipeline","quality_claim":"performance only; no independent quality or listening approval","project_load_ms":project_load_ms,"cold_analysis_ms":cold_analysis_ms,"cold_stages_ms":cold.timings,"warm_analysis_ms":warm_analysis_ms,"warm_stages_ms":warm.timings,"gain_edits":gain,"cold_stats":cold_stats,"warm_stats":warm_stats,"fitted_models":cold.ownership.models.as_ref().map_or(0,|m|m.models.len()),"cache_before_bytes":cache_before_bytes,"cache_after_bytes":cache_bytes(cache)?,"pre_canceled_job_ms":cancellation_ms,"pre_canceled_job_retained_accepted":retained,"sources":project.sources.iter().map(|s|serde_json::json!({"track":s.track_id,"fingerprint":s.fingerprint,"grid":s.metadata.grid})).collect::<Vec<_>>(),"recipe":cold.recipe,"saved_project_unchanged":true,"inflight_cancellation_measured":false,"peak_process_memory_measured":false});
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
        serde_json::json!("reused immutable prepared audio; fresh prepared-audio namespace");
    report["cancellation_target_phase"] = serde_json::json!(
        "progress 55: audio preparation and reference readiness complete; acoustic ownership starts"
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

fn contention_summary(mut samples: Vec<f64>, observed: u64) -> serde_json::Value {
    samples.sort_by(f64::total_cmp);
    let percentile = |percent: usize| samples[(samples.len() * percent).div_ceil(100) - 1];
    serde_json::json!({
        "observed": observed, "retained": samples.len(),
        "overwritten": observed - samples.len() as u64,
        "median_ms": percentile(50), "p95_ms": percentile(95),
        "p99_ms": percentile(99), "max_ms": samples.last(),
    })
}

/// Opt-in headless comparison; the editor's worker, gain drafts and playback consumer are real.
fn benchmark_editor_contention(
    args: &[std::ffi::OsString],
) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::time::Duration;
    if args.len() != 4 {
        return Err(
            "--contention <session.cleanslate> <new-report.json> <new-cache-directory>".into(),
        );
    }
    let output = std::path::Path::new(&args[2]);
    if output.exists() {
        return Err("Benchmark output already exists; choose a new path.".into());
    }
    let project = Project::open(std::path::PathBuf::from(&args[1]))?;
    let saved = std::fs::read(project.path())?;
    let cache = std::path::Path::new(&args[3]);
    std::fs::create_dir(cache)?;
    let cancel = AtomicBool::new(false);
    let progress = AtomicU8::new(0);
    let start = Instant::now();
    let baseline = crate::EditorAnalysisWorker::default().analyze(
        &project,
        &cache.join("standalone"),
        &progress,
        &cancel,
    )?;
    let standalone_analysis_ms = start.elapsed().as_secs_f64() * 1000.0;
    let settings = crate::project_fade_settings(&project)?;
    let mut envelopes = crate::EffectiveEnvelopeCache::default();
    let accepted = envelopes.update(&project, &baseline.temporal, settings, Some(&cancel))?;
    let prepared = crate::PreparedAudio::prepare_sources_for_rate(
        &project.audio_sources(),
        None,
        48_000,
        &cancel,
    )?;
    let mut playback = prepared.transport(
        accepted.envelopes.clone(),
        crate::AuditionMode::Processed,
        1.0 / project.sources.len() as f32,
        crate::TimeRange::new(0, prepared.duration())?,
    )?;
    playback.set_loop(true);
    let range = crate::TimeRange::new(0, prepared.duration().min(crate::TICKS_PER_SECOND))?;
    let mut phases = Vec::new();
    let mut concurrent = None;
    for active in [false, true] {
        playback.seek(0);
        std::thread::sleep(Duration::from_millis(20));
        let phase = std::thread::scope(|scope| -> Result<_, RenderBlocker> {
            let job = active.then(|| {
                scope.spawn(|| {
                    let start = Instant::now();
                    let result = crate::EditorAnalysisWorker::default().analyze(
                        &project,
                        &cache.join("concurrent"),
                        &progress,
                        &cancel,
                    );
                    (result, start.elapsed().as_secs_f64() * 1000.0)
                })
            });
            let start = Instant::now();
            let mut due = start;
            let mut samples = Vec::with_capacity(4096);
            let mut work_samples = Vec::with_capacity(4096);
            let mut observed = 0_u64;
            let mut starved = 0_u64;
            let foreground = (|| -> Result<(), RenderBlocker> {
                while job.as_ref().map_or_else(
                    || start.elapsed() < Duration::from_secs(2),
                    |job| !job.is_finished(),
                ) {
                    let work = Instant::now();
                    let draft = project.region_gain_edit(
                        project.sources[0].track_id,
                        range,
                        crate::GainRegionAction::Attenuate(if observed.is_multiple_of(2) {
                            0.25
                        } else {
                            0.5
                        }),
                    )?;
                    let preview = envelopes.preview_edit(
                        &project,
                        &baseline.temporal,
                        settings,
                        &draft,
                        Some(&cancel),
                    )?;
                    std::hint::black_box(preview.envelopes[0].gain_at_time(range.end / 2));
                    playback.replace_envelopes(preview.envelopes.clone());
                    // Consume 20 ms at 48 kHz; stalled position means software starvation, not a device underrun.
                    for _ in 0..960 {
                        let before = playback.position_frame();
                        let sample = playback
                            .next()
                            .ok_or_else(|| RenderBlocker("Benchmark playback ended.".into()))?;
                        std::hint::black_box(sample);
                        starved += u64::from(playback.position_frame() == before);
                    }
                    if let Some(error) = playback.error() {
                        return Err(RenderBlocker(error.to_string()));
                    }
                    let response = due.elapsed().as_secs_f64() * 1000.0;
                    let elapsed = work.elapsed().as_secs_f64() * 1000.0;
                    let index = observed as usize % 4096;
                    if samples.len() < 4096 {
                        samples.push(response);
                        work_samples.push(elapsed);
                    } else {
                        samples[index] = response;
                        work_samples[index] = elapsed;
                    }
                    observed += 1;
                    // Do not catch up with unbounded work after a missed cadence.
                    due = Instant::now() + Duration::from_millis(20);
                    std::thread::sleep(due.saturating_duration_since(Instant::now()));
                }
                if observed == 0 {
                    return Err(RenderBlocker("Analysis completed before any concurrent observation; repeat with a larger project.".into()));
                }
                Ok(())
            })();
            if foreground.is_err() {
                cancel.store(true, Ordering::Relaxed);
            }
            if let Some(job) = job {
                let (result, milliseconds) = job
                    .join()
                    .map_err(|_| RenderBlocker("Benchmark analysis thread failed.".into()))?;
                concurrent = Some((result?, milliseconds));
            }
            foreground?;
            let mut summary = contention_summary(samples, observed);
            summary["work_elapsed"] = contention_summary(work_samples, observed);
            summary["software_starved_samples"] = serde_json::json!(starved);
            summary["elapsed_ms"] = serde_json::json!(start.elapsed().as_secs_f64() * 1000.0);
            Ok(summary)
        })?;
        phases.push(phase);
    }
    let (analysis, analysis_ms) = concurrent.ok_or("Concurrent analysis was not measured.")?;
    let effective = envelopes.update(&project, &analysis.temporal, settings, Some(&cancel))?;
    if accepted.envelopes != effective.envelopes
        || baseline.temporal.regions() != analysis.temporal.regions()
    {
        return Err(
            "Concurrent analysis changed authoritative regions or effective envelopes.".into(),
        );
    }
    project.validate_sources_cancellable(Some(&cancel))?;
    if std::fs::read(project.path())? != saved {
        return Err("Project changed during benchmark.".into());
    }
    let track_seconds: f64 = project
        .sources
        .iter()
        .map(|s| s.metadata.grid.frame_count as f64 / f64::from(s.metadata.grid.sample_rate))
        .sum();
    let report = serde_json::json!({
        "version": 1, "measurement_kind": "headless-editor-background-contention",
        "application_version": env!("CARGO_PKG_VERSION"), "debug_assertions_enabled": cfg!(debug_assertions),
        "source_count": project.sources.len(), "source_rates": project.sources.iter().map(|s| s.metadata.grid.sample_rate).collect::<Vec<_>>(),
        "track_seconds": track_seconds, "standalone_analysis_ms": standalone_analysis_ms, "analysis_ms": analysis_ms,
        "standalone_stages_ms": baseline.timings, "concurrent_stages_ms": analysis.timings,
        "standalone_track_seconds_per_second": track_seconds * 1000.0 / standalone_analysis_ms,
        "concurrent_track_seconds_per_second": track_seconds * 1000.0 / analysis_ms,
        "foreground_baseline": phases[0], "foreground_during_analysis": phases[1],
        "foreground_boundary": "transient gain draft, shared effective envelope, playback delivery and 960 software output samples; completion includes wake lateness after a 20 ms rest; no catch-up",
        "sample_limit": 4096, "percentiles": "nearest rank; most recent retained observations",
        "cache_state": "separate initially empty application caches; OS and shared playback caches are not flushed; standalone analysis runs first",
        "analysis_result_matches_baseline": true, "saved_project_unchanged": true,
        "native_device_underruns": null, "peak_process_memory_bytes": null,
        "limitations": "headless component contention, not application frames, native callback deadlines, physical latency, listening or independent quality; use native capture and OS sampler alongside",
    });
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    serde_json::to_writer_pretty(file, &report)?;
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
