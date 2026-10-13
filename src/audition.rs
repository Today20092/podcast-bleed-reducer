#[cfg(test)]
use crate::prepared_analysis::source_stamps;
use crate::{
    GainEnvelope, RenderBlocker,
    timebase::{Rounding, SourceTimebase, TimePosition, TimeRange},
    waveform::SourceStamp,
};
use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use std::{
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    time::Duration,
};

const CONTROL_BLOCK: usize = 256;

#[cfg(test)]
thread_local! {
    static PLAYBACK_WORK: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0; 3]) };
}
#[cfg(test)]
fn count_playback_work(index: usize) {
    PLAYBACK_WORK.with(|counts| {
        let mut value = counts.get();
        value[index] += 1;
        counts.set(value);
    });
}

#[derive(Clone)]
pub struct PreparedAudio {
    stamps: Vec<SourceStamp>,
    sample_rate: u32,
    stems: Arc<Vec<crate::PreparedAnalysisAudio>>,
    sources: Vec<crate::AudioSource>,
}

impl PreparedAudio {
    pub fn transport(
        &self,
        envelopes: Vec<GainEnvelope>,
        mode: AuditionMode,
        monitor: f32,
        range: TimeRange,
    ) -> Result<AuditionPlayback, RenderBlocker> {
        let mut playback = self.playback(envelopes, mode, monitor, range.start, range.end)?;
        playback.source.persistent = true;
        Ok(playback)
    }
    pub fn duration(&self) -> TimePosition {
        SourceTimebase::new(self.sample_rate, self.frame_count() as u64)
            .expect("validated preparation grid")
            .duration()
            .expect("validated duration")
    }
    pub fn playback(
        &self,
        envelopes: Vec<GainEnvelope>,
        mode: AuditionMode,
        monitor: f32,
        start: TimePosition,
        end: TimePosition,
    ) -> Result<AuditionPlayback, RenderBlocker> {
        let grid = SourceTimebase::new(self.sample_rate, self.frame_count() as u64)?;
        if start > end
            || end > grid.duration()?
            || !monitor.is_finite()
            || !(0.0..=1.0).contains(&monitor)
        {
            return Err(RenderBlocker(
                "invalid audition range or monitoring gain".into(),
            ));
        }
        let start = grid.frame_at_time(start, Rounding::Floor)? as usize;
        let end = grid.frame_at_time(end, Rounding::Ceil)? as usize;
        Ok(AuditionSource::prepare_stream(
            self.clone(),
            Arc::new(Controls::new(mode, monitor, start, envelopes)),
            end,
        ))
    }
    pub fn prepare_for_rate(
        paths: &[PathBuf],
        rate: u32,
        cancelled: &AtomicBool,
    ) -> Result<Self, RenderBlocker> {
        Self::prepare_at_rate(paths, None, rate, cancelled)
    }
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub fn frame_count(&self) -> usize {
        self.stems
            .iter()
            .map(|audio| audio.identity.frame_count as usize)
            .max()
            .unwrap_or_default()
    }
    pub fn read_frames(
        &self,
        range: std::ops::Range<usize>,
    ) -> Result<Vec<Vec<f32>>, RenderBlocker> {
        if range.start > range.end || range.len() > 65_536 || range.end > self.frame_count() {
            return Err(RenderBlocker(
                "playback read must be bounded and inside the prepared grid".into(),
            ));
        }
        self.stems
            .iter()
            .map(|audio| {
                let end = (audio.identity.frame_count as usize).min(range.end);
                let start = range.start.min(end);
                let mut samples = audio.read_frames(start..end)?;
                // Only the common playback rounding tail is extended; native delivery never is.
                samples.resize(range.len(), 0.0);
                Ok(samples)
            })
            .collect()
    }
    pub(crate) fn matches_sources(&self, sources: &[SourceStamp]) -> bool {
        self.stamps == sources
    }

    #[cfg(test)]
    fn prepare(
        paths: &[PathBuf],
        cached: Option<Self>,
        cancelled: &AtomicBool,
    ) -> Result<Self, RenderBlocker> {
        Self::prepare_at_rate(paths, cached, 48_000, cancelled)
    }
    fn prepare_at_rate(
        paths: &[PathBuf],
        cached: Option<Self>,
        rate: u32,
        cancelled: &AtomicBool,
    ) -> Result<Self, RenderBlocker> {
        Self::prepare_sources_for_rate(
            &paths
                .iter()
                .cloned()
                .map(crate::AudioSource::mono)
                .collect::<Vec<_>>(),
            cached,
            rate,
            cancelled,
        )
    }

