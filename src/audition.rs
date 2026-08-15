use crate::{GainEnvelope, RenderBlocker, audio_io::Stem};
use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use std::{
    path::PathBuf,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU8, AtomicU32, AtomicUsize, Ordering},
    },
    time::Duration,
};

const SAMPLE_RATE: u32 = 48_000;

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
    mode: AtomicU8,
    monitor: AtomicU32,
    position: AtomicUsize,
    envelopes: RwLock<Vec<GainEnvelope>>,
}

impl Controls {
    fn new(
        mode: AuditionMode,
        monitor: f32,
        position: usize,
        envelopes: Vec<GainEnvelope>,
    ) -> Self {
        Self {
            mode: AtomicU8::new(mode as u8),
            monitor: AtomicU32::new(monitor.to_bits()),
            position: AtomicUsize::new(position),
            envelopes: RwLock::new(envelopes),
        }
    }

    fn mode(&self) -> AuditionMode {
        match self.mode.load(Ordering::Relaxed) {
            1 => AuditionMode::Processed,
            2 => AuditionMode::Removed,
            _ => AuditionMode::Original,
        }
    }
}

struct AuditionSource {
    stems: Vec<Vec<f32>>,
    controls: Arc<Controls>,
    end: usize,
}

impl Iterator for AuditionSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.controls.position.load(Ordering::Relaxed);
        if position >= self.end {
            return None;
        }
        let mode = self.controls.mode();
        let monitor = f32::from_bits(self.controls.monitor.load(Ordering::Relaxed));
        let envelopes = self.controls.envelopes.read().unwrap();
        let sample = self
            .stems
            .iter()
            .enumerate()
            .map(|(track, samples)| {
                let source = samples.get(position).copied().unwrap_or_default();
                let gain = envelopes
                    .get(track)
                    .map_or(1.0, |envelope| envelope.gain_at(position));
                audition_sample(source, gain, mode, monitor)
            })
            .sum::<f32>();
        self.controls
            .position
            .store(position + 1, Ordering::Relaxed);
        Some(sample)
    }
}

impl Source for AuditionSource {
    fn current_span_len(&self) -> Option<usize> {
        Some(
            self.end
                .saturating_sub(self.controls.position.load(Ordering::Relaxed)),
        )
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
        paths: Vec<PathBuf>,
        envelopes: Vec<GainEnvelope>,
        mode: AuditionMode,
        monitor: f32,
        start: usize,
        end: usize,
    ) -> Result<Self, RenderBlocker> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|error| RenderBlocker(format!("audio output unavailable: {error}")))?;
        let player = Sink::connect_new(stream.mixer());
        let stems = paths
            .iter()
            .map(|path| Stem::read(path).map(|stem| stem.samples))
            .collect::<Result<Vec<_>, _>>()?;
        let end = end.min(stems.iter().map(Vec::len).max().unwrap_or_default());
        let controls = Arc::new(Controls::new(mode, monitor, start.min(end), envelopes));
        player.append(AuditionSource {
            stems,
            controls: Arc::clone(&controls),
            end,
        });
        player.play();
        Ok(Self {
            _stream: stream,
            player,
            controls,
            end,
        })
    }

    pub(crate) fn set_mode(&self, mode: AuditionMode) {
        self.controls.mode.store(mode as u8, Ordering::Relaxed);
    }
    pub(crate) fn set_monitor(&self, gain: f32) {
        self.controls
            .monitor
            .store(gain.to_bits(), Ordering::Relaxed);
    }
    pub(crate) fn replace_envelopes(&self, envelopes: Vec<GainEnvelope>) {
        *self.controls.envelopes.write().unwrap() = envelopes;
    }
    pub(crate) fn position(&self) -> usize {
        self.controls.position.load(Ordering::Relaxed)
    }
    pub(crate) fn seek(&self, position: usize) {
        self.controls
            .position
            .store(position.min(self.end), Ordering::Relaxed);
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
            1,
            vec![GainEnvelope::constant(4, 0.5)],
        ));
        let mut source = AuditionSource {
            stems: vec![vec![1.0; 4]],
            controls: Arc::clone(&controls),
            end: 4,
        };
        assert_eq!(source.next(), Some(0.5));
        let position = controls.position.load(Ordering::Relaxed);
        *controls.envelopes.write().unwrap() = vec![GainEnvelope::constant(4, 0.25)];
        assert_eq!(controls.position.load(Ordering::Relaxed), position);
        assert_eq!(source.next(), Some(0.25));
        let position = controls.position.load(Ordering::Relaxed);
        controls
            .mode
            .store(AuditionMode::Removed as u8, Ordering::Relaxed);
        assert_eq!(controls.position.load(Ordering::Relaxed), position);
        assert_eq!(controls.mode(), AuditionMode::Removed);
        assert_eq!(source.next(), Some(0.75));
    }
}
