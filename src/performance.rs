//! Bounded opt-in observations of the editor, not GPU presentation or physical latency.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;

const SAMPLE_LIMIT: usize = 4096;
static SESSION: OnceLock<Option<Session>> = OnceLock::new();

#[derive(Clone, Copy)]
pub enum Metric {
    AppUpdate,
    WorkerPoll,
    WaveformDraw,
    GainDraw,
    WaveformRebuild,
    GainRebuild,
    RedrawInterval,
    InputToCanvas,
}

impl Metric {
    fn name(self) -> &'static str {
        match self {
            Self::AppUpdate => "application_update_elapsed_ms",
            Self::WorkerPoll => "worker_poll_update_elapsed_ms",
            Self::WaveformDraw => "waveform_canvas_draw_elapsed_ms",
            Self::GainDraw => "gain_canvas_draw_elapsed_ms",
            Self::WaveformRebuild => "waveform_cache_build_closure_elapsed_ms",
            Self::GainRebuild => "gain_cache_build_closure_elapsed_ms",
            Self::RedrawInterval => "redraw_request_interval_ms",
            Self::InputToCanvas => "observed_input_to_first_canvas_completion_ms",
        }
    }
}

#[derive(Default)]
struct Samples {
    values: Vec<f64>,
    observations: u64,
}

#[derive(Serialize)]
pub struct Summary {
    observations: u64,
    retained_samples: usize,
    overwritten_samples: u64,
    median_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    max_ms: f64,
}

impl Samples {
    fn push(&mut self, value: f64) {
        if self.values.len() < SAMPLE_LIMIT {
            self.values.push(value);
        } else {
            self.values[self.observations as usize % SAMPLE_LIMIT] = value;
        }
        self.observations += 1;
    }

    fn summary(&self) -> Summary {
        let mut sorted = self.values.clone();
        sorted.sort_by(f64::total_cmp);
        let percentile =
            |fraction: f64| sorted[(fraction * sorted.len() as f64).ceil() as usize - 1];
        Summary {
            observations: self.observations,
            retained_samples: sorted.len(),
            overwritten_samples: self.observations - sorted.len() as u64,
            median_ms: percentile(0.5),
            p95_ms: percentile(0.95),
            p99_ms: percentile(0.99),
            max_ms: sorted[sorted.len() - 1],
        }
    }
}

/// One scenario's observations. Percentiles use the most recent 4096 samples per metric.
pub struct Capture {
    scenario: String,
    started: Instant,
    samples: BTreeMap<&'static str, Samples>,
    pending_input: Option<Instant>,
    last_redraw: Option<Instant>,
    input_events: u64,
    resize_events: u64,
    window_logical_size: Option<[f32; 2]>,
    window_scale_factor: Option<f32>,
}

#[derive(Serialize)]
pub struct Report {
    schema_version: u8,
    scenario: String,
    application_version: &'static str,
    build_profile: &'static str,
    platform: String,
    source_commit: Option<String>,
    executable_sha256: Option<String>,
    identity_origin: &'static str,
    elapsed_ms: f64,
    sample_limit_per_metric: usize,
    input_events: u64,
    resize_events: u64,
    window_logical_size: Option<[f32; 2]>,
    window_scale_factor: Option<f32>,
    observations: BTreeMap<&'static str, Summary>,
    unavailable: [&'static str; 5],
}

fn bounded_text(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(160)
        .collect()
}

impl Capture {
    pub fn new(scenario: &str) -> Self {
        Self {
            scenario: bounded_text(scenario),
            started: Instant::now(),
            samples: BTreeMap::new(),
            pending_input: None,
            last_redraw: None,
            input_events: 0,
            resize_events: 0,
            window_logical_size: None,
            window_scale_factor: None,
        }
    }

    pub fn record_duration(&mut self, metric: Metric, elapsed: Duration) {
        self.samples
            .entry(metric.name())
            .or_default()
            .push(elapsed.as_secs_f64() * 1000.0);
    }

