use crate::{
    GainEnvelope, RenderBlocker, audio_io::Stem, prepared_analysis::source_stamps,
    waveform::SourceStamp,
};
use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use std::{
    path::PathBuf,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    time::Duration,
};

const SAMPLE_RATE: u32 = 48_000;
const CONTROL_BLOCK: usize = 256;

#[derive(Clone)]
pub(crate) struct PreparedAudio {
    stamps: Vec<SourceStamp>,
    // ponytail: one full session of f32 PCM; stream when session size exceeds available memory.
    stems: Arc<Vec<Vec<f32>>>,
}

impl PreparedAudio {
    pub(crate) fn matches_sources(&self, sources: &[SourceStamp]) -> bool {
        self.stamps == sources
    }

    fn prepare(
        paths: &[PathBuf],
        cached: Option<Self>,
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
        let stamps = source_stamps(paths)?;
        if let Some(cached) = cached
            && cached.stamps == stamps
        {
            return Ok(cached);
        }
        let stems = paths
            .iter()
            .map(|path| {
                check_cancelled()?;
                Stem::read_cancellable(path, Some(cancelled)).map(|stem| stem.samples)
            })
            .collect::<Result<Vec<_>, _>>()?;
        check_cancelled()?;
        let prepared = Self {
            stamps,
            stems: Arc::new(stems),
        };
        if !prepared.is_current(paths) {
            return Err(RenderBlocker(
                "sources changed during audio preparation".into(),
            ));
        }
        Ok(prepared)
    }

    pub(crate) fn is_current(&self, paths: &[PathBuf]) -> bool {
        source_stamps(paths).is_ok_and(|stamps| stamps == self.stamps)
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
    pending: Option<Vec<PathBuf>>,
}

impl AuditionPreparation {
    pub(crate) fn request(&mut self, paths: Vec<PathBuf>) {
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

    fn launch(&mut self) {
        if self.job.is_some() {
            return;
        }
        let Some(paths) = self.pending.take() else {
            return;
        };
        let cached = self.cache.take();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            // A cancelled warm request may still return its validated PCM to the cache.
            // Generation checks prevent that result from starting playback.
            let prepared = match cached {
                Some(cached) if cached.is_current(&paths) => Ok(cached),
                cached => PreparedAudio::prepare(&paths, cached, &worker_cancelled),
            };
            let _ = sender.send(prepared);
        });
        self.job = Some(PreparationJob {
            generation: self.generation,
            retain_cache: true,
            cancelled,
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
pub(crate) enum AuditionMode {
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
}

struct Settings {
    mode: AuditionMode,
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
            settings: RwLock::new(Settings {
                mode,
                monitor,
                seek: Some(position),
                envelopes: Arc::new(envelopes),
            }),
        }
    }
}

struct AuditionSource {
    stems: Arc<Vec<Vec<f32>>>,
    controls: Arc<Controls>,
    end: usize,
    position: usize,
    remaining: usize,
    mode: AuditionMode,
    monitor: f32,
    envelopes: Arc<Vec<GainEnvelope>>,
    cursors: Vec<usize>,
}

impl AuditionSource {
    fn new(stems: Arc<Vec<Vec<f32>>>, controls: Arc<Controls>, end: usize) -> Self {
        let cursors = vec![usize::MAX; stems.len()];
        let position = controls.position.load(Ordering::Relaxed);
        Self {
            stems,
            controls,
            end,
            position,
            remaining: 0,
            mode: AuditionMode::Original,
            monitor: 1.0,
            envelopes: Arc::new(vec![]),
            cursors,
        }
    }
}

impl Iterator for AuditionSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            let mut settings = self.controls.settings.write().unwrap();
            if let Some(position) = settings.seek.take() {
                self.position = position;
                self.cursors.fill(usize::MAX);
            }
            self.mode = settings.mode;
            self.monitor = settings.monitor;
            if !Arc::ptr_eq(&self.envelopes, &settings.envelopes) {
                self.envelopes = Arc::clone(&settings.envelopes);
                self.cursors.fill(usize::MAX);
            }
            self.remaining = CONTROL_BLOCK;
        }
        let position = self.position;
        if position >= self.end {
            return None;
        }
        let sample = self
            .stems
            .iter()
            .enumerate()
            .map(|(track, samples)| {
                let source = samples.get(position).copied().unwrap_or_default();
                let gain = self.envelopes.get(track).map_or(1.0, |envelope| {
                    envelope.gain_at_sequential(position, &mut self.cursors[track])
                });
                audition_sample(source, gain, self.mode, self.monitor)
            })
            .sum::<f32>();
        self.position += 1;
        self.remaining -= 1;
        if self.remaining == 0 || self.position == self.end {
            self.controls
                .position
                .store(self.position, Ordering::Relaxed);
        }
        Some(sample)
    }
}

