use crate::{
    ListenMode,
    calibration::{self, Label, ModelBank, ReferenceFrame, Session},
    probe::{Binding, Registry},
    shared_audio::SharedAudio,
};
use parking_lot::Mutex;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI64, AtomicU8, AtomicU32, Ordering::SeqCst},
};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum RuntimeStatus {
    Disabled,
    NeedGroup,
    NeedCalibration,
    ConfirmCalibration,
    Warmup,
    Processing,
    Restarting,
    UnsupportedClap,
    InvalidSettings,
}

pub struct GuiState {
    pub position: AtomicI64,
    pub playing: AtomicBool,
    pub capture: Mutex<Option<Arc<crate::session_mode::SessionCapture>>>,
    pub session: Mutex<Option<Arc<Session>>>,
    pub probability: AtomicU32,
    pub gain: AtomicU32,
    pub latency_ms: AtomicU32,
    pub status: AtomicU8,
    pub error: Mutex<Option<String>>,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            position: AtomicI64::new(-1),
            playing: AtomicBool::new(false),
            capture: Mutex::new(None),
            session: Mutex::new(None),
            probability: AtomicU32::new(1.0_f32.to_bits()),
            gain: AtomicU32::new(1.0_f32.to_bits()),
            latency_ms: AtomicU32::new(0),
            status: AtomicU8::new(0),
            error: Mutex::new(None),
        }
    }
    pub fn status_text(&self) -> &'static str {
        match self.status.load(SeqCst) {
            0 => "Processing off — original audio",
            1 => "Choose a group and two unique members",
            2 => "Capture examples and Learn — original preserved",
            3 => "Confirm saved calibration — original preserved",
            4 => "Warming up or waiting for synchronized evidence — original preserved",
            5 => "Processing from calibrated ownership",
            6 => "Waiting for host restart — stop and restart playback if needed",
            7 => "CLAP attenuation awaits export qualification — original preserved; use VST3",
            _ => "Invalid settings — original preserved",
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Decision {
    index: i64,
    valid: bool,
    owned: bool,
}

pub struct Runtime {
    pub enabled: bool,
    pub delay_frames: usize,
    rate: i64,
    channels: usize,
    delay: Vec<[f32; 2]>,
    delay_cursor: usize,
    expected: Option<i64>,
    binding: Option<Binding>,
    bank: Option<ModelBank>,
    decisions: [Decision; 128],
    next_frame: Option<i64>,
    owned: bool,
    own_scratch: Vec<[f32; 2]>,
    peer_scratch: Vec<[f32; 2]>,
    last_capture: Option<(u64, u64, i64)>,
}

pub struct RenderSettings {
    pub attenuation_db: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub listen: ListenMode,
}

impl RenderSettings {
    fn valid(&self) -> bool {
        self.attenuation_db.is_finite()
            && (-48.0..=0.0).contains(&self.attenuation_db)
            && self.attack_ms.is_finite()
            && (5.0..=80.0).contains(&self.attack_ms)
            && self.release_ms.is_finite()
            && (10.0..=150.0).contains(&self.release_ms)
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            enabled: false,
            delay_frames: 0,
            rate: 48000,
            channels: 2,
            delay: Vec::new(),
            delay_cursor: 0,
            expected: None,
            binding: None,
            bank: None,
            decisions: [Decision::default(); 128],
            next_frame: None,
            owned: true,
            own_scratch: vec![[0.0; 2]; 8192],
            peer_scratch: vec![[0.0; 2]; 8192],
            last_capture: None,
        }
    }

    pub fn activate(
        &mut self,
        sample_rate: f32,
        enabled: bool,
        channels: usize,
        max_buffer: usize,
    ) -> bool {
        if !sample_rate.is_finite()
            || !(8000.0..=192000.0).contains(&sample_rate)
            || sample_rate.fract() != 0.0
        {
            return false;
        }
        if !(1..=2).contains(&channels) || max_buffer == 0 || max_buffer > 8192 {
            return false;
        }
        self.rate = sample_rate as i64;
        self.channels = channels;
        self.enabled = enabled;
        self.delay_frames = if enabled {
            // One full host block plus analysis lag, maximum fades and frame rounding.
            (self.rate * 416 / 1000) as usize + max_buffer
        } else {
            0
        };
        self.delay = vec![[0.0; 2]; self.delay_frames.max(1)];
        self.reset_transport();
        true
    }

    pub fn reset_transport(&mut self) {
        self.delay.fill([0.0; 2]);
        self.delay_cursor = 0;
        self.expected = None;
        self.reset_decisions();
    }

    pub fn prepare(&mut self, start: Option<i64>, active: bool) {
        if active && start.is_some() && self.expected.is_some() && start != self.expected {
            self.reset_transport();
        }
    }

    fn reset_decisions(&mut self) {
        self.decisions.fill(Decision::default());
        self.next_frame = None;
        self.owned = true;
        self.last_capture = None;
    }

    fn frame_index(&self, sample: i64) -> i64 {
        (i128::from(sample) * 1000).div_euclid(i128::from(self.rate) * 32) as i64
    }
    fn bounds(&self, index: i64) -> Option<(i64, i64)> {
        let start = (i128::from(index) * i128::from(self.rate) * 32).div_euclid(1000);
        let end = ((i128::from(index) + 1) * i128::from(self.rate) * 32).div_euclid(1000);
        Some((start.try_into().ok()?, end.try_into().ok()?))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn analyze(
        &mut self,
        audio: &SharedAudio,
        registry: &Registry,
        binding: Option<Binding>,
        session: Option<&Session>,
        member: usize,
        start: Option<i64>,
        frames: usize,
        active: bool,
        gui: &GuiState,
    ) {
        let Some(session) = session else {
            return;
        };
        // Consume at most two fixed-size model copies; no Arc/model destruction in callbacks.
        for _ in 0..2 {
            let Some(bank) = session.models[member].pop() else {
                break;
            };
            self.bank = Some(bank);
            // A new classifier invalidates decisions, not the continuously received history.
            let history_cursor = self.next_frame;
            self.reset_decisions();
            self.next_frame = history_cursor;
        }
        if self.binding != binding {
            self.binding = binding;
            self.reset_decisions();
        }
        if !active {
            return;
        }
        let (Some(binding), Some(start)) = (binding, start) else {
            return;
        };
        let initial = self.frame_index(start);
        let initial = if self.bounds(initial).is_some_and(|bounds| bounds.0 < start) {
            initial + 1
        } else {
            initial
        };
        let first = *self.next_frame.get_or_insert(initial);
        let highest = self.frame_index(start.saturating_sub(self.rate * 96 / 1000));
        let mut index = first;
        for _ in 0..64 {
            if index > highest {
                break;
            }
            let Some((left, right)) = self.bounds(index) else {
                break;
            };
            let count = (right - left) as usize;
            if count == 0 || count > self.own_scratch.len() {
                break;
            }
            if !audio.read_pair(
                registry,
                binding,
                left,
                &mut self.own_scratch[..count],
                &mut self.peer_scratch[..count],
            ) {
                if right
                    <= start
                        .saturating_add(frames as i64)
                        .saturating_sub(self.delay_frames as i64)
                {
                    index += 1;
                    self.next_frame = Some(index);
                    continue;
                }
                break;
            }
            let channels = self.channels;
            let Some(mut features) = calibration::acoustic_features(
                &self.own_scratch[..count],
                &self.peer_scratch[..count],
                channels,
            ) else {
                index += 1;
                self.next_frame = Some(index);
                continue;
            };
            if member == 1 {
                features.swap(0, 1);
            }
            if member == 0 {
                session
                    .history
                    .push(crate::history::HistoryFrame::from_audio(
                        binding.epoch,
                        ReferenceFrame {
                            start: left,
                            end: right,
                            label: Label::None,
                            rate: (self.rate as f32).to_bits(),
                            channels,
                            features,
                        },
                        &self.own_scratch[..count],
                        &self.peer_scratch[..count],
                        channels,
                    ));
                let (generation, capture) = session.capture_key();
                if self.last_capture != Some((generation, capture, index)) {
                    session.capture_frame(ReferenceFrame {
                        start: left,
                        end: right,
                        label: Label::None,
                        rate: (self.rate as f32).to_bits(),
                        channels,
                        features,
                    });
                    self.last_capture = Some((generation, capture, index));
                }
            }
            if let Some(bank) = self.bank
                && bank.generation == session.generation.load(SeqCst)
                && bank.rate == (self.rate as f32).to_bits()
                && bank.channels == self.channels
                && session.confirmed.load(SeqCst)
                && let Some(probability) = bank.predictors[member].estimate(&features[member])
            {
                gui.probability
                    .store((probability as f32).to_bits(), SeqCst);
                if let Some(mask) = bank.truth_at(left) {
                    self.owned = mask & (1 << member) != 0;
                } else if probability >= 0.8 {
                    self.owned = true;
                } else if probability <= 0.2 {
                    self.owned = false;
                }
                self.decisions[index.rem_euclid(128) as usize] = Decision {
                    index,
                    valid: true,
                    owned: self.owned,
                };
            }
            index += 1;
            self.next_frame = Some(index);
        }
    }

    fn decision(&self, index: i64) -> Option<bool> {
        let decision = self.decisions[index.rem_euclid(128) as usize];
        (decision.valid && decision.index == index).then_some(decision.owned)
    }

    fn gain_at(&self, sample: i64, member: usize, settings: &RenderSettings) -> Option<f32> {
        let bank = self.bank?;
        if bank
            .truth_at(sample)
            .is_some_and(|mask| mask & (1 << member) != 0)
        {
            return Some(1.0);
        }
        let index = self.frame_index(sample);
        if self.decision(index)? {
            return Some(1.0);
        }
        let close = (f64::from(settings.release_ms) * self.rate as f64 / 1000.0) as i64;
        let open = (f64::from(settings.attack_ms) * self.rate as f64 / 1000.0) as i64;
        let horizon = (close + open).div_euclid(self.rate * 32 / 1000) + 2;
        let mut previous_end = None;
        for offset in 1..=horizon {
            match self.decision(index - offset) {
                Some(false) => {}
                Some(true) | None => {
                    previous_end = Some(self.bounds(index - offset)?.1);
                    break;
                }
            }
        }
        let mut next_start = None;
        for offset in 1..=horizon {
            match self.decision(index + offset) {
                Some(false) => {}
                Some(true) => {
                    next_start = Some(self.bounds(index + offset)?.0);
                    break;
                }
                None => return None,
            }
        }
        let mut close_length = close;
        let mut open_length = open;
        if let (Some(previous), Some(next)) = (previous_end, next_start) {
            let gap = (next - previous).max(1);
            if gap < close + open {
                close_length = gap * close / (close + open);
                open_length = gap - close_length;
            }
        }
        let floor = 10.0_f64.powf(f64::from(settings.attenuation_db) / 20.0);
        let smooth = |t: f64| {
            let t = t.clamp(0.0, 1.0);
            t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
        };
        let mut gain = floor;
        if let Some(previous) = previous_end
            && sample < previous + close_length
            && close_length > 0
        {
            gain = 1.0 + (floor - 1.0) * smooth((sample - previous) as f64 / close_length as f64);
        }
        if let Some(next) = next_start
            && sample > next - open_length
            && open_length > 0
        {
            gain = gain.max(
                floor
                    + (1.0 - floor)
                        * smooth((sample - (next - open_length)) as f64 / open_length as f64),
            );
        }
        Some(gain.clamp(0.0, 1.0) as f32)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        channels: &mut [&mut [f32]],
        start: Option<i64>,
        active: bool,
        session: Option<&Session>,
        member: usize,
        settings: RenderSettings,
        allow_attenuation: bool,
        gui: &GuiState,
    ) {
        let frames = channels.first().map_or(0, |channel| channel.len());
        let current = session.is_some_and(|session| {
            session.confirmed.load(SeqCst)
                && self.bank.is_some_and(|bank| {
                    bank.generation == session.generation.load(SeqCst)
                        && bank.rate == (self.rate as f32).to_bits()
                        && bank.channels == self.channels
                })
        });
        let valid = settings.valid() && current && active && allow_attenuation && self.enabled;
        let status = if !self.enabled {
            RuntimeStatus::Disabled
        } else if !allow_attenuation {
            RuntimeStatus::UnsupportedClap
        } else if !settings.valid() {
            RuntimeStatus::InvalidSettings
        } else if session.is_none() {
            RuntimeStatus::NeedGroup
        } else if self.bank.is_none() {
            RuntimeStatus::NeedCalibration
        } else if !current {
            RuntimeStatus::ConfirmCalibration
        } else {
            RuntimeStatus::Warmup
        };
        gui.status.store(status as u8, SeqCst);
        let mut last_gain = 1.0_f32;
        for offset in 0..frames {
            let mut input = [0.0; 2];
            for c in 0..channels.len().min(2) {
                input[c] = channels[c][offset];
            }
            let original = if self.delay_frames == 0 {
                input
            } else {
                let old = self.delay[self.delay_cursor];
                self.delay[self.delay_cursor] = input;
                self.delay_cursor = (self.delay_cursor + 1) % self.delay_frames;
                old
            };
            let time = start
                .and_then(|start| start.checked_add(offset as i64))
                .and_then(|time| time.checked_sub(self.delay_frames as i64));
            let still_current = session.is_some_and(|session| {
                session.confirmed.load(SeqCst)
                    && self
                        .bank
                        .is_some_and(|bank| bank.generation == session.generation.load(SeqCst))
            });
            let computed = if valid && still_current {
                time.and_then(|time| self.gain_at(time, member, &settings))
            } else {
                None
            };
            let gain = computed.unwrap_or(1.0);
            if computed.is_some() {
                gui.status.store(RuntimeStatus::Processing as u8, SeqCst);
            }
            last_gain = gain;
            for c in 0..channels.len().min(2) {
                channels[c][offset] = match settings.listen {
                    ListenMode::Original => original[c],
                    ListenMode::Processed => {
                        if gain == 1.0 {
                            original[c]
                        } else {
                            original[c] * gain
                        }
                    }
                    ListenMode::Removed => {
                        original[c]
                            - if gain == 1.0 {
                                original[c]
                            } else {
                                original[c] * gain
                            }
                    }
                };
            }
        }
        gui.gain.store(last_gain.to_bits(), SeqCst);
        self.expected = start.and_then(|start| start.checked_add(frames as i64));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::{CalibrationDocument, tests::rows};
    use std::sync::atomic::AtomicBool;

    #[test]
    fn missing_transport_clock_preserves_continuous_delayed_original() {
        let mut runtime = Runtime::new();
        assert!(runtime.activate(8000.0, true, 1, 256));
        let gui = GuiState::new();
        let mut reference = std::collections::VecDeque::from(vec![0.0_f32; runtime.delay_frames]);
        for block in 0..24 {
            let start = if (12..15).contains(&block) {
                None
            } else {
                Some(block * 256)
            };
            runtime.prepare(start, true);
            let mut audio = (0..256)
                .map(|i| ((block * 256 + i) as f32 * 0.01).sin())
                .collect::<Vec<_>>();
            let expected = audio
                .iter()
                .map(|&value| {
                    reference.push_back(value);
                    reference.pop_front().unwrap()
                })
                .collect::<Vec<_>>();
            runtime.render(
                &mut [&mut audio],
                start,
                true,
                None,
                0,
                settings(ListenMode::Original),
                false,
                &gui,
            );
            assert_eq!(
                audio, expected,
                "missing clock metadata must not discard received audio"
            );
        }
    }

    fn settings(listen: ListenMode) -> RenderSettings {
        RenderSettings {
            attenuation_db: -12.0,
            attack_ms: 15.0,
            release_ms: 100.0,
            listen,
        }
    }

    #[test]
    fn delayed_fallback_preserves_samples_tail_and_seek_boundaries() {
        for rate in [44100.0, 48000.0, 192000.0] {
            let mut runtime = Runtime::new();
            assert!(runtime.activate(rate, true, 1, 1024));
            let delay = runtime.delay_frames;
            let signal: Vec<f32> = (0..delay + 6000).map(|i| (i as f32 * 0.13).sin()).collect();
            let mut output = Vec::new();
            let gui = GuiState::new();
            let mut start = 0;
            while start < signal.len() + delay {
                let count = 257.min(signal.len() + delay - start);
                let mut block: Vec<f32> = (start..start + count)
                    .map(|i| signal.get(i).copied().unwrap_or(0.0))
                    .collect();
                let active = start < signal.len();
                runtime.prepare(Some(start as i64), active);
                runtime.render(
                    &mut [&mut block],
                    Some(start as i64),
                    active,
                    None,
                    0,
                    settings(ListenMode::Processed),
                    false,
                    &gui,
                );
                output.extend(block);
                start += count;
            }
            assert!(output[..delay].iter().all(|&v| v == 0.0));
            assert_eq!(&output[delay..delay + signal.len()], signal.as_slice());
            runtime.prepare(Some(900000), true);
            let mut block = [1.0; 64];
            runtime.render(
                &mut [&mut block],
                Some(900000),
                true,
                None,
                0,
                settings(ListenMode::Processed),
                false,
                &gui,
            );
            assert_eq!(
                block, [0.0; 64],
                "A seek must not leak pre-seek delayed samples"
            );
        }
    }

    #[test]
    fn finite_fades_preserve_truth_and_unknown_and_monitoring_reconstructs_original() {
        let document = CalibrationDocument::fit(&rows(), &AtomicBool::new(false)).unwrap();
        let session = Session::new();
        session.confirmed.store(true, SeqCst);
        let mut runtime = Runtime::new();
        runtime.activate(48000.0, true, 1, 1024);
        // Fixtures are stereo; the runtime validates the matching input layout.
        runtime.channels = 2;
        runtime.bank = Some(document.compile(session.generation.load(SeqCst)).unwrap());
        for index in 270_i64..390 {
            runtime.decisions[index.rem_euclid(128) as usize] = Decision {
                index,
                valid: true,
                owned: !(300..330).contains(&index),
            };
        }
        let cfg = settings(ListenMode::Processed);
        let start = 300 * 1536;
        let end = 330 * 1536;
        assert_eq!(runtime.gain_at(start, 0, &cfg), Some(1.0));
        let floor = 10.0_f32.powf(-12.0 / 20.0);
        assert!((runtime.gain_at(start + 4800, 0, &cfg).unwrap() - floor).abs() < 1e-6);
        assert_eq!(runtime.gain_at(end, 0, &cfg), Some(1.0));
        for sample in start..end {
            let gain = runtime.gain_at(sample, 0, &cfg).unwrap();
            assert!((floor..=1.0).contains(&gain));
        }
        assert_eq!(runtime.gain_at(600 * 1536, 0, &cfg), None);
        // Explicit human speech remains exact unity even if an inferred decision disagrees.
        runtime.decisions[60] = Decision {
            index: 60,
            valid: true,
            owned: false,
        };
        assert_eq!(runtime.gain_at(60 * 1536, 0, &cfg), Some(1.0));
        let mut outputs = Vec::new();
        for mode in [
            ListenMode::Original,
            ListenMode::Processed,
            ListenMode::Removed,
        ] {
            runtime.delay.fill([0.75, -0.25]);
            runtime.delay_cursor = 0;
            let mut left = vec![0.0; 1024];
            let mut right = vec![0.0; 1024];
            runtime.render(
                &mut [&mut left, &mut right],
                Some(start + 6000 + runtime.delay_frames as i64),
                true,
                Some(&session),
                0,
                settings(mode),
                true,
                &GuiState::new(),
            );
            outputs.push([left, right]);
        }
        for ((original, processed), removed) in outputs[0]
            .iter()
            .flatten()
            .zip(outputs[1].iter().flatten())
            .zip(outputs[2].iter().flatten())
        {
            assert!((processed + removed - original).abs() < 1e-7);
        }
        assert!((outputs[1][0][0] - 0.75 * floor).abs() < 1e-6);
        session.generation.fetch_add(1, SeqCst);
        runtime.delay.fill([0.75, -0.25]);
        let mut left = [0.0; 16];
        let mut right = [0.0; 16];
        runtime.render(
            &mut [&mut left, &mut right],
            Some(start + 6000 + runtime.delay_frames as i64),
            true,
            Some(&session),
            0,
            settings(ListenMode::Processed),
            true,
            &GuiState::new(),
        );
        assert_eq!(left, [0.75; 16], "Stale calibration must preserve original");
    }

    #[test]
    fn paired_streaming_preserves_speech_and_attenuates_absence_across_host_blocks() {
        use crate::{probe::Probe, shared_audio::bus};
        const RATE: usize = 44_100;
        const SOURCE_START: i64 = 1_000_000;
        let labels = [Label::Room, Label::Member1, Label::Member2, Label::Both];
        let mut training = rows();
        for row in &mut training {
            row.rate = (RATE as f32).to_bits();
        }
        let document = CalibrationDocument::fit(&training, &AtomicBool::new(false)).unwrap();
        // Held-out variation, separated from all reference intervals: predictions must drive gain.
        let mut input = [Vec::new(), Vec::new()];
        for label in labels {
            let (a, b) = crate::calibration::tests::signal(label, 67, 2);
            for i in 0..RATE * 2 {
                input[0].push(a[i % a.len()]);
                input[1].push(b[i % b.len()]);
            }
        }
        for block in [512, 8192] {
            let registry = Registry::new();
            let session = Session::new();
            let params = [
                crate::FlProbe::default().params.clone(),
                crate::FlProbe::default().params.clone(),
            ];
            for (member, params) in params.iter().enumerate() {
                *params.saved.lock() = Some(document.clone());
                session
                    .bind(member, params, (RATE as f32).to_bits(), 2)
                    .unwrap();
            }
            session.confirm_saved().unwrap();
            assert!(session.snapshot().confirmed);
            let mut probes = [Probe::new(), Probe::new()];
            let mut audio = [SharedAudio::for_processing(), SharedAudio::for_processing()];
            let mut runtimes = [Runtime::new(), Runtime::new()];
            for runtime in &mut runtimes {
                assert!(runtime.activate(RATE as f32, true, 2, block));
            }
            let delay = runtimes[0].delay_frames;
            let mut output = [Vec::new(), Vec::new()];
            let mut publication_offset = None;
            for offset in (0..input[0].len() + delay).step_by(block) {
                if publication_offset.is_none() && offset > RATE * 3 {
                    for queue in &session.models {
                        queue
                            .force_push(document.compile(session.generation.load(SeqCst)).unwrap());
                    }
                    publication_offset = Some(offset);
                }
                let count = block.min(input[0].len() + delay - offset);
                let start = SOURCE_START + offset as i64;
                // Match a sequential host: each instance publishes, analyzes and renders in order.
                for member in 0..2 {
                    let mut left: Vec<_> = (offset..offset + count)
                        .map(|i| input[member].get(i).map_or(0.0, |sample| sample[0]))
                        .collect();
                    let mut right: Vec<_> = (offset..offset + count)
                        .map(|i| input[member].get(i).map_or(0.0, |sample| sample[1]))
                        .collect();
                    let channels = &mut [&mut left[..], &mut right[..]];
                    probes[member].observe(
                        &registry,
                        1,
                        member as i32 + 1,
                        Some(start),
                        count as u64,
                        (RATE as f32).to_bits(),
                        true,
                    );
                    let binding = probes[member].binding(&registry);
                    audio[member].process(
                        bus(),
                        &registry,
                        binding,
                        Some(start),
                        (RATE as f32).to_bits(),
                        channels,
                        true,
                    );
                    let gui = GuiState::new();
                    runtimes[member].prepare(Some(start), true);
                    runtimes[member].analyze(
                        &audio[member],
                        &registry,
                        binding,
                        Some(&session),
                        member,
                        Some(start),
                        count,
                        true,
                        &gui,
                    );
                    runtimes[member].render(
                        channels,
                        Some(start),
                        true,
                        Some(&session),
                        member,
                        RenderSettings {
                            attenuation_db: -12.0,
                            attack_ms: 80.0,
                            release_ms: 150.0,
                            listen: ListenMode::Processed,
                        },
                        binding.is_some_and(|binding| registry.is_current(binding)),
                        &gui,
                    );
                    output[member].extend_from_slice(&left);
                }
            }
            let floor = 10.0_f32.powf(-12.0 / 20.0);
            let history = session.history.snapshot();
            assert!(history.frames.len() > 100);
            assert!(
                history
                    .frames
                    .windows(2)
                    .all(|pair| pair[0].reference.end == pair[1].reference.start),
                "Publishing a learned model must not leave a hole in source history"
            );
            for (section, label) in labels.into_iter().enumerate() {
                for member in 0..2 {
                    let owned = matches!(label, Label::Both)
                        || (label == Label::Member1 && member == 0)
                        || (label == Label::Member2 && member == 1);
                    let expected_gain = if owned { 1.0 } else { floor };
                    // Inspect every interior sample to catch periodic dry gaps at callback edges.
                    for i in section * RATE * 2 + RATE / 2..section * RATE * 2 + RATE * 3 / 2 {
                        // New models preserve original audio until decisions/fades warm up.
                        if publication_offset.is_some_and(|t| {
                            i >= t.saturating_sub(delay + RATE / 4) && i < t + RATE / 4
                        }) {
                            continue;
                        }
                        let expected = input[member][i][0] * expected_gain;
                        assert!(
                            (output[member][i + delay] - expected).abs() < 1e-6,
                            "block {block}, {label:?}, member {member}, sample {i}: got {}, expected {expected}",
                            output[member][i + delay],
                        );
                    }
                }
            }
            for probe in &mut probes {
                probe.reset(&registry);
            }
        }
    }
}
