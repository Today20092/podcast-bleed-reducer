//! Fixed 10 ms peak envelopes from actual host audio, independent of host block size.
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering::SeqCst};

pub(crate) const BINS: usize = 512;
#[derive(Clone, Copy, Debug)]
pub(crate) struct Bin {
    pub input: f32,
    pub output: f32,
    pub gain: f32,
}
struct Slot {
    sequence: AtomicU64,
    peaks: AtomicU64,
    gain: AtomicU32,
}
pub(crate) struct History {
    slots: [Slot; BINS],
    pub latest: AtomicU64,
    pub generation: AtomicU64,
    cleared_through: AtomicU64,
}
impl Default for History {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| Slot {
                sequence: AtomicU64::new(0),
                peaks: AtomicU64::new(0),
                gain: AtomicU32::new(1.0_f32.to_bits()),
            }),
            latest: AtomicU64::new(0),
            generation: AtomicU64::new(0),
            cleared_through: AtomicU64::new(0),
        }
    }
}
impl History {
    pub fn clear(&self) {
        self.cleared_through.store(self.latest.load(SeqCst), SeqCst);
        self.generation.fetch_add(1, SeqCst);
    }
    fn publish(&self, bin: Bin) {
        let sequence = self.latest.load(SeqCst) + 1;
        let slot = &self.slots[sequence as usize % BINS];
        // Sequential ordering makes the two stamp reads reject any torn overwrite.
        slot.sequence.store(0, SeqCst);
        slot.peaks.store(
            u64::from(bin.input.to_bits()) | (u64::from(bin.output.to_bits()) << 32),
            SeqCst,
        );
        slot.gain.store(bin.gain.to_bits(), SeqCst);
        slot.sequence.store(sequence, SeqCst);
        self.latest.store(sequence, SeqCst);
    }
    pub fn read(&self, sequence: u64) -> Option<Bin> {
        if sequence == 0 || sequence <= self.cleared_through.load(SeqCst) {
            return None;
        }
        let slot = &self.slots[sequence as usize % BINS];
        if slot.sequence.load(SeqCst) != sequence {
            return None;
        }
        let peaks = slot.peaks.load(SeqCst);
        let gain = slot.gain.load(SeqCst);
        (slot.sequence.load(SeqCst) == sequence).then_some(Bin {
            input: f32::from_bits(peaks as u32),
            output: f32::from_bits((peaks >> 32) as u32),
            gain: f32::from_bits(gain),
        })
    }
}
pub(crate) struct Recorder {
    samples_per_bin: u32,
    count: u32,
    bin: Bin,
}
impl Recorder {
    pub fn new(rate: u32) -> Self {
        Self {
            samples_per_bin: rate / 100,
            count: 0,
            bin: Bin {
                input: 0.0,
                output: 0.0,
                gain: 1.0,
            },
        }
    }
    pub fn reset(&mut self) {
        self.count = 0;
        self.bin = Bin {
            input: 0.0,
            output: 0.0,
            gain: 1.0,
        };
    }
    pub fn observe(&mut self, input: f32, output: f32, gain: f32, history: &History) {
        self.bin.input = self
            .bin
            .input
            .max(if input.is_finite() { input.abs() } else { 0.0 });
        self.bin.output = self.bin.output.max(if output.is_finite() {
            output.abs()
        } else {
            0.0
        });
        self.bin.gain = self.bin.gain.min(gain);
        self.count += 1;
        if self.count == self.samples_per_bin {
            history.publish(self.bin);
            self.reset();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bins_are_time_based_and_overwrites_are_detected() {
        for rate in [44_100, 48_000, 96_000] {
            let history = History::default();
            let mut recorder = Recorder::new(rate);
            for _ in 0..rate / 100 {
                recorder.observe(0.8, 0.4, 0.5, &history);
            }
            assert_eq!(history.latest.load(SeqCst), 1);
            let bin = history.read(1).unwrap();
            assert_eq!((bin.input, bin.output, bin.gain), (0.8, 0.4, 0.5));
            for _ in 0..BINS {
                history.publish(Bin {
                    input: 0.1,
                    output: 0.1,
                    gain: 1.0,
                });
            }
            assert!(history.read(1).is_none());
            history.clear();
            assert!(history.read(history.latest.load(SeqCst)).is_none());
        }
    }
}
