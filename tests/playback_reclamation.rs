use rust_app::{AuditionMode, AuditionPlayback, GainEnvelope, PreparedAudio};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

struct MeasuredAllocator;

thread_local! {
    // Only the thread producing the observed sample is measured, never its worker.
    static COUNTS: Cell<Option<[usize; 3]>> = const { Cell::new(None) };
}

fn count(index: usize) {
    COUNTS.with(|counts| {
        if let Some(mut value) = counts.get() {
            value[index] += 1;
            counts.set(Some(value));
        }
    });
}

// SAFETY: Every request is forwarded unchanged to System. The constant TLS
// counter neither allocates nor dereferences the caller's storage.
unsafe impl GlobalAlloc for MeasuredAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(0);
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count(0);
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count(1);
        unsafe { System.realloc(ptr, layout, size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        count(2);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: MeasuredAllocator = MeasuredAllocator;

#[derive(Default)]
struct Observation {
    counts: [usize; 3],
    calls: usize,
    silent_waits: usize,
    durations: Vec<Duration>,
}

impl Observation {
    fn next(&mut self, playback: &mut AuditionPlayback) -> Option<f32> {
        let at = Instant::now();
        COUNTS.with(|counts| counts.set(Some([0; 3])));
        let sample = playback.next();
        let counts = COUNTS.with(|counts| counts.replace(None).unwrap());
        let elapsed = at.elapsed();
        for (total, value) in self.counts.iter_mut().zip(counts) {
            *total += value;
        }
        self.calls += 1;
        self.durations.push(elapsed);
        sample
    }

    fn consume(&mut self, playback: &mut AuditionPlayback, frames: usize, expected: f32) {
        let mut observed = 0;
        let deadline = Instant::now() + Duration::from_secs(5);
        while observed < frames {
            let before = playback.position_frame();
            let sample = self.next(playback).expect("stream remains available");
            if playback.position_frame() > before {
                assert!((sample - expected).abs() < 1e-6);
                observed += 1;
            } else {
                assert_eq!(sample, 0.0, "starvation must emit silence without seeking");
                self.silent_waits += 1;
                std::thread::sleep(Duration::from_micros(50));
            }
            assert!(Instant::now() < deadline, "playback worker stalled");
        }
    }

    fn report(&mut self, scenario: &str) {
        self.durations.sort_unstable();
        let at =
            |percent: usize| self.durations[(self.durations.len() - 1) * percent / 100].as_nanos();
        eprintln!(
            "PLAYBACK_CONSUMER scenario={scenario} calls={} alloc={} realloc={} dealloc={} silent_waits={} next_p50_ns={} next_p95_ns={} next_p99_ns={} next_max_ns={}",
            self.calls,
            self.counts[0],
            self.counts[1],
            self.counts[2],
            self.silent_waits,
            at(50),
            at(95),
            at(99),
            self.durations.last().unwrap().as_nanos(),
        );
    }
}

#[test]
fn sample_production_keeps_pcm_and_envelope_reclamation_off_the_consumer() {
    let root = std::env::temp_dir().join(format!("playback-reclamation-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("constant.wav");
    let mut format = bwavfile::WaveFmt::new_pcm_mono(48_000, 32);
    format.tag = 3;
    let mut writer = bwavfile::WaveWriter::create(&path, format)
        .unwrap()
        .audio_frame_writer()
        .unwrap();
    writer.write_frames(&vec![0.5; 32_768]).unwrap();
    writer.end().unwrap();
    let prepared =
        PreparedAudio::prepare_for_rate(&[path], 48_000, &AtomicBool::new(false)).unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let envelope = |gain| GainEnvelope::constant_at_rate(32_768, 48_000, gain).unwrap();
    let mut playback = prepared
        .playback(
            vec![envelope(0.5)],
            AuditionMode::Processed,
            1.0,
            0,
            prepared.duration(),
        )
        .unwrap();

    let mut steady = Observation::default();
    steady.consume(&mut playback, 20_480, 0.25);
    steady.report("steady-block-transitions");

    let mut edits = Observation::default();
    playback.replace_envelopes(vec![envelope(0.25)]);
    edits.consume(&mut playback, 256, 0.125);
    playback.set_mode(AuditionMode::Removed);
    edits.consume(&mut playback, 256, 0.375);
    playback.set_mode(AuditionMode::Original);
    edits.consume(&mut playback, 256, 0.5);
    playback.set_mode(AuditionMode::Processed);
    for _ in 0..40 {
        playback.replace_envelopes(vec![envelope(0.5)]);
        playback.replace_envelopes(vec![envelope(0.25)]);
        edits.consume(&mut playback, 256, 0.125);
    }
    edits.report("envelope-and-listening-modes");

    let mut seeks = Observation::default();
    playback.set_mode(AuditionMode::Original);
    for _ in 0..40 {
        playback.seek(0);
        seeks.consume(&mut playback, 1, 0.5);
    }
    seeks.report("rapid-seeks-and-stale-blocks");

    COUNTS.with(|counts| counts.set(Some([0; 3])));
    drop(playback);
    let teardown = COUNTS.with(|counts| counts.replace(None).unwrap());
    eprintln!(
        "PLAYBACK_CONSUMER scenario=headless-teardown alloc={} realloc={} dealloc={}",
        teardown[0], teardown[1], teardown[2]
    );

    assert_eq!(
        steady.counts, [0; 3],
        "steady sample production must not reclaim PCM"
    );
    assert_eq!(
        edits.counts, [0; 3],
        "sample production must not reclaim old envelopes"
    );
    assert_eq!(
        seeks.counts, [0; 3],
        "discarded stale PCM must be reclaimed off consumer"
    );
}
