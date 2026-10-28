//! Bounded inference handoff. Preparation and teardown belong off the audio thread.
use cleanslate_gate_detector::{Detector, Event};
use nice_plug::prelude::ProcessMode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const CHUNK: usize = 256;
const CAPACITY: usize = 32;
const AUDIO_WORDS: usize = CHUNK * 2 + 3; // Generation, public epoch, length, channel samples.

// Private SPSC queues: only the bridge writes audio/reads events, and only the
// worker reads audio/writes events. Atomic payloads avoid unsafe shared storage;
// release/acquire cursors publish each complete record without CAS retry loops.
struct Queue<const N: usize> {
    slots: Box<[[AtomicU64; N]]>,
    written: AtomicUsize,
    read: AtomicUsize,
}

impl<const N: usize> Queue<N> {
    fn new() -> Self {
        Self {
            slots: (0..CAPACITY)
                .map(|_| std::array::from_fn(|_| AtomicU64::new(0)))
                .collect(),
            written: AtomicUsize::new(0),
            read: AtomicUsize::new(0),
        }
    }

    fn remaining(&self) -> usize {
        let written = self.written.load(Ordering::Relaxed);
        CAPACITY - written.wrapping_sub(self.read.load(Ordering::Acquire))
    }

    fn push(&self, record: &[u64; N]) -> bool {
        let written = self.written.load(Ordering::Relaxed);
        if written.wrapping_sub(self.read.load(Ordering::Acquire)) == CAPACITY {
            return false;
        }
        for (slot, value) in self.slots[written % CAPACITY].iter().zip(record) {
            slot.store(*value, Ordering::Relaxed);
        }
        self.written
            .store(written.wrapping_add(1), Ordering::Release);
        true
    }

    fn pop(&self) -> Option<[u64; N]> {
        let read = self.read.load(Ordering::Relaxed);
        if read == self.written.load(Ordering::Acquire) {
            return None;
        }
        let record =
            std::array::from_fn(|i| self.slots[read % CAPACITY][i].load(Ordering::Relaxed));
        self.read.store(read.wrapping_add(1), Ordering::Release);
        Some(record)
    }
}

struct Shared {
    audio: Queue<AUDIO_WORDS>,
    events: Queue<5>,
    generation: AtomicU64,
    fault_generation: AtomicU64,
    processed_generation: AtomicU64,
    processed_samples: AtomicU64,
    stop: AtomicBool,
}

/// A single callback-side owner and one inference thread. Not yet callback-linked.
/// Stereo takes the greater confidence from two independent recurrent detectors;
/// it never averages signed channels. Output gain remains a separate policy.
pub struct WorkerBridge {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
    offline: bool,
    generation: u64,
    epoch: u64,
    channels: u32,
}

impl WorkerBridge {
    /// Allocates queues and prepares the model before starting the worker.
    pub fn prepare(rate: u32, channels: u32, mode: ProcessMode) -> Result<Self, String> {
        if ![1, 2].contains(&channels) {
            return Err("Gate inference requires one or two channels".into());
        }
        let detector = Detector::new(rate)?;
        let right_detector = if channels == 2 {
            Some(Detector::new(rate)?)
        } else {
            None
        };
        let shared = Arc::new(Shared {
            audio: Queue::new(),
            events: Queue::new(),
            generation: AtomicU64::new(1),
            fault_generation: AtomicU64::new(0),
            processed_generation: AtomicU64::new(0),
            processed_samples: AtomicU64::new(0),
            stop: AtomicBool::new(false),
        });
        let worker_shared = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name("cleanslate-gate-inference".into())
            .spawn(move || run(detector, right_detector, worker_shared))
            .map_err(|error| format!("cannot start Gate inference worker: {error}"))?;
        Ok(Self {
            shared,
            thread: Some(thread),
            offline: matches!(mode, ProcessMode::Offline),
            generation: 1,
            epoch: 0,
            channels,
        })
    }

    /// Invalidates in-flight work without touching the worker's recurrent state.
    pub fn reset(&mut self, epoch: u64) {
        let Some(generation) = self.generation.checked_add(1) else {
            self.shared.stop.store(true, Ordering::Release);
            return;
        };
        self.generation = generation;
        self.epoch = epoch;
        self.shared.generation.store(generation, Ordering::Release);
        for _ in 0..CAPACITY {
            if self.shared.events.pop().is_none() {
                break;
            }
        }
    }

    /// Nonblocking, all-or-fault submission. Overflow/invalid input latches until reset.
    pub fn submit(&mut self, left: &[f32], right: Option<&[f32]>) -> bool {
        if self.failed() {
            return false;
        }
        if (self.channels == 2) != right.is_some()
            || right.is_some_and(|samples| samples.len() != left.len())
            || left
                .iter()
                .chain(right.into_iter().flatten())
                .any(|sample| !sample.is_finite())
            || left.len().div_ceil(CHUNK) > self.shared.audio.remaining()
        {
            self.shared
                .fault_generation
                .fetch_max(self.generation, Ordering::Release);
            return false;
        }
        for (chunk_index, samples) in left.chunks(CHUNK).enumerate() {
            let mut record = [0; AUDIO_WORDS];
            record[..3].copy_from_slice(&[self.generation, self.epoch, samples.len() as u64]);
            for (word, sample) in record[3..].iter_mut().zip(samples) {
                *word = u64::from(sample.to_bits());
            }
            if let Some(right) = right {
                let start = chunk_index * CHUNK;
                for (word, sample) in record[3 + CHUNK..]
                    .iter_mut()
                    .zip(&right[start..start + samples.len()])
                {
                    *word = u64::from(sample.to_bits());
                }
            }
            if !self.shared.audio.push(&record) {
                self.shared
                    .fault_generation
                    .fetch_max(self.generation, Ordering::Release);
                return false;
            }
        }
        true
    }