impl Source for AuditionSource {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.end.saturating_sub(self.position))
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.current_span_len()? as f64 / SAMPLE_RATE as f64,
        ))
    }
}

pub(crate) struct LiveAudition {
    _stream: OutputStream,
    player: Sink,
    controls: Arc<Controls>,
    end: usize,
}

impl LiveAudition {
    pub(crate) fn start(
        prepared: PreparedAudio,
        envelopes: Vec<GainEnvelope>,
        mode: AuditionMode,
        monitor: f32,
        start: usize,
        end: usize,
    ) -> Result<Self, RenderBlocker> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|error| RenderBlocker(format!("audio output unavailable: {error}")))?;
        let player = Sink::connect_new(stream.mixer());
        let stems = prepared.stems;
        let end = end.min(stems.iter().map(Vec::len).max().unwrap_or_default());
        let controls = Arc::new(Controls::new(mode, monitor, start.min(end), envelopes));
        player.append(AuditionSource::new(stems, Arc::clone(&controls), end));
        player.play();
        Ok(Self {
            _stream: stream,
            player,
            controls,
            end,
        })
    }

    pub(crate) fn set_mode(&self, mode: AuditionMode) {
        self.controls.settings.write().unwrap().mode = mode;
    }
    pub(crate) fn set_monitor(&self, gain: f32) {
        self.controls.settings.write().unwrap().monitor = gain;
    }
    pub(crate) fn replace_envelopes(&self, envelopes: Vec<GainEnvelope>) {
        self.controls.settings.write().unwrap().envelopes = Arc::new(envelopes);
    }
    pub(crate) fn position(&self) -> usize {
        self.controls.position.load(Ordering::Relaxed)
    }
    pub(crate) fn seek(&self, position: usize) {
        self.controls.settings.write().unwrap().seek = Some(position.min(self.end));
    }
    pub(crate) fn is_finished(&self) -> bool {
        self.position() >= self.end || self.player.empty()
    }
    pub(crate) fn stop(&self) {
        self.player.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(changed.stems[0].len(), 1024);
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
        let mut source =
            AuditionSource::new(Arc::new(vec![vec![1.0; 1024]]), Arc::clone(&controls), 1024);
        assert!(
            source
                .by_ref()
                .take(CONTROL_BLOCK)
                .all(|sample| sample == 0.5)
        );
        let position = controls.position.load(Ordering::Relaxed);
        controls.settings.write().unwrap().envelopes =
            Arc::new(vec![GainEnvelope::constant(1024, 0.25)]);
        assert_eq!(controls.position.load(Ordering::Relaxed), position);
        assert!(
            source
                .by_ref()
                .take(CONTROL_BLOCK)
                .all(|sample| sample == 0.25)
        );
        let position = controls.position.load(Ordering::Relaxed);
        controls.settings.write().unwrap().mode = AuditionMode::Removed;
        assert_eq!(controls.position.load(Ordering::Relaxed), position);
        assert_eq!(source.next(), Some(0.75));
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
        preparation.cancel();
        assert!(wait(&mut preparation).is_empty());
        preparation.request(vec![]);
        preparation.request(vec![PathBuf::from("missing-relinked-source.wav")]);
        let results = wait(&mut preparation);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
        preparation.request(vec![]);
        preparation.invalidate();
        assert!(wait(&mut preparation).is_empty());
    }

    #[test]
    fn cancelled_warm_requests_retain_the_shared_pcm() {
        let mut preparation = AuditionPreparation::default();
        let wait = |preparation: &mut AuditionPreparation| {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while preparation.is_busy() {
                preparation.poll();
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
        };
        preparation.request(vec![]);
        wait(&mut preparation);
        let first = Arc::clone(&preparation.cache.as_ref().unwrap().stems);
        preparation.request(vec![]);
        preparation.cancel();
        wait(&mut preparation);
        assert!(Arc::ptr_eq(
            &first,
            &preparation.cache.as_ref().unwrap().stems
        ));
        preparation.request(vec![]);
        preparation.request(vec![]);
        wait(&mut preparation);
        assert!(Arc::ptr_eq(
            &first,
            &preparation.cache.as_ref().unwrap().stems
        ));
        preparation.request(vec![]);
        preparation.invalidate();
        wait(&mut preparation);
        assert!(preparation.cache.is_none());
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
            let mut source = AuditionSource::new(Arc::clone(&stems), Arc::clone(&controls), 1400);
            for position in 123..123 + CONTROL_BLOCK {
                let expected = stems
                    .iter()
                    .zip(&envelopes)
                    .map(|(samples, envelope)| {
                        audition_sample(samples[position], envelope.gain_at(position), mode, 0.5)
                    })
                    .sum::<f32>();
                assert_eq!(source.next(), Some(expected));
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
                assert_eq!(source.next(), Some(expected));
            }
            assert_eq!(source.next(), None);
            assert_eq!(controls.position.load(Ordering::Relaxed), 1400);
        }
    }
}