    pub fn prepare_sources_for_rate(
        inputs: &[crate::AudioSource],
        cached: Option<Self>,
        rate: u32,
        cancelled: &AtomicBool,
    ) -> Result<Self, RenderBlocker> {
        let check_cancelled = || {
            if cancelled.load(Ordering::Relaxed) {
                Err(RenderBlocker("audio preparation cancelled".into()))
            } else {
                Ok(())
            }
        };
        check_cancelled()?;
        let stamps = inputs
            .iter()
            .map(SourceStamp::read_source)
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(cached) = cached
            && cached.stamps == stamps
            && cached.sample_rate == rate
        {
            return Ok(cached);
        }
        #[cfg(test)]
        count_playback_work(0);
        let stems = inputs
            .iter()
            .map(|path| {
                check_cancelled()?;
                crate::analysis_audio::prepare_source_at_rate_in(
                    path,
                    &std::env::temp_dir().join("cleanslate-playback-audio-v1"),
                    rate,
                    Some(cancelled),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        check_cancelled()?;
        crate::engine::validate_source_metadata(
            &stems
                .iter()
                .map(|audio| audio.identity.source)
                .collect::<Vec<_>>(),
        )?;
        let prepared = Self {
            stamps,
            sample_rate: rate,
            stems: Arc::new(stems),
            sources: inputs.to_vec(),
        };
        if !prepared.is_current_sources(inputs) {
            return Err(RenderBlocker(
                "sources changed during audio preparation".into(),
            ));
        }
        Ok(prepared)
    }

    #[cfg(test)]
    pub(crate) fn is_current(&self, paths: &[PathBuf]) -> bool {
        self.is_current_sources(
            &paths
                .iter()
                .cloned()
                .map(crate::AudioSource::mono)
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn is_current_sources(&self, inputs: &[crate::AudioSource]) -> bool {
        self.sources
            .iter()
            .zip(inputs)
            .all(|(a, b)| a.path == b.path && a.channel == b.channel)
            && inputs.len() == self.sources.len()
            && inputs
                .iter()
                .map(SourceStamp::read_source)
                .collect::<Result<Vec<_>, _>>()
                .is_ok_and(|stamps| stamps == self.stamps)
    }
}

struct PreparationJob {
    generation: u64,
    retain_cache: bool,
    cancelled: Arc<AtomicBool>,
    result: Receiver<Result<PreparedAudio, RenderBlocker>>,
}

/// One worker and one shared session of PCM, regardless of repeated Play requests.
#[derive(Default)]
pub(crate) struct AuditionPreparation {
    generation: u64,
    cache: Option<PreparedAudio>,
    job: Option<PreparationJob>,
    pending: Option<Vec<crate::AudioSource>>,
}

impl AuditionPreparation {
    pub(crate) fn ready_sources(&self, sources: &[crate::AudioSource]) -> Option<PreparedAudio> {
        let rate = output_configuration().ok()?.rate;
        self.cache
            .as_ref()
            .filter(|audio| audio.sample_rate == rate && audio.is_current_sources(sources))
            .cloned()
    }
    #[cfg(test)]
    pub(crate) fn request(&mut self, paths: Vec<PathBuf>) {
        self.request_sources(paths.into_iter().map(crate::AudioSource::mono).collect());
    }

    pub(crate) fn request_sources(&mut self, paths: Vec<crate::AudioSource>) {
        self.cancel();
        self.pending = Some(paths);
        self.launch();
    }

    pub(crate) fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending = None;
        if let Some(job) = &self.job {
            job.cancelled.store(true, Ordering::Relaxed);
        }
    }

    pub(crate) fn invalidate(&mut self) {
        self.cancel();
        self.cache = None;
        if let Some(job) = &mut self.job {
            job.retain_cache = false;
        }
    }

    pub(crate) fn is_busy(&self) -> bool {
        self.job.is_some() || self.pending.is_some()
    }

    pub(crate) fn is_requested(&self) -> bool {
        self.pending.is_some()
            || self
                .job
                .as_ref()
                .is_some_and(|job| job.generation == self.generation)
    }

    fn launch(&mut self) {
        if self.job.is_some() {
            return;
        }
        let Some(paths) = self.pending.take() else {
            return;
        };
        // Keep CPAL's cached Windows device enumerator on the long-lived GUI thread.
        #[cfg(not(test))]
        let output_rate = (|| {
            use rodio::cpal::traits::{DeviceTrait, HostTrait};
            let device = rodio::cpal::default_host()
                .default_output_device()
                .ok_or_else(|| {
                    RenderBlocker("audio output unavailable: no default device".into())
                })?;
            device
                .default_output_config()
                .map(|config| config.sample_rate().0)
                .map_err(|error| {
                    RenderBlocker(format!("audio output configuration unavailable: {error}"))
                })
        })();
        // Unit tests exercise real preparation/cancellation without sound hardware.
        #[cfg(test)]
        let output_rate = Ok::<_, RenderBlocker>(48_000);
        let cached = self.cache.take();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            // A cancelled warm request may still return its validated PCM to the cache.
            // Generation checks prevent that result from starting playback.
            let prepared = (|| {
                let rate = output_rate?;
                match cached {
                    Some(cached)
                        if cached.is_current_sources(&paths) && cached.sample_rate == rate =>
                    {
                        Ok(cached)
                    }
                    cached => PreparedAudio::prepare_sources_for_rate(
                        &paths,
                        cached,
                        rate,
                        &worker_cancelled,
                    ),
                }
            })();
            let _ = sender.send(prepared);
        });
        self.job = Some(PreparationJob {
            generation: self.generation,
            retain_cache: true,
            cancelled: Arc::clone(&cancelled),
            result,
        });
    }

    pub(crate) fn poll(&mut self) -> Option<Result<PreparedAudio, RenderBlocker>> {
        let job = self.job.as_ref()?;
        let result = match job.result.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err(RenderBlocker("audio preparation worker stopped".into()))
            }
        };
        let current = job.generation == self.generation;
        if job.retain_cache
            && let Ok(prepared) = &result
        {
            self.cache = Some(prepared.clone());
        }
        self.job = None;
        if current {
            Some(result)
        } else {
            self.launch();
            None
        }
    }
}

impl Drop for AuditionPreparation {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AuditionMode {
    #[default]
    Original,
    Processed,
    Removed,
}

fn audition_sample(source: f32, gain: f32, mode: AuditionMode, monitor: f32) -> f32 {
    let sample = match mode {
        AuditionMode::Original => source,
        AuditionMode::Processed => source * gain,
        AuditionMode::Removed => source * (1.0 - gain),
    };
    sample * monitor
}

struct Controls {
    position: AtomicUsize,
    settings: RwLock<Settings>,
    generation: AtomicUsize,
    requested: AtomicUsize,
    end: AtomicUsize,
    running: AtomicBool,
    underruns: AtomicUsize,
    commands: AtomicUsize,
    first_buffer_command: AtomicUsize,
}

struct Settings {
    range: Option<(usize, usize, bool)>,
    loop_start: Option<usize>,
    mode: AuditionMode,
    solo: u64,
    monitor: f32,
    seek: Option<usize>,
    envelopes: Arc<Vec<GainEnvelope>>,
}

impl Controls {
    fn new(
        mode: AuditionMode,
        monitor: f32,
        position: usize,
        envelopes: Vec<GainEnvelope>,
    ) -> Self {
        Self {
            position: AtomicUsize::new(position),
            generation: AtomicUsize::new(0),
            requested: AtomicUsize::new(position),
            end: AtomicUsize::new(0),
            running: AtomicBool::new(true),
            underruns: AtomicUsize::new(0),
            commands: AtomicUsize::new(0),
            first_buffer_command: AtomicUsize::new(usize::MAX),
            settings: RwLock::new(Settings {
                range: None,
                loop_start: None,
                mode,
                solo: 0,
                monitor,
                seek: None,
                envelopes: Arc::new(envelopes),
            }),
        }
    }
}

struct PlaybackBlock {
    generation: usize,
    start: usize,
    tracks: Vec<Vec<f32>>,
}

/// Device iterator: bounded worker-fed PCM, with silence and a frozen playhead
/// on underrun. Decoding, hashing, resampling and file reads stay on workers.
struct AuditionSource {
    persistent: bool,
    command_generation: usize,
    loop_start: Option<usize>,
    sample_rate: u32,
    blocks: Receiver<(usize, Result<PlaybackBlock, RenderBlocker>)>,
    block: Option<PlaybackBlock>,
    cancelled: Arc<AtomicBool>,
    controls: Arc<Controls>,
    end: usize,
    position: usize,
    remaining: usize,
    mode: AuditionMode,
    solo: u64,
    monitor: f32,
    envelopes: Arc<Vec<GainEnvelope>>,
    failure: Option<RenderBlocker>,
}

struct PlaybackWorker {
    cancelled: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}
impl Drop for PlaybackWorker {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        if let Some(worker) = self.handle.take() {
            let _ = worker.join();
        }
    }
}