    pub fn report(&self) -> Report {
        Report {
            schema_version: 1,
            scenario: self.scenario.clone(),
            application_version: env!("CARGO_PKG_VERSION"),
            build_profile: if cfg!(debug_assertions) {
                "development"
            } else {
                "release"
            },
            platform: format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH),
            source_commit: std::env::var("CLEANSLATE_PERF_SOURCE_COMMIT")
                .ok()
                .map(|s| bounded_text(&s)),
            executable_sha256: std::env::var("CLEANSLATE_PERF_EXECUTABLE_SHA256")
                .ok()
                .map(|s| bounded_text(&s)),
            identity_origin: "launch metadata; executable hash must be supplied by the launcher",
            elapsed_ms: self.started.elapsed().as_secs_f64() * 1000.0,
            sample_limit_per_metric: SAMPLE_LIMIT,
            input_events: self.input_events,
            resize_events: self.resize_events,
            window_logical_size: self.window_logical_size,
            window_scale_factor: self.window_scale_factor,
            observations: self
                .samples
                .iter()
                .map(|(name, samples)| (*name, samples.summary()))
                .collect(),
            unavailable: [
                "GPU presentation completion, busy time and monitor refresh rate",
                "physical input-to-photon latency and acoustic onset",
                "process CPU and working set: collect with a separately labeled OS sampler",
                "audio callback deadlines: use the separate playback probes",
                "graphics adapter/backend: use About diagnostics when available",
            ],
        }
    }

    /// Refuses to replace an existing file, including a project or source file.
    pub fn write_report(&self, path: &Path) -> Result<(), String> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("Could not create performance report: {e}"))?;
        serde_json::to_writer_pretty(file, &self.report())
            .map_err(|e| format!("Could not write performance report: {e}"))
    }
}

struct Session {
    capture: Mutex<Capture>,
    path: PathBuf,
}

pub(crate) fn start_from_environment() {
    SESSION.get_or_init(|| {
        let path = PathBuf::from(std::env::var_os("CLEANSLATE_PERF_REPORT")?);
        if !path.is_absolute() || path.exists() || !path.parent().is_some_and(Path::is_dir) {
            eprintln!("Performance capture disabled: use a new absolute report path in an existing directory.");
            return None;
        }
        let scenario = std::env::var("CLEANSLATE_PERF_SCENARIO").unwrap_or_else(|_| "unspecified".into());
        Some(Session { capture: Mutex::new(Capture::new(&scenario)), path })
    });
}

pub(crate) fn is_active() -> bool {
    SESSION.get().is_some_and(Option::is_some)
}

fn with_capture(f: impl FnOnce(&mut Capture)) {
    if let Some(Some(session)) = SESSION.get()
        && let Ok(mut capture) = session.capture.lock()
    {
        f(&mut capture);
    }
}

pub(crate) fn finish() {
    if let Some(Some(session)) = SESSION.get()
        && let Ok(capture) = session.capture.lock()
        && let Err(error) = capture.write_report(&session.path)
    {
        eprintln!("{error}");
    }
}

pub(crate) struct Measurement(Option<(Metric, Instant)>);

pub(crate) fn measure(metric: Metric) -> Measurement {
    Measurement(is_active().then(|| (metric, Instant::now())))
}

impl Drop for Measurement {
    fn drop(&mut self) {
        if let Some((metric, started)) = self.0 {
            let finished = Instant::now();
            with_capture(|capture| {
                capture.record_duration(metric, finished.saturating_duration_since(started));
                if matches!(metric, Metric::WaveformDraw | Metric::GainDraw)
                    && let Some(input) = capture.pending_input.take()
                {
                    capture.record_duration(
                        Metric::InputToCanvas,
                        finished.saturating_duration_since(input),
                    );
                }
            });
        }
    }
}

/// Called by a passive event filter which emits no messages (and requests no redraw).
pub(crate) fn observe_event(event: &iced::Event) {
    with_capture(|capture| match event {
        iced::Event::Window(iced::window::Event::RedrawRequested(at)) => {
            if let Some(previous) = capture.last_redraw.replace(*at) {
                capture.record_duration(
                    Metric::RedrawInterval,
                    at.saturating_duration_since(previous),
                );
            }
        }
        iced::Event::Window(iced::window::Event::Opened { size, .. }) => {
            capture.window_logical_size = Some([size.width, size.height]);
        }
        iced::Event::Window(iced::window::Event::Resized(size)) => {
            capture.resize_events += 1;
            capture.window_logical_size = Some([size.width, size.height]);
        }
        iced::Event::Window(iced::window::Event::Rescaled(scale)) => {
            capture.window_scale_factor = Some(*scale);
        }
        iced::Event::Mouse(_) | iced::Event::Keyboard(_) => {
            capture.input_events += 1;
            capture.pending_input = Some(Instant::now());
        }
        _ => {}
    });
}