    /// Discards stale generations, including a result published during reset.
    pub fn pop_event(&mut self) -> Option<Event> {
        if self.failed() {
            return None;
        }
        for _ in 0..CAPACITY {
            let record = self.shared.events.pop()?;
            if record[0] == self.generation {
                return Some(Event {
                    epoch: record[1],
                    end_16k: record[2],
                    available_at: record[3],
                    probability: f32::from_bits(record[4] as u32),
                });
            }
        }
        None
    }

    /// Offline-only synchronization, never a realtime wait. Failure stays original.
    /// The 500 ms deadline is an engineering bound, not host/deadline qualification.
    pub fn wait_until_processed(&self, source_sample: u64) -> bool {
        if !self.offline {
            return false;
        }
        let deadline = Instant::now() + Duration::from_millis(500);
        while Instant::now() < deadline {
            if self.failed() {
                return false;
            }
            if self.shared.processed_generation.load(Ordering::Acquire) == self.generation
                && self.shared.processed_samples.load(Ordering::Acquire) >= source_sample
            {
                return true;
            }
            thread::sleep(Duration::from_millis(1));
        }
        self.shared
            .fault_generation
            .fetch_max(self.generation, Ordering::Release);
        false
    }

    /// A failed bridge cannot publish decisions until an explicit stream reset.
    pub fn failed(&self) -> bool {
        self.shared.stop.load(Ordering::Acquire)
            || self.shared.fault_generation.load(Ordering::Acquire) == self.generation
            || self.thread.as_ref().is_none_or(JoinHandle::is_finished)
    }
}

impl Drop for WorkerBridge {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            // The owner must drop off the audio thread; never detach model work.
            let _ = thread.join();
        }
    }
}

fn run(mut detector: Detector, mut right_detector: Option<Detector>, shared: Arc<Shared>) {
    let mut generation = 0;
    let mut processed = 0_u64;
    while !shared.stop.load(Ordering::Acquire) {
        let Some(record) = shared.audio.pop() else {
            thread::sleep(Duration::from_millis(1));
            continue;
        };
        if record[0] != shared.generation.load(Ordering::Acquire)
            || record[0] == shared.fault_generation.load(Ordering::Acquire)
        {
            continue;
        }
        if generation != record[0] {
            detector.reset();
            if let Some(right) = &mut right_detector {
                right.reset();
            }
            generation = record[0];
            processed = 0;
        }
        let count = record[2] as usize;
        let samples: [f32; CHUNK] = std::array::from_fn(|i| f32::from_bits(record[3 + i] as u32));
        // At every supported rate a 256-native-sample chunk contains less than
        // one 512-sample detector frame, so each side emits at most one event.
        let mut left_event = None;
        let mut right_event = None;
        let result = detector.push(&samples[..count], |event| left_event = Some(event));
        let right_result = if let Some(right) = &mut right_detector {
            let samples: [f32; CHUNK] =
                std::array::from_fn(|i| f32::from_bits(record[3 + CHUNK + i] as u32));
            right.push(&samples[..count], |event| right_event = Some(event))
        } else {
            Ok(())
        };
        if result.is_err() || right_result.is_err() {
            shared
                .fault_generation
                .fetch_max(generation, Ordering::Release);
            continue;
        }
        if generation != shared.generation.load(Ordering::Acquire)
            || generation == shared.fault_generation.load(Ordering::Acquire)
        {
            continue;
        }
        let event = match (left_event, right_event) {
            (Some(mut left), Some(right))
                if left.end_16k == right.end_16k && left.available_at == right.available_at =>
            {
                left.probability = left.probability.max(right.probability);
                Some(left)
            }
            (Some(left), None) if right_detector.is_none() => Some(left),
            (None, None) => None,
            _ => {
                shared
                    .fault_generation
                    .fetch_max(generation, Ordering::Release);
                continue;
            }
        };
        if let Some(event) = event
            && !shared.events.push(&[
                generation,
                record[1],
                event.end_16k,
                event.available_at,
                u64::from(event.probability.to_bits()),
            ])
        {
            shared
                .fault_generation
                .fetch_max(generation, Ordering::Release);
        }
        processed += count as u64;
        shared.processed_samples.store(processed, Ordering::Release);
        shared
            .processed_generation
            .store(generation, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_queue_preserves_order_across_full_empty_and_wrapped_storage() {
        let queue = Queue::<2>::new();
        for round in 0..20 {
            assert!(queue.pop().is_none());
            for index in 0..CAPACITY {
                assert!(queue.push(&[round, index as u64]));
            }
            assert_eq!(queue.remaining(), 0);
            assert!(!queue.push(&[99, 99]));
            for index in 0..CAPACITY {
                assert_eq!(queue.pop(), Some([round, index as u64]));
            }
            assert_eq!(queue.remaining(), CAPACITY);
        }
    }

    #[test]
    fn concurrent_queue_records_are_published_whole() {
        let queue = Arc::new(Queue::<4>::new());
        let producer_queue = Arc::clone(&queue);
        let deadline = Instant::now() + Duration::from_secs(2);
        let producer = thread::spawn(move || {
            for index in 0..10_000 {
                while !producer_queue.push(&[index; 4]) {
                    assert!(Instant::now() < deadline, "queue producer stalled");
                    thread::yield_now();
                }
            }
        });
        for index in 0..10_000 {
            loop {
                if let Some(record) = queue.pop() {
                    assert_eq!(record, [index; 4], "partially published queue record");
                    break;
                }
                assert!(Instant::now() < deadline, "queue consumer stalled");
                thread::yield_now();
            }
        }
        producer.join().expect("queue producer panicked");
    }
}
