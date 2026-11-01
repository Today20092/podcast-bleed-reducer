//! Bounded display/feature history. Only the ingress is touched by audio callbacks.
use crate::calibration::ReferenceFrame;
use crossbeam::queue::ArrayQueue;
use parking_lot::Mutex;
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU64, Ordering::SeqCst},
};

pub const BINS: usize = 8;
const RETAINED: usize = 938;
const INGRESS: usize = 2048;

#[derive(Clone, Copy, Debug, Default)]
pub struct Peak {
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct HistoryFrame {
    pub epoch: u64,
    pub reference: ReferenceFrame,
    pub peaks: [[Peak; BINS]; 2],
}

impl HistoryFrame {
    pub fn from_audio(
        epoch: u64,
        reference: ReferenceFrame,
        own: &[[f32; 2]],
        peer: &[[f32; 2]],
        channels: usize,
    ) -> Self {
        let mut peaks = [[Peak::default(); BINS]; 2];
        for (mic, audio) in [own, peer].iter().enumerate() {
            for (bin, peak) in peaks[mic].iter_mut().enumerate() {
                let mut minimum = f32::INFINITY;
                let mut maximum = f32::NEG_INFINITY;
                for sample in &audio[bin * audio.len() / BINS..(bin + 1) * audio.len() / BINS] {
                    for value in sample.iter().take(channels.min(2)) {
                        minimum = minimum.min(*value);
                        maximum = maximum.max(*value);
                    }
                }
                if minimum.is_finite() && maximum.is_finite() {
                    *peak = Peak {
                        min: minimum,
                        max: maximum,
                    };
                }
            }
        }
        Self {
            epoch,
            reference,
            peaks,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HistorySnapshot {
    pub epoch: u64,
    pub frames: Vec<HistoryFrame>,
}

pub struct History {
    ingress: ArrayQueue<HistoryFrame>,
    latest_epoch: AtomicU64,
    retained: Mutex<VecDeque<HistoryFrame>>,
}

impl History {
    pub fn new() -> Self {
        Self {
            ingress: ArrayQueue::new(INGRESS),
            latest_epoch: AtomicU64::new(0),
            retained: Mutex::new(VecDeque::with_capacity(RETAINED)),
        }
    }

    pub fn push(&self, frame: HistoryFrame) {
        self.latest_epoch.store(frame.epoch, SeqCst);
        self.ingress.force_push(frame);
    }

    pub fn latest_epoch(&self) -> u64 {
        self.latest_epoch.load(SeqCst)
    }

    pub fn snapshot(&self) -> HistorySnapshot {
        let mut retained = self.retained.lock();
        for _ in 0..INGRESS {
            let Some(frame) = self.ingress.pop() else {
                break;
            };
            if frame.epoch != self.latest_epoch() {
                continue;
            }
            if let Some(previous) = retained.back() {
                if previous.epoch != frame.epoch
                    || previous.reference.rate != frame.reference.rate
                    || previous.reference.channels != frame.reference.channels
                {
                    retained.clear();
                } else if frame.reference.start < previous.reference.end {
                    // Repeated analysis after a model update must not duplicate source time.
                    continue;
                }
            }
            if retained.len() == RETAINED {
                retained.pop_front();
            }
            retained.push_back(frame);
        }
        let epoch = self.latest_epoch();
        if retained.front().is_some_and(|frame| frame.epoch != epoch) {
            retained.clear();
        }
        HistorySnapshot {
            epoch,
            frames: retained.iter().copied().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::Label;
    fn frame(index: i64, epoch: u64) -> HistoryFrame {
        HistoryFrame::from_audio(
            epoch,
            ReferenceFrame {
                start: index * 1536,
                end: (index + 1) * 1536,
                label: Label::None,
                rate: 48000_f32.to_bits(),
                channels: 2,
                features: [[0.0; 4]; 2],
            },
            &[[0.8, -0.8]; 16],
            &[[0.2, -0.3]; 16],
            2,
        )
    }
    #[test]
    fn waveform_bins_preserve_stereo_peaks_without_channel_cancellation() {
        let frame = frame(0, 1);
        assert!(frame.peaks[0].iter().all(|p| p.min == -0.8 && p.max == 0.8));
        assert!(frame.peaks[1].iter().all(|p| p.min == -0.3 && p.max == 0.2));
        let mono =
            HistoryFrame::from_audio(1, frame.reference, &[[0.4, 0.0]; 16], &[[0.2, 0.0]; 16], 1);
        assert!(mono.peaks[0].iter().all(|p| p.min == 0.4 && p.max == 0.4));
    }
    #[test]
    fn snapshots_are_bounded_stable_and_preserve_gaps_but_not_old_epochs() {
        let history = History::new();
        history.push(frame(0, 1));
        history.push(frame(2, 1));
        let frozen = history.snapshot();
        assert_eq!(frozen.frames.len(), 2);
        assert_ne!(
            frozen.frames[0].reference.end,
            frozen.frames[1].reference.start
        );
        for i in 3..4000 {
            history.push(frame(i, 1));
        }
        let live = history.snapshot();
        assert_eq!(live.frames.len(), RETAINED);
        assert_eq!(live.frames.last().unwrap().reference.start, 3999 * 1536);
        assert_eq!(frozen.frames.len(), 2);
        history.push(frame(0, 2));
        let fresh = history.snapshot();
        assert_eq!(fresh.epoch, 2);
        assert_eq!(fresh.frames.len(), 1);
    }
}