/// Headless audition iterator and its off-device worker lifetime. The editor
/// transfers only the private source to rodio and keeps the worker owner.
pub struct AuditionPlayback {
    grid: SourceTimebase,
    source: AuditionSource,
    worker: PlaybackWorker,
}
impl AuditionPlayback {
    pub fn restart(&mut self, range: TimeRange, repeating: bool) -> Result<(), RenderBlocker> {
        restart_controls(&self.source.controls, self.grid, range, repeating)
    }
    pub fn stop(&self) {
        self.source.controls.running.store(false, Ordering::Release);
    }
    pub fn set_loop(&mut self, enabled: bool) {
        self.source.loop_start = enabled
            .then(|| self.source.controls.requested.load(Ordering::Acquire))
            .filter(|start| *start < self.source.end);
        self.source.controls.settings.write().unwrap().loop_start = self.source.loop_start;
    }
    pub fn position_frame(&self) -> usize {
        self.source.position_frame()
    }
    pub fn position(&self) -> TimePosition {
        self.source.position()
    }
    pub fn seek(&self, time: TimePosition) {
        self.source.seek(time);
    }
    pub fn set_mode(&self, mode: AuditionMode) {
        self.source.set_mode(mode);
    }
    pub fn set_monitor(&self, gain: f32) -> Result<(), RenderBlocker> {
        if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
            return Err(RenderBlocker("monitor gain must be finite headroom".into()));
        }
        self.source.controls.settings.write().unwrap().monitor = gain;
        Ok(())
    }
    pub fn replace_envelopes(&self, envelopes: Vec<GainEnvelope>) {
        self.source.controls.settings.write().unwrap().envelopes = Arc::new(envelopes);
    }
    pub fn error(&self) -> Option<&RenderBlocker> {
        self.source.error()
    }
}
impl Iterator for AuditionPlayback {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        self.source.next()
    }
}

impl AuditionSource {
    fn prepare_stream(
        prepared: PreparedAudio,
        controls: Arc<Controls>,
        end: usize,
    ) -> AuditionPlayback {
        let sample_rate = prepared.sample_rate;
        let grid = SourceTimebase::new(sample_rate, prepared.frame_count() as u64)
            .expect("validated playback grid");
        controls.end.store(end, Ordering::Release);
        let position = controls.position.load(Ordering::Relaxed);
        let (sender, blocks) = mpsc::sync_channel(4);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let worker_controls = Arc::clone(&controls);
        #[cfg(test)]
        count_playback_work(2);
        let worker = std::thread::spawn(move || {
            let mut generation = usize::MAX;
            let mut position = 0;
            let mut end = end;
            while !worker_cancelled.load(Ordering::Relaxed) {
                if !worker_controls.running.load(Ordering::Acquire) {
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }
                let current = worker_controls.generation.load(Ordering::Acquire);
                if generation != current {
                    generation = current;
                    end = worker_controls.end.load(Ordering::Acquire);
                    position = worker_controls.requested.load(Ordering::Acquire).min(end);
                }
                if position == end {
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }
                let next = position.saturating_add(4096).min(end);
                let mut result = prepared
                    .read_frames(position..next)
                    .map(|tracks| PlaybackBlock {
                        generation,
                        start: position,
                        tracks,
                    });
                let failed = result.is_err();
                loop {
                    if worker_cancelled.load(Ordering::Relaxed) {
                        return;
                    }
                    if worker_controls.generation.load(Ordering::Acquire) != generation {
                        break;
                    }
                    match sender.try_send((generation, result)) {
                        Ok(()) => {
                            position = if failed { end } else { next };
                            break;
                        }
                        Err(mpsc::TrySendError::Disconnected(_)) => return,
                        Err(mpsc::TrySendError::Full((_, value))) => {
                            result = value;
                            std::thread::sleep(Duration::from_millis(1));
                        }
                    }
                }
            }
        });
        let loop_start = controls.settings.read().unwrap().loop_start;
        let source = Self {
            persistent: false,
            command_generation: 0,
            loop_start,
            sample_rate,
            blocks,
            block: None,
            cancelled: Arc::clone(&cancelled),
            controls,
            end,
            position,
            remaining: 0,
            mode: AuditionMode::Original,
            solo: 0,
            monitor: 1.0,
            envelopes: Arc::new(vec![]),
            failure: None,
        };
        AuditionPlayback {
            grid,
            source,
            worker: PlaybackWorker {
                cancelled: Arc::clone(&cancelled),
                handle: Some(worker),
            },
        }
    }
    pub fn position_frame(&self) -> usize {
        self.controls.position.load(Ordering::Relaxed)
    }
    pub fn position(&self) -> TimePosition {
        SourceTimebase::new(self.sample_rate, self.end as u64)
            .expect("validated playback grid")
            .time_at_frame(self.position_frame() as u64, Rounding::Floor)
            .expect("bounded position")
    }
    pub fn seek(&self, time: TimePosition) {
        let grid = SourceTimebase::new(self.sample_rate, self.end as u64)
            .expect("validated playback grid");
        let frame = grid
            .frame_at_time(
                time.min(grid.duration().expect("validated duration")),
                Rounding::Floor,
            )
            .expect("bounded seek");
        self.controls.settings.write().unwrap().seek = Some(frame as usize);
        self.controls.commands.fetch_add(1, Ordering::AcqRel);
    }
    pub fn set_mode(&self, mode: AuditionMode) {
        self.controls.settings.write().unwrap().mode = mode;
    }
    pub fn error(&self) -> Option<&RenderBlocker> {
        self.failure.as_ref()
    }
}

