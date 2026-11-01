//! Bounded PCM transport proof. Outputs remain dry; PCM never leaves memory.
use crate::probe::{Binding, Registry};
use crossbeam::queue::ArrayQueue;
use std::sync::OnceLock;

const PACKET_FRAMES: usize = 256;
const QUEUE_PACKETS: usize = 64;
#[cfg(test)]
const HISTORY_FRAMES: usize = 16_384;
const MAX_CALLBACK_FRAMES: usize = 8192;
pub const LOOKBACK: i64 = 2048;
pub const WINDOW: usize = 512;

static BUS: OnceLock<Bus> = OnceLock::new();

pub fn bus() -> &'static Bus {
    BUS.get_or_init(Bus::new)
}

pub struct Bus {
    queues: [ArrayQueue<Packet>; 16],
}

impl Bus {
    fn new() -> Self {
        Self {
            queues: std::array::from_fn(|_| ArrayQueue::new(QUEUE_PACKETS)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Stamp {
    epoch: u64,
    owner: u64,
    rate: u32,
    channels: usize,
}

#[derive(Clone, Copy)]
struct Packet {
    stamp: Stamp,
    recipient: u64,
    start: i64,
    frames: usize,
    samples: [[f32; 2]; PACKET_FRAMES],
}

struct History {
    samples: Vec<[f32; 2]>,
    stamp: Option<Stamp>,
    start: i64,
    end: i64,
}

impl History {
    fn new_with_capacity(frames: usize) -> Self {
        Self {
            samples: vec![[0.0; 2]; frames],
            stamp: None,
            start: 0,
            end: 0,
        }
    }

    #[cfg(test)]
    fn new() -> Self {
        Self::new_with_capacity(HISTORY_FRAMES)
    }

    fn clear(&mut self) {
        self.stamp = None;
    }

    fn push(&mut self, packet: &Packet) {
        if self.stamp != Some(packet.stamp) || self.end != packet.start {
            self.stamp = Some(packet.stamp);
            self.start = packet.start;
        }
        for (offset, &sample) in packet.samples[..packet.frames].iter().enumerate() {
            let position = packet.start + offset as i64;
            let index = position.rem_euclid(self.samples.len() as i64) as usize;
            self.samples[index] = sample;
        }
        self.end = packet.start + packet.frames as i64;
        self.start = self
            .start
            .max(self.end.saturating_sub(self.samples.len() as i64));
    }

    fn covers(&self, start: i64, stamp: Stamp) -> bool {
        self.covers_range(start, WINDOW, stamp)
    }

    fn covers_range(&self, start: i64, frames: usize, stamp: Stamp) -> bool {
        self.stamp == Some(stamp)
            && start >= self.start
            && start
                .checked_add(frames as i64)
                .is_some_and(|end| end <= self.end)
    }

    fn sample(&self, position: i64) -> [f32; 2] {
        self.samples[position.rem_euclid(self.samples.len() as i64) as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShareStatus {
    Inactive,
    MissingTime,
    MissingPeer,
    Warmup,
    Ready,
    QueueOverflow,
    InvalidAudio,
    UnsupportedBuffer,
    Incompatible,
    Reset,
}

#[derive(Clone, Copy, Debug)]
pub struct ShareReport {
    pub status: ShareStatus,
    pub start: Option<i64>,
    pub paired_frames: usize,
    pub queue_dropped: u64,
    pub peer_depth: usize,
}

impl ShareReport {
    fn new(status: ShareStatus) -> Self {
        Self {
            status,
            start: None,
            paired_frames: 0,
            queue_dropped: 0,
            peer_depth: 0,
        }
    }
}

impl Default for ShareReport {
    fn default() -> Self {
        Self::new(ShareStatus::Inactive)
    }
}

pub struct SharedAudio {
    own: History,
    peer: History,
    binding: Option<Binding>,
    dropped: u64,
}

impl SharedAudio {
    pub fn for_processing() -> Self {
        Self {
            own: History::new_with_capacity(131_072),
            peer: History::new_with_capacity(131_072),
            binding: None,
            dropped: 0,
        }
    }

    pub fn read_pair(
        &self,
        registry: &Registry,
        binding: Binding,
        start: i64,
        own: &mut [[f32; 2]],
        peer: &mut [[f32; 2]],
    ) -> bool {
        if own.len() != peer.len()
            || own.is_empty()
            || own.len() > 8192
            || self.binding != Some(binding)
            || !registry.is_current(binding)
        {
            return false;
        }
        let Some(stamp) = self.own.stamp else {
            return false;
        };
        if stamp.owner != binding.owner || stamp.epoch != binding.epoch {
            return false;
        }
        let peer_stamp = Stamp {
            owner: binding.peer,
            ..stamp
        };
        if !self.own.covers_range(start, own.len(), stamp)
            || !self.peer.covers_range(start, peer.len(), peer_stamp)
        {
            return false;
        }
        for (offset, (own, peer)) in own.iter_mut().zip(peer).enumerate() {
            *own = self.own.sample(start + offset as i64);
            *peer = self.peer.sample(start + offset as i64);
        }
        registry.is_current(binding)
    }
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            own: History::new(),
            peer: History::new(),
            binding: None,
            dropped: 0,
        }
    }

    pub fn reset(&mut self) {
        self.own.clear();
        self.peer.clear();
        self.binding = None;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        bus: &Bus,
        registry: &Registry,
        binding: Option<Binding>,
        start: Option<i64>,
        rate: u32,
        channels: &[&mut [f32]],
        active: bool,
    ) -> ShareReport {
        let Some(binding) = binding.filter(|b| registry.is_current(*b)) else {
            self.reset();
            return ShareReport::new(ShareStatus::Inactive);
        };
        if self.binding != Some(binding) {
            self.own.clear();
            self.peer.clear();
            self.binding = Some(binding);
        }
        let mut report = ShareReport::new(ShareStatus::Warmup);
        report.queue_dropped = self.dropped;
        if !active {
            self.own.clear();
            self.peer.clear();
            report.status = ShareStatus::Inactive;
            return report;
        }
        let Some(start) = start else {
            self.reset();
            report.status = ShareStatus::MissingTime;
            return report;
        };
        let frames = channels.first().map_or(0, |channel| channel.len());
        if !(1..=2).contains(&channels.len())
            || frames == 0
            || frames > MAX_CALLBACK_FRAMES
            || channels.iter().any(|channel| channel.len() != frames)
            || start.checked_add(frames as i64).is_none()
        {
            self.own.clear();
            report.status = ShareStatus::UnsupportedBuffer;
            return report;
        }
        let rate_hz = f32::from_bits(rate);
        if !rate_hz.is_finite()
            || rate_hz <= 0.0
            || channels
                .iter()
                .any(|channel| channel.iter().any(|sample| !sample.is_finite()))
        {
            self.own.clear();
            report.status = ShareStatus::InvalidAudio;
            return report;
        }
        let stamp = Stamp {
            epoch: binding.epoch,
            owner: binding.owner,
            rate,
            channels: channels.len(),
        };
        for offset in (0..frames).step_by(PACKET_FRAMES) {
            let count = (frames - offset).min(PACKET_FRAMES);
            let mut packet = Packet {
                stamp,
                recipient: binding.peer,
                start: start + offset as i64,
                frames: count,
                samples: [[0.0; 2]; PACKET_FRAMES],
            };
            for (channel_index, channel) in channels.iter().enumerate() {
                for (sample, &input) in packet.samples[..count]
                    .iter_mut()
                    .zip(&channel[offset..offset + count])
                {
                    sample[channel_index] = input;
                }
            }
            self.own.push(&packet);
            if binding.peer != 0 && bus.queues[binding.index].push(packet).is_err() {
                self.dropped += 1;
                report.status = ShareStatus::QueueOverflow;
            }
        }
        report.queue_dropped = self.dropped;
        if binding.peer == 0 {
            report.status = ShareStatus::MissingPeer;
            return report;
        }
        // One fixed-capacity drain; never wait for another callback to publish.
        for _ in 0..QUEUE_PACKETS {
            let Some(packet) = bus.queues[binding.index ^ 1].pop() else {
                break;
            };
            if packet.stamp.epoch == binding.epoch
                && packet.stamp.owner == binding.peer
                && packet.recipient == binding.owner
            {
                self.peer.push(&packet);
            }
        }
        report.peer_depth = bus.queues[binding.index ^ 1].len();
        if !registry.is_current(binding) {
            self.reset();
            report.status = ShareStatus::Reset;
            return report;
        }
        if report.status == ShareStatus::QueueOverflow {
            return report;
        }
        let peer_stamp = Stamp {
            owner: binding.peer,
            ..stamp
        };
        if self
            .peer
            .stamp
            .is_some_and(|s| s.rate != rate || s.channels != channels.len())
        {
            report.status = ShareStatus::Incompatible;
            return report;
        }
        let Some(window_start) = start.checked_sub(LOOKBACK) else {
            return report;
        };
        if self.own.covers(window_start, stamp) && self.peer.covers(window_start, peer_stamp) {
            // Read and validate both actual PCM windows; only availability/timing reaches logs.
            for time in window_start..window_start + WINDOW as i64 {
                if self.own.sample(time)[..stamp.channels]
                    .iter()
                    .chain(self.peer.sample(time)[..stamp.channels].iter())
                    .any(|x| !x.is_finite())
                {
                    report.status = ShareStatus::InvalidAudio;
                    return report;
                }
            }
            if !registry.is_current(binding) {
                report.status = ShareStatus::Reset;
                return report;
            }
            report.start = Some(window_start);
            report.paired_frames = WINDOW;
            report.status = ShareStatus::Ready;
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::Probe;

    fn step(
        audio: &mut SharedAudio,
        bus: &Bus,
        registry: &Registry,
        probe: &mut Probe,
        member: i32,
        start: i64,
        samples: &mut [f32],
    ) -> ShareReport {
        probe.observe(
            registry,
            1,
            member,
            Some(start),
            samples.len() as u64,
            48000_f32.to_bits(),
            true,
        );
        audio.process(
            bus,
            registry,
            probe.binding(registry),
            Some(start),
            48000_f32.to_bits(),
            &[samples],
            true,
        )
    }

    #[test]
    fn exact_pcm_pairing_with_serial_callbacks_and_variable_packets() {
        let (bus, registry) = (Bus::new(), Registry::new());
        let (mut a, mut b) = (SharedAudio::new(), SharedAudio::new());
        let (mut pa, mut pb) = (Probe::new(), Probe::new());
        let mut start = 0;
        let mut ready = 0;
        for length in [1, 215, 511, 128, 1024, 256, 215, 1024, 1024, 1024] {
            let mut sa: Vec<_> = (0..length)
                .map(|i| (start + i as i64) as f32 / 10_000.0)
                .collect();
            let mut sb: Vec<_> = sa.iter().map(|x| -*x).collect();
            let ra = step(&mut a, &bus, &registry, &mut pa, 1, start, &mut sa);
            let rb = step(&mut b, &bus, &registry, &mut pb, 2, start, &mut sb);
            for (state, report, sign) in [(&a, ra, 1.0), (&b, rb, -1.0)] {
                if report.status == ShareStatus::Ready {
                    ready += 1;
                    let time = report.start.unwrap();
                    assert_eq!(state.own.sample(time)[0], sign * time as f32 / 10_000.0);
                    assert_eq!(state.peer.sample(time)[0], -sign * time as f32 / 10_000.0);
                    assert_eq!(report.paired_frames, WINDOW);
                }
                assert_eq!(report.queue_dropped, 0);
            }
            start += length as i64;
        }
        assert!(ready >= 4);
        // Rewinding to a previously valid window must not reuse the old generation.
        let mut samples = [0.25; 256];
        assert_ne!(
            step(&mut a, &bus, &registry, &mut pa, 1, 0, &mut samples).status,
            ShareStatus::Ready
        );
        pb.reset(&registry);
        assert_eq!(
            step(&mut a, &bus, &registry, &mut pa, 1, 256, &mut samples).status,
            ShareStatus::MissingPeer
        );
    }

    #[test]
    fn history_wrap_and_gaps_do_not_fabricate_samples() {
        let mut history = History::new();
        let stamp = Stamp {
            epoch: 1,
            owner: 1,
            rate: 1,
            channels: 1,
        };
        for index in 0..100 {
            let packet = Packet {
                stamp,
                recipient: 2,
                start: index * 256,
                frames: 256,
                samples: [[index as f32; 2]; 256],
            };
            history.push(&packet);
        }
        assert!(!history.covers(0, stamp));
        assert!(history.covers(24000, stamp));
        assert_eq!(history.sample(24000)[0], (24000 / 256) as f32);
        history.push(&Packet {
            stamp,
            recipient: 2,
            start: -256,
            frames: 256,
            samples: [[0.5; 2]; 256],
        });
        assert!(!history.covers(24000, stamp));
        assert_eq!(history.sample(-1)[0], 0.5);
    }

    #[test]
    fn queue_overflow_invalid_audio_and_duplicate_members_stay_unready() {
        let (bus, registry) = (Bus::new(), Registry::new());
        let (mut a, mut pa, mut pb) = (SharedAudio::new(), Probe::new(), Probe::new());
        pb.observe(&registry, 1, 2, Some(0), 256, 48000_f32.to_bits(), true);
        let mut samples = [0.25; 256];
        let mut report = ShareReport::new(ShareStatus::Inactive);
        for i in 0..QUEUE_PACKETS + 5 {
            report = step(
                &mut a,
                &bus,
                &registry,
                &mut pa,
                1,
                (i * 256) as i64,
                &mut samples,
            );
        }
        assert_eq!(report.status, ShareStatus::QueueOverflow);
        assert_eq!(report.queue_dropped, 5);
        assert_eq!(bus.queues[0].len(), QUEUE_PACKETS);
        samples[0] = f32::NAN;
        assert_eq!(
            step(&mut a, &bus, &registry, &mut pa, 1, 20000, &mut samples).status,
            ShareStatus::InvalidAudio
        );
        let mut duplicate = Probe::new();
        duplicate.observe(&registry, 1, 1, Some(20000), 256, 48000_f32.to_bits(), true);
        assert_eq!(
            a.process(
                &bus,
                &registry,
                duplicate.binding(&registry),
                Some(20000),
                48000_f32.to_bits(),
                &[&mut samples],
                true
            )
            .status,
            ShareStatus::Inactive
        );
    }

    #[test]
    fn alternating_callback_order_and_replaced_peer_recover_without_old_audio() {
        let (bus, registry) = (Bus::new(), Registry::new());
        let (mut a, mut b, mut pa, mut pb) = (
            SharedAudio::new(),
            SharedAudio::new(),
            Probe::new(),
            Probe::new(),
        );
        let mut start = 0;
        let mut ready = 0;
        for i in 0..300 {
            let length = i % 215 + 1;
            let (mut sa, mut sb) = ([0.125; 215], [-0.25; 215]);
            let reports = if i % 2 == 0 {
                [
                    step(
                        &mut a,
                        &bus,
                        &registry,
                        &mut pa,
                        1,
                        start,
                        &mut sa[..length],
                    ),
                    step(
                        &mut b,
                        &bus,
                        &registry,
                        &mut pb,
                        2,
                        start,
                        &mut sb[..length],
                    ),
                ]
            } else {
                [
                    step(
                        &mut b,
                        &bus,
                        &registry,
                        &mut pb,
                        2,
                        start,
                        &mut sb[..length],
                    ),
                    step(
                        &mut a,
                        &bus,
                        &registry,
                        &mut pa,
                        1,
                        start,
                        &mut sa[..length],
                    ),
                ]
            };
            for report in reports {
                if report.status == ShareStatus::Ready {
                    ready += 1;
                }
                assert_eq!(report.queue_dropped, 0);
            }
            start += length as i64;
        }
        assert!(ready > 200);
        pb.reset(&registry);
        let mut replacement = Probe::new();
        let mut new_audio = SharedAudio::new();
        let mut new_samples = [0.75; 256];
        assert_ne!(
            step(
                &mut new_audio,
                &bus,
                &registry,
                &mut replacement,
                2,
                start,
                &mut new_samples
            )
            .status,
            ShareStatus::Ready
        );
        let mut samples = [0.125; 256];
        assert_ne!(
            step(&mut a, &bus, &registry, &mut pa, 1, start, &mut samples).status,
            ShareStatus::Ready
        );
    }

    #[test]
    fn missing_time_oversize_and_layout_changes_cannot_claim_a_window() {
        let (bus, registry) = (Bus::new(), Registry::new());
        let (mut a, mut pa, mut pb) = (SharedAudio::new(), Probe::new(), Probe::new());
        pa.observe(&registry, 1, 1, Some(0), 256, 48000_f32.to_bits(), true);
        pb.observe(&registry, 1, 2, Some(0), 256, 48000_f32.to_bits(), true);
        let binding = pa.binding(&registry).unwrap();
        let mut samples = [0.25; 256];
        assert_eq!(
            a.process(
                &bus,
                &registry,
                Some(binding),
                None,
                48000_f32.to_bits(),
                &[&mut samples],
                true
            )
            .status,
            ShareStatus::MissingTime
        );
        let mut huge = vec![0.25; MAX_CALLBACK_FRAMES + 1];
        assert_eq!(
            a.process(
                &bus,
                &registry,
                Some(binding),
                Some(0),
                48000_f32.to_bits(),
                &[&mut huge],
                true
            )
            .status,
            ShareStatus::UnsupportedBuffer
        );
        // Deliberately deliver a different-rate, stereo peer packet to a mono 48 kHz instance.
        let peer = Packet {
            stamp: Stamp {
                owner: binding.peer,
                epoch: binding.epoch,
                rate: 44100_f32.to_bits(),
                channels: 2,
            },
            recipient: binding.owner,
            start: 0,
            frames: 256,
            samples: [[0.5; 2]; 256],
        };
        assert!(bus.queues[binding.index ^ 1].push(peer).is_ok());
        assert_eq!(
            a.process(
                &bus,
                &registry,
                Some(binding),
                Some(0),
                48000_f32.to_bits(),
                &[&mut samples],
                true
            )
            .status,
            ShareStatus::Incompatible
        );
        pa.reset(&registry);
        assert_eq!(
            a.process(
                &bus,
                &registry,
                Some(binding),
                Some(0),
                48000_f32.to_bits(),
                &[&mut samples],
                true
            )
            .status,
            ShareStatus::Inactive
        );
    }

    #[test]
    fn concurrent_callbacks_deliver_intact_pcm_without_callback_waits() {
        use std::sync::{Arc, Barrier};
        let (bus, registry, barrier) = (
            Arc::new(Bus::new()),
            Arc::new(Registry::new()),
            Arc::new(Barrier::new(2)),
        );
        let threads: Vec<_> = (1..=2)
            .map(|member| {
                let (bus, registry, barrier) = (bus.clone(), registry.clone(), barrier.clone());
                std::thread::spawn(move || {
                    let (mut audio, mut probe) = (SharedAudio::new(), Probe::new());
                    let mut ready = 0;
                    for index in 0..100 {
                        let mut samples = [member as f32 / 4.0; 256];
                        let report = step(
                            &mut audio,
                            &bus,
                            &registry,
                            &mut probe,
                            member,
                            index * 256,
                            &mut samples,
                        );
                        if report.status == ShareStatus::Ready {
                            let start = report.start.unwrap();
                            assert_eq!(audio.own.sample(start)[0], member as f32 / 4.0);
                            assert_eq!(audio.peer.sample(start)[0], (3 - member) as f32 / 4.0);
                            ready += 1;
                        }
                        assert_eq!(report.queue_dropped, 0);
                        // Test scheduling is synchronized between callbacks, never inside process().
                        barrier.wait();
                    }
                    ready
                })
            })
            .collect();
        for thread in threads {
            assert!(thread.join().unwrap() > 50);
        }
    }

    #[test]
    fn recorded_clap_export_skew_is_rejected_and_matching_timeline_recovers() {
        // Observed CLAP bounce skew, replayed with synthetic PCM instead of a recording.
        let (bus, registry) = (Bus::new(), Registry::new());
        let (mut a, mut b, mut pa, mut pb) = (
            SharedAudio::new(),
            SharedAudio::new(),
            Probe::new(),
            Probe::new(),
        );
        for i in 0..100 {
            let (mut sa, mut sb) = ([0.125; 215], [-0.25; 215]);
            for report in [
                step(
                    &mut a,
                    &bus,
                    &registry,
                    &mut pa,
                    1,
                    i * 215 + 73_072,
                    &mut sa,
                ),
                step(&mut b, &bus, &registry, &mut pb, 2, i * 215, &mut sb),
            ] {
                assert_ne!(report.status, ShareStatus::Ready);
                assert_eq!(report.paired_frames, 0);
                assert_eq!(report.queue_dropped, 0);
            }
        }
        let mut recovered = 0;
        for i in 0..100 {
            let (mut sa, mut sb) = ([0.125; 215], [-0.25; 215]);
            for report in [
                step(
                    &mut a,
                    &bus,
                    &registry,
                    &mut pa,
                    1,
                    1_000_000 + i * 215,
                    &mut sa,
                ),
                step(
                    &mut b,
                    &bus,
                    &registry,
                    &mut pb,
                    2,
                    1_000_000 + i * 215,
                    &mut sb,
                ),
            ] {
                if report.status == ShareStatus::Ready {
                    recovered += 1;
                }
                assert_eq!(report.queue_dropped, 0);
            }
        }
        assert!(recovered > 100);
    }
}