impl Drop for AuditionSource {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl Iterator for AuditionSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let command = self.controls.commands.load(Ordering::Acquire);
        if self.command_generation != command {
            self.remaining = 0;
        }
        if self.persistent && !self.controls.running.load(Ordering::Acquire) {
            return Some(0.0);
        }
        if self.remaining == 0 {
            if let Ok(mut settings) = self.controls.settings.try_write() {
                if let Some((start, end, repeating)) = settings.range.take() {
                    self.end = end;
                    self.controls.end.store(end, Ordering::Release);
                    settings.seek = Some(start);
                    settings.loop_start = repeating.then_some(start);
                }
                if let Some(position) = settings.seek.take() {
                    self.position = position.min(self.end);
                    self.block = None;
                    self.controls
                        .requested
                        .store(self.position, Ordering::Release);
                    self.controls.generation.fetch_add(1, Ordering::AcqRel);
                    self.controls
                        .position
                        .store(self.position, Ordering::Relaxed);
                }
                self.command_generation = command;
                self.mode = settings.mode;
                self.loop_start = settings.loop_start;
                self.solo = settings.solo;
                self.monitor = settings.monitor;
                if !Arc::ptr_eq(&self.envelopes, &settings.envelopes) {
                    self.envelopes = Arc::clone(&settings.envelopes);
                }
            }
            if self.command_generation != command {
                return Some(0.0);
            }
            self.remaining = CONTROL_BLOCK;
        }
        if self.position >= self.end
            && self.failure.is_none()
            && let Some(start) = self.loop_start
        {
            self.position = start;
            self.block = None;
            self.remaining = CONTROL_BLOCK;
            self.controls.requested.store(start, Ordering::Release);
            self.controls.generation.fetch_add(1, Ordering::AcqRel);
            self.controls.position.store(start, Ordering::Relaxed);
        }
        let position = self.position;
        if position >= self.end || self.failure.is_some() {
            return (self.persistent && self.failure.is_none()).then_some(0.0);
        }
        let generation = self.controls.generation.load(Ordering::Acquire);
        if !self.block.as_ref().is_some_and(|block| {
            block.generation == generation
                && block.start <= position
                && position - block.start < block.tracks.first().map_or(0, Vec::len)
        }) {
            self.block = None;
            // At most the bounded queue is inspected; a stalled worker never
            // advances any track or performs I/O on the device thread.
            for _ in 0..4 {
                match self.blocks.try_recv() {
                    Ok((block_generation, _)) if block_generation != generation => continue,
                    Ok((_, Ok(block))) if block.start == position => {
                        self.block = Some(block);
                        break;
                    }
                    Ok((_, Ok(_))) => continue,
                    Ok((_, Err(error))) => {
                        self.failure = Some(error);
                        return None;
                    }
                    Err(TryRecvError::Disconnected) => {
                        self.failure = Some(RenderBlocker("playback worker stopped".into()));
                        return None;
                    }
                    Err(TryRecvError::Empty) => break,
                }
            }
            if self.block.is_none() {
                self.controls.underruns.fetch_add(1, Ordering::Relaxed);
                self.remaining = 0;
                return Some(0.0);
            }
        }
        let block = self.block.as_ref().expect("prepared playback block");
        let time = SourceTimebase {
            sample_rate: self.sample_rate,
            frame_count: self.end as u64,
        }
        .time_at_frame(position as u64, Rounding::Floor)
        .expect("bounded playback time");
        let sample = self
            .block
            .as_ref()
            .expect("prepared playback block")
            .tracks
            .iter()
            .enumerate()
            .map(|(track, samples)| {
                if self.solo != 0 && self.solo & (1u64 << track) == 0 {
                    return 0.0;
                }
                let source = samples[position - block.start];
                let gain = self
                    .envelopes
                    .get(track)
                    .map_or(1.0, |envelope| envelope.gain_at_time(time));
                audition_sample(source, gain, self.mode, self.monitor)
            })
            .sum::<f32>();
        self.position += 1;
        if self.controls.first_buffer_command.load(Ordering::Relaxed) != self.command_generation {
            self.controls
                .first_buffer_command
                .store(self.command_generation, Ordering::Release);
        }
        self.remaining -= 1;
        self.controls
            .position
            .store(self.position, Ordering::Relaxed);
        Some(sample)
    }
}

impl Source for AuditionSource {
    fn current_span_len(&self) -> Option<usize> {
        if self.persistent || self.loop_start.is_some() {
            None
        } else {
            Some(self.end.saturating_sub(self.position))
        }
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.current_span_len()? as f64 / self.sample_rate as f64,
        ))
    }
}

fn restart_controls(
    controls: &Controls,
    grid: SourceTimebase,
    range: TimeRange,
    repeating: bool,
) -> Result<(), RenderBlocker> {
    if range.start >= range.end || range.end > grid.duration()? {
        return Err(RenderBlocker("invalid audition range".into()));
    }
    let start = grid.frame_at_time(range.start, Rounding::Floor)? as usize;
    let end = grid.frame_at_time(range.end, Rounding::Ceil)? as usize;
    {
        let mut settings = controls.settings.write().unwrap();
        settings.range = Some((start, end, repeating));
        settings.seek = None;
    }
    controls.commands.fetch_add(1, Ordering::AcqRel);
    controls.running.store(true, Ordering::Release);
    Ok(())
}

#[derive(PartialEq, Eq)]
struct OutputConfiguration {
    name: String,
    rate: u32,
    channels: u16,
    format: rodio::cpal::SampleFormat,
}

fn output_configuration() -> Result<OutputConfiguration, RenderBlocker> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};
    let device = rodio::cpal::default_host()
        .default_output_device()
        .ok_or_else(|| RenderBlocker("audio output unavailable: no default device".into()))?;
    let config = device.default_output_config().map_err(|error| {
        RenderBlocker(format!("audio output configuration unavailable: {error}"))
    })?;
    Ok(OutputConfiguration {
        name: device
            .name()
            .map_err(|error| RenderBlocker(format!("audio output unavailable: {error}")))?,
        rate: config.sample_rate().0,
        channels: config.channels(),
        format: config.sample_format(),
    })
}

pub(crate) struct AuditionOutput {
    stream: OutputStream,
    configuration: OutputConfiguration,
    failed: Arc<AtomicBool>,
}

pub(crate) struct LiveAudition {
    grid: SourceTimebase,
    output: Rc<AuditionOutput>,
    prepared: PreparedAudio,
    player: Sink,
    controls: Arc<Controls>,
    _worker: PlaybackWorker,
}

impl LiveAudition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start_on_output(
        output: &mut Option<Rc<AuditionOutput>>,
        prepared: PreparedAudio,
        envelopes: Vec<GainEnvelope>,
        mode: AuditionMode,
        monitor: f32,
        range: TimeRange,
        solo: u64,
        repeating: bool,
    ) -> Result<Self, RenderBlocker> {
        let grid = SourceTimebase::new(prepared.sample_rate, prepared.frame_count() as u64)?;
        let start =
            grid.frame_at_time(range.start.min(grid.duration()?), Rounding::Floor)? as usize;
        let end = grid.frame_at_time(range.end.min(grid.duration()?), Rounding::Ceil)? as usize;
        let configuration = output_configuration()?;
        if configuration.rate != prepared.sample_rate {
            return Err(RenderBlocker(
                "output rate changed; prepare audio again".into(),
            ));
        }
        if output.as_ref().is_none_or(|output| {
            output.configuration != configuration || output.failed.load(Ordering::Acquire)
        }) {
            *output = None;
            #[cfg(test)]
            count_playback_work(1);
            let failed = Arc::new(AtomicBool::new(false));
            let error_flag = Arc::clone(&failed);
            let mut stream = OutputStreamBuilder::from_default_device()
                .map(|builder| {
                    builder
                        .with_sample_rate(prepared.sample_rate)
                        .with_error_callback(move |_| {
                            error_flag.store(true, Ordering::Release);
                        })
                })
                .and_then(OutputStreamBuilder::open_stream)
                .map_err(|error| {
                    RenderBlocker(format!(
                        "audio output unavailable; check device and press Play to retry: {error}"
                    ))
                })?;
            stream.log_on_drop(false);
            *output = Some(Rc::new(AuditionOutput {
                stream,
                configuration,
                failed,
            }));
        }
        let output = Rc::clone(output.as_ref().expect("available output"));
        let player = Sink::connect_new(output.stream.mixer());
        let end = end.min(prepared.frame_count());
        let controls = Arc::new(Controls::new(mode, monitor, start.min(end), envelopes));
        controls.settings.write().unwrap().solo = solo;
        controls.settings.write().unwrap().loop_start = (repeating && start < end).then_some(start);
        let AuditionPlayback {
            mut source, worker, ..
        } = AuditionSource::prepare_stream(prepared.clone(), Arc::clone(&controls), end);
        source.persistent = true;
        player.append(source);
        player.play();
        Ok(Self {
            grid,
            output,
            prepared,
            player,
            controls,
            _worker: worker,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn restart(
        &self,
        prepared: &PreparedAudio,
        envelopes: Vec<GainEnvelope>,
        mode: AuditionMode,
        monitor: f32,
        range: TimeRange,
        solo: u64,
        repeating: bool,
    ) -> Result<bool, RenderBlocker> {
        if self.failed()
            || self.prepared.stamps != prepared.stamps
            || self.prepared.sources != prepared.sources
            || self.prepared.sample_rate != prepared.sample_rate
            || self.output.configuration != output_configuration()?
        {
            return Ok(false);
        }
        self.player.pause();
        {
            let mut settings = self.controls.settings.write().unwrap();
            settings.mode = mode;
            settings.monitor = monitor;
            settings.solo = solo;
            settings.envelopes = Arc::new(envelopes);
            settings.seek = None;
        }
        restart_controls(&self.controls, self.grid, range, repeating)?;
        self.player.play();
        Ok(true)
    }

    pub(crate) fn failed(&self) -> bool {
        self.output.failed.load(Ordering::Acquire) || self.player.empty()
    }

    pub(crate) fn is_buffering(&self) -> bool {
        !self.player.is_paused()
            && self.controls.first_buffer_command.load(Ordering::Acquire)
                != self.controls.commands.load(Ordering::Acquire)
    }

    pub(crate) fn set_mode(&self, mode: AuditionMode) {
        self.controls.settings.write().unwrap().mode = mode;
    }
    pub(crate) fn set_monitor(&self, gain: f32) {
        self.controls.settings.write().unwrap().monitor = gain;
    }
    pub(crate) fn set_solo(&self, solo: u64) {
        self.controls.settings.write().unwrap().solo = solo;
    }
    pub(crate) fn replace_envelopes(&self, envelopes: Vec<GainEnvelope>) {
        let mut settings = self.controls.settings.write().unwrap();
        // Added recordings join on the next start; the active stream keeps its source snapshot.
        if settings.envelopes.len() == envelopes.len() {
            settings.envelopes = Arc::new(envelopes);
        }
    }
    pub(crate) fn position(&self) -> TimePosition {
        self.grid
            .time_at_frame(
                self.controls.position.load(Ordering::Relaxed) as u64,
                Rounding::Floor,
            )
            .expect("bounded playback position")
    }
    pub(crate) fn seek(&self, position: TimePosition) {
        let frame = self
            .grid
            .frame_at_time(
                position.min(self.grid.duration().expect("validated playback grid")),
                Rounding::Floor,
            )
            .expect("bounded playback seek");
        self.controls.settings.write().unwrap().seek =
            Some((frame as usize).min(self.controls.end.load(Ordering::Acquire)));
        self.controls.commands.fetch_add(1, Ordering::AcqRel);
    }
    pub(crate) fn is_finished(&self) -> bool {
        (self.controls.settings.read().unwrap().loop_start.is_none()
            && self.controls.position.load(Ordering::Relaxed)
                >= self.controls.end.load(Ordering::Acquire))
            || self.player.empty()
    }
    pub(crate) fn stop(&self) {
        self.controls.running.store(false, Ordering::Release);
        self.player.pause();
    }
    pub(crate) fn toggle_pause(&self) {
        if self.player.is_paused() {
            self.player.play();
        } else {
            self.player.pause();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "owner-device timing; set CLEANSLATE_PLAYBACK_BENCH_SOURCE"]
    fn owner_selection_start_timings() {
        let path = PathBuf::from(std::env::var("CLEANSLATE_PLAYBACK_BENCH_SOURCE").unwrap());
        let inputs = (0..3)
            .map(|channel| crate::AudioSource {
                path: path.clone(),
                channel: Some(channel),
                name: None,
            })
            .collect::<Vec<_>>();
        use rodio::cpal::traits::{DeviceTrait, HostTrait};
        let device = rodio::cpal::default_host().default_output_device().unwrap();
        let rate = device.default_output_config().unwrap().sample_rate().0;
        println!("DEVICE {:?} rate={rate}", device.name());
        let timer = std::time::Instant::now();
        let prepared =
            PreparedAudio::prepare_sources_for_rate(&inputs, None, rate, &AtomicBool::new(false))
                .unwrap();
        println!("PREPARE {:.3}ms", timer.elapsed().as_secs_f64() * 1000.0);
        let mut output = None;
        let mut live: Option<LiveAudition> = None;
        let initial_work = PLAYBACK_WORK.with(|counts| counts.get());
        for trial in 0..24 {
            let start = (trial % 6) as u64 * crate::TICKS_PER_SECOND;
            let end = start + crate::TICKS_PER_SECOND;
            let timer = std::time::Instant::now();
            let audio = PreparedAudio::prepare_sources_for_rate(
                &inputs,
                Some(prepared.clone()),
                rate,
                &AtomicBool::new(false),
            )
            .unwrap();
            let cache_ms = timer.elapsed().as_secs_f64() * 1000.0;
            if let Some(live) = &live {
                assert!(
                    live.restart(
                        &audio,
                        vec![],
                        AuditionMode::Original,
                        0.0,
                        TimeRange { start, end },
                        0,
                        trial % 2 == 0
                    )
                    .unwrap()
                );
            } else {
                live = Some(
                    LiveAudition::start_on_output(
                        &mut output,
                        audio,
                        vec![],
                        AuditionMode::Original,
                        0.0,
                        TimeRange { start, end },
                        0,
                        trial % 2 == 0,
                    )
                    .unwrap(),
                );
            }
            let live = live.as_ref().unwrap();
            let attach_ms = timer.elapsed().as_secs_f64() * 1000.0;
            let command = live.controls.commands.load(Ordering::Acquire);
            while live.controls.first_buffer_command.load(Ordering::Acquire) != command {
                assert!(timer.elapsed() < Duration::from_secs(5));
                std::thread::sleep(Duration::from_millis(1));
            }
            println!(
                "START {trial} cache_ms={cache_ms:.3} attach_ms={attach_ms:.3} first_ms={:.3}",
                timer.elapsed().as_secs_f64() * 1000.0
            );
            live.stop();
        }
        let final_work = PLAYBACK_WORK.with(|counts| counts.get());
        assert_eq!(
            final_work[0] - initial_work[0],
            0,
            "warm full-source preparations"
        );
        assert_eq!(
            final_work[1] - initial_work[1],
            1,
            "one initial device stream"
        );
        assert_eq!(
            final_work[2] - initial_work[2],
            1,
            "one source worker for every range"
        );
        println!(
            "WORK preparations={} opens={} workers={} underruns={}",
            final_work[0] - initial_work[0],
            final_work[1] - initial_work[1],
            final_work[2] - initial_work[2],
            live.as_ref()
                .unwrap()
                .controls
                .underruns
                .load(Ordering::Relaxed)
        );
    }

    #[test]
    fn selected_loop_repeats_half_open_range_without_finishing() {
        let prepared = prepared_samples(&[vec![0.1, 0.2, 0.3, 0.4]], "selected-loop");
        let grid = SourceTimebase::new(48_000, 4).unwrap();
        let mut playback = prepared
            .playback(
                vec![],
                AuditionMode::Original,
                0.5,
                grid.time_at_frame(1, Rounding::Ceil).unwrap(),
                grid.time_at_frame(3, Rounding::Floor).unwrap(),
            )
            .unwrap();
        playback.set_loop(true);
        for _ in 0..3 {
            assert!((next_at(&mut playback, 1).unwrap() - 0.1).abs() < 0.0001);
            assert!((next_at(&mut playback, 2).unwrap() - 0.15).abs() < 0.0001);
        }
        let mut tiny = prepared
            .playback(
                vec![],
                AuditionMode::Original,
                0.5,
                grid.time_at_frame(1, Rounding::Ceil).unwrap(),
                grid.time_at_frame(2, Rounding::Floor).unwrap(),
            )
            .unwrap();
        tiny.set_loop(true);
        for _ in 0..8 {
            assert!((next_at(&mut tiny, 1).unwrap() - 0.1).abs() < 0.0001);
        }
        tiny.set_loop(false);
        assert!(tiny.next().is_none());
    }

    #[test]
    fn ready_transport_restarts_ranges_after_stop_and_completion() {
        let prepared = prepared_samples(&[vec![0.1, 0.2, 0.3, 0.4]], "ready-restart");
        let grid = SourceTimebase::new(48_000, 4).unwrap();
        let range = |a, b| TimeRange {
            start: grid.time_at_frame(a, Rounding::Floor).unwrap(),
            end: grid.time_at_frame(b, Rounding::Floor).unwrap(),
        };
        let mut playback = prepared
            .transport(vec![], AuditionMode::Original, 1.0, range(0, 2))
            .unwrap();
        assert_eq!(next_at(&mut playback, 0), Some(0.1));
        assert_eq!(next_at(&mut playback, 1), Some(0.2));
        assert_eq!(playback.next(), Some(0.0));
        playback.restart(range(2, 4), false).unwrap();
        assert_eq!(next_at(&mut playback, 2), Some(0.3));
        playback.stop();
        let position = playback.position_frame();
        assert_eq!(playback.next(), Some(0.0));
        assert_eq!(playback.position_frame(), position);
        playback.restart(range(1, 3), true).unwrap();
        for _ in 0..3 {
            assert_eq!(next_at(&mut playback, 1), Some(0.2));
            assert_eq!(next_at(&mut playback, 2), Some(0.3));
        }
        assert!(playback.restart(range(3, 2), false).is_err());
    }

    #[test]
    fn replacing_range_discards_an_older_pending_seek() {
        let prepared = prepared_samples(&[vec![0.1, 0.2, 0.3, 0.4]], "seek-replacement");
        let grid = SourceTimebase::new(48_000, 4).unwrap();
        let time = |frame| grid.time_at_frame(frame, Rounding::Floor).unwrap();
        let mut playback = prepared
            .transport(
                vec![],
                AuditionMode::Original,
                1.0,
                TimeRange {
                    start: time(0),
                    end: time(4),
                },
            )
            .unwrap();
        playback.seek(time(3));
        playback
            .restart(
                TimeRange {
                    start: time(1),
                    end: time(2),
                },
                false,
            )
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        loop {
            let sample = playback.next().unwrap();
            if sample != 0.0 {
                assert_eq!(sample, 0.2);
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
    }

    fn prepared_samples(stems: &[Vec<f32>], name: &str) -> PreparedAudio {
        let root = std::env::temp_dir().join(format!("audition-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let paths: Vec<_> = stems
            .iter()
            .enumerate()
            .map(|(track, samples)| {
                let path = root.join(format!("{track}.wav"));
                let mut format = bwavfile::WaveFmt::new_pcm_mono(48_000, 32);
                format.tag = 3;
                let mut writer = bwavfile::WaveWriter::create(&path, format)
                    .unwrap()
                    .audio_frame_writer()
                    .unwrap();
                writer.write_frames(samples).unwrap();
                writer.end().unwrap();
                path
            })
            .collect();
        let prepared =
            PreparedAudio::prepare_for_rate(&paths, 48_000, &AtomicBool::new(false)).unwrap();
        std::fs::remove_dir_all(root).unwrap();
        prepared
    }
    fn next_at(source: &mut AuditionPlayback, frame: usize) -> Option<f32> {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            let sample = source.next();
            if sample.is_none() || source.position_frame() == frame + 1 {
                return sample;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
    }

    #[test]
    fn preparation_reuses_audio_and_rejects_changed_sources_and_cancellation() {
        let directory = std::env::temp_dir().join(format!("audition-cache-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("voice.wav");
        let write = |samples: &[f32]| {
            let writer =
                bwavfile::WaveWriter::create(&path, bwavfile::WaveFmt::new_pcm_mono(48_000, 16))
                    .unwrap();
            let mut frames = writer.audio_frame_writer().unwrap();
            frames.write_frames(samples).unwrap();
            frames.end().unwrap();
        };
        write(&[0.25; 512]);
        let paths = vec![path.clone()];
        let cancelled = AtomicBool::new(false);
        let first = PreparedAudio::prepare(&paths, None, &cancelled).unwrap();
        let preview_sources = source_stamps(&paths).unwrap();
        assert!(first.matches_sources(&preview_sources));
        let second = PreparedAudio::prepare(&paths, Some(first.clone()), &cancelled).unwrap();
        assert!(Arc::ptr_eq(&first.stems, &second.stems));
        write(&[0.5; 1024]);
        assert!(!first.is_current(&paths));
        let changed = PreparedAudio::prepare(&paths, Some(second), &cancelled).unwrap();
        assert_eq!(changed.stems[0].identity.frame_count, 1024);
        assert!(!changed.matches_sources(&preview_sources));
        assert!(!Arc::ptr_eq(&first.stems, &changed.stems));
        cancelled.store(true, Ordering::Relaxed);
        assert!(PreparedAudio::prepare(&paths, Some(changed), &cancelled).is_err());
        cancelled.store(false, Ordering::Relaxed);
        std::fs::remove_file(&path).unwrap();
        assert!(PreparedAudio::prepare(&paths, Some(first), &cancelled).is_err());
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn modes_are_complementary_and_monitor_gain_is_mode_neutral() {
        let source = 0.8;
        let gain = 0.25;
        let monitor = 0.5;
        assert_eq!(
            audition_sample(source, gain, AuditionMode::Original, monitor),
            0.4
        );
        let processed = audition_sample(source, gain, AuditionMode::Processed, monitor);
        let removed = audition_sample(source, gain, AuditionMode::Removed, monitor);
        assert_eq!(processed, 0.1);
        assert_eq!(removed, 0.3);
        assert_eq!(processed + removed, 0.4);
    }

    #[test]
    fn mode_and_envelope_changes_preserve_position() {
        let controls = Arc::new(Controls::new(
            AuditionMode::Processed,
            1.0,
            0,
            vec![GainEnvelope::constant(1024, 0.5)],
        ));
        let mut source = AuditionSource::prepare_stream(
            prepared_samples(&[vec![1.0; 1024]], "changes"),
            Arc::clone(&controls),
            1024,
        );
        assert!((0..CONTROL_BLOCK).all(|frame| next_at(&mut source, frame) == Some(0.5)));
        let position = controls.position.load(Ordering::Relaxed);
        controls.settings.write().unwrap().envelopes =
            Arc::new(vec![GainEnvelope::constant(1024, 0.25)]);
        assert_eq!(controls.position.load(Ordering::Relaxed), position);
        assert!(
            (CONTROL_BLOCK..CONTROL_BLOCK * 2)
                .all(|frame| next_at(&mut source, frame) == Some(0.25))
        );
        let position = controls.position.load(Ordering::Relaxed);
        controls.settings.write().unwrap().mode = AuditionMode::Removed;
        assert_eq!(controls.position.load(Ordering::Relaxed), position);
        assert_eq!(next_at(&mut source, CONTROL_BLOCK * 2), Some(0.75));
    }

    #[test]
    fn solo_combinations_update_live_without_seeking_in_all_modes() {
        for mode in [
            AuditionMode::Original,
            AuditionMode::Processed,
            AuditionMode::Removed,
        ] {
            let controls = Arc::new(Controls::new(
                mode,
                0.5,
                0,
                vec![GainEnvelope::constant(2048, 0.25); 3],
            ));
            let mut playback = AuditionSource::prepare_stream(
                prepared_samples(
                    &[vec![0.2; 2048], vec![0.4; 2048], vec![0.8; 2048]],
                    &format!("solo-{mode:?}"),
                ),
                Arc::clone(&controls),
                2048,
            );
            for (block, (solo, sum)) in [(0, 1.4), (1, 0.2), (5, 1.0), (2, 0.4), (0, 1.4)]
                .into_iter()
                .enumerate()
            {
                let position = controls.position.load(Ordering::Relaxed);
                controls.settings.write().unwrap().solo = solo;
                assert_eq!(controls.position.load(Ordering::Relaxed), position);
                let expected = audition_sample(sum, 0.25, mode, 0.5);
                for frame in block * CONTROL_BLOCK..(block + 1) * CONTROL_BLOCK {
                    assert!((next_at(&mut playback, frame).unwrap() - expected).abs() < 1e-6);
                }
            }
        }
    }

    #[test]
    fn cancelled_and_relinked_preparations_never_deliver_old_results() {
        let mut preparation = AuditionPreparation::default();
        let wait = |preparation: &mut AuditionPreparation| {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut delivered = vec![];
            while preparation.is_busy() {
                if let Some(result) = preparation.poll() {
                    delivered.push(result);
                }
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
            delivered
        };
        preparation.request(vec![PathBuf::from("missing-audition-source.wav")]);
        assert!(preparation.is_requested());
        preparation.cancel();
        assert!(preparation.is_busy());
        assert!(!preparation.is_requested());
        assert!(wait(&mut preparation).is_empty());
        preparation.request(vec![]);
        preparation.request(vec![PathBuf::from("missing-relinked-source.wav")]);
        assert!(preparation.is_requested());
        let results = wait(&mut preparation);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
        assert!(!preparation.is_requested());
        preparation.request(vec![]);
        preparation.invalidate();
        assert!(wait(&mut preparation).is_empty());
    }

    #[test]
    fn cancelled_warm_requests_retain_the_shared_pcm() {
        let directory = std::env::temp_dir().join(format!("audition-warm-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("voice.wav");
        let writer =
            bwavfile::WaveWriter::create(&path, bwavfile::WaveFmt::new_pcm_mono(48_000, 16))
                .unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(&[0.25_f32; 512]).unwrap();
        frames.end().unwrap();
        let mut preparation = AuditionPreparation::default();
        let wait = |preparation: &mut AuditionPreparation| {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while preparation.is_busy() {
                preparation.poll();
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
        };
        preparation.request(vec![path.clone()]);
        wait(&mut preparation);
        let first = Arc::clone(&preparation.cache.as_ref().unwrap().stems);
        preparation.request(vec![path.clone()]);
        preparation.cancel();
        wait(&mut preparation);
        assert!(Arc::ptr_eq(
            &first,
            &preparation.cache.as_ref().unwrap().stems
        ));
        preparation.request(vec![path.clone()]);
        preparation.request(vec![path.clone()]);
        wait(&mut preparation);
        assert!(Arc::ptr_eq(
            &first,
            &preparation.cache.as_ref().unwrap().stems
        ));
        preparation.request(vec![path]);
        preparation.invalidate();
        wait(&mut preparation);
        assert!(preparation.cache.is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn selection_seek_and_monitoring_match_sample_oracle_in_every_mode() {
        let stems = Arc::new(vec![
            (0..1500)
                .map(|i| (i as f32 * 0.13).sin())
                .collect::<Vec<_>>(),
            vec![0.25; 1500],
        ]);
        let envelopes = vec![
            GainEnvelope::constant(1500, 0.25),
            GainEnvelope::constant(1500, 0.5),
        ];
        for mode in [
            AuditionMode::Original,
            AuditionMode::Processed,
            AuditionMode::Removed,
        ] {
            let controls = Arc::new(Controls::new(mode, 0.5, 123, envelopes.clone()));
            let mut source = AuditionSource::prepare_stream(
                prepared_samples(&stems, "selection"),
                Arc::clone(&controls),
                1400,
            );
            for position in 123..123 + CONTROL_BLOCK {
                let expected = stems
                    .iter()
                    .zip(&envelopes)
                    .map(|(samples, envelope)| {
                        audition_sample(samples[position], envelope.gain_at(position), mode, 0.5)
                    })
                    .sum::<f32>();
                assert_eq!(next_at(&mut source, position), Some(expected));
            }
            controls.settings.write().unwrap().seek = Some(17);
            for position in 17..1400 {
                let expected = stems
                    .iter()
                    .zip(&envelopes)
                    .map(|(samples, envelope)| {
                        audition_sample(samples[position], envelope.gain_at(position), mode, 0.5)
                    })
                    .sum::<f32>();
                assert_eq!(next_at(&mut source, position), Some(expected));
            }
            assert_eq!(source.next(), None);
            assert_eq!(controls.position.load(Ordering::Relaxed), 1400);
        }
    }
}
#[test]
fn native_96khz_audition_requires_the_source_time_adapter() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/import/mono96000.wav");
    assert!(PreparedAudio::prepare(&[source], None, &AtomicBool::new(false)).is_ok());
}
