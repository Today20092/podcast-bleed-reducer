//! Shared mono/stereo envelope application; worker failure always preserves input.
use crate::{gain::GainPolicy, worker::WorkerBridge};
use cleanslate_gate_detector::Event;
use nice_plug::prelude::{Buffer, ProcessMode};

pub(crate) struct Engine {
    worker: WorkerBridge,
    gain: GainPolicy,
    channels: usize,
    offline: bool,
    epoch: u64,
    position: u64,
    pending: Option<Event>,
    enabled: bool,
    fault: bool,
    pub(crate) minimum_gain: f32,
    recorder: crate::history::Recorder,
    rate: u32,
    delay: Vec<Vec<f32>>,
    delay_write: usize,
    lookahead: usize,
    fade_from: usize,
    fade_left: u32,
    fade_total: u32,
    /// 1 follows the gate. 0 is unity. Host bypass slews this over about 5 ms.
    bypass_mix: f32,
}

impl Engine {
    pub(crate) fn prepare(rate: u32, channels: u32, mode: ProcessMode) -> Result<Self, String> {
        let mut engine = Self {
            worker: WorkerBridge::prepare(rate, channels, mode)?,
            gain: GainPolicy::new(rate).map_err(str::to_owned)?,
            channels: channels as usize,
            offline: matches!(mode, ProcessMode::Offline),
            epoch: 0,
            position: 0,
            pending: None,
            enabled: false,
            fault: false,
            minimum_gain: 1.0,
            recorder: crate::history::Recorder::new(rate),
            rate,
            delay: vec![vec![0.0; (rate as usize) * 20 / 1000 + 8]; channels as usize],
            delay_write: 0,
            lookahead: 0,
            fade_from: 0,
            fade_left: 0,
            fade_total: 1,
            bypass_mix: 1.0,
        };
        engine.reset();
        Ok(engine)
    }

    pub(crate) fn reset(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.worker.reset(self.epoch);
        self.gain.reset(self.epoch);
        self.position = 0;
        self.pending = None;
        self.enabled = false;
        self.fault = false;
        self.recorder.reset();
        for line in &mut self.delay {
            line.fill(0.0);
        }
        self.delay_write = 0;
        self.fade_from = self.lookahead;
        self.fade_left = 0;
        self.fade_total = 1;
        self.bypass_mix = 1.0;
    }

    /// Samples of delay the host must report for this lookahead setting.
    pub(crate) fn latency_samples(rate: u32, lookahead_ms: f32) -> u32 {
        let cap = (rate as usize) * 20 / 1000 + 8;
        let requested = if lookahead_ms.is_finite() {
            (f64::from(rate) * f64::from(lookahead_ms.clamp(0.0, 20.0)) / 1000.0).round() as usize
        } else {
            0
        };
        requested.min(cap.saturating_sub(1)) as u32
    }

    fn update_lookahead(&mut self, lookahead_ms: f32) {
        let next = Self::latency_samples(self.rate, lookahead_ms) as usize;
        if next == self.lookahead {
            return;
        }
        // A fresh or reset line has not reached the host yet, so take the new
        // delay immediately. A change during playback crossfades for about 5 ms.
        if self.position == 0 && self.delay_write == 0 {
            self.lookahead = next;
            self.fade_from = next;
            self.fade_left = 0;
            return;
        }
        self.fade_from = self.lookahead;
        self.fade_total = (self.rate / 200).clamp(1, 256);
        self.fade_left = self.fade_total;
        self.lookahead = next;
    }

    /// Writes this frame into the lookahead line and replaces it with the delayed sample.
    fn apply_delay(&mut self, buffer: &mut Buffer, frame: usize, gain: f32) {
        let cap = self.delay.first().map(|line| line.len()).unwrap_or(1);
        let look = self.lookahead.min(cap.saturating_sub(1));
        let from = self.fade_from.min(cap.saturating_sub(1));
        let write = self.delay_write % cap;
        let read = (write + cap - look) % cap;
        let read_from = (write + cap - from) % cap;
        let mix = if self.fade_left > 0 && self.fade_total > 0 {
            self.fade_left as f32 / self.fade_total as f32
        } else {
            0.0
        };
        for (index, channel) in buffer.as_slice().iter_mut().enumerate() {
            let Some(line) = self.delay.get_mut(index) else {
                continue;
            };
            let current = channel[frame];
            let at_read = if read == write { current } else { line[read] };
            let at_from = if read_from == write { current } else { line[read_from] };
            line[write] = current;
            let delayed = if mix > 0.0 {
                at_from * mix + at_read * (1.0 - mix)
            } else {
                at_read
            };
            channel[frame] = delayed * gain;
        }
        if self.fade_left > 0 {
            self.fade_left -= 1;
        }
        self.delay_write = (write + 1) % cap;
    }

    fn delay_unity(&mut self, buffer: &mut Buffer, start: usize) {
        for frame in start..buffer.samples() {
            self.apply_delay(buffer, frame, 1.0);
        }
    }

    pub(crate) fn failed(&self) -> bool {
        self.fault || self.worker.failed()
    }

    pub(crate) fn current_activity(&self) -> Option<f32> {
        if self.failed() || !self.enabled {
            return None;
        }
        self.gain.current_activity(self.position.saturating_sub(1))
    }

    pub(crate) fn detection_state(&self) -> crate::DetectionState {
        if self.failed() {
            crate::DetectionState::Failed
        } else if self
            .gain
            .evidence_age(self.position.saturating_sub(1))
            .is_some()
        {
            crate::DetectionState::Current
        } else {
            crate::DetectionState::Waiting
        }
    }

    fn record_original(
        &mut self,
        buffer: &Buffer,
        start: usize,
        history: &crate::history::History,
    ) {
        for frame in start..buffer.samples() {
            let peak = buffer
                .as_slice_immutable()
                .iter()
                .fold(0.0_f32, |peak, channel| peak.max(channel[frame].abs()));
            self.recorder.observe(peak, peak, 1.0, history);
        }
    }

    pub(crate) fn process(
        &mut self,
        buffer: &mut Buffer,
        enabled: bool,
        range: f32,
        open: f32,
        hold_ms: f32,
        attack_ms: f32,
        release_ms: f32,
        level_bias: f32,
        lookahead_ms: f32,
        history: &crate::history::History,
        host_aligned: bool,
    ) {
        self.minimum_gain = 1.0;
        self.gain.set_range_db(range);
        self.gain.set_timing(open, hold_ms, attack_ms, release_ms);
        self.gain.set_level_bias(level_bias);
        self.update_lookahead(lookahead_ms);
        // The experimental host reports zero latency, so its bypass stays bit-exact.
        // The JUCE host reports this delay, so bypass keeps the same delay line.
        if !enabled && !host_aligned {
            if self.enabled {
                self.reset();
            }
            self.record_original(buffer, 0, history);
            return;
        }
        if enabled && !self.enabled && !host_aligned {
            self.reset();
        }
        self.enabled = enabled;
        if self.fault || self.worker.failed() {
            if host_aligned {
                self.delay_unity(buffer, 0);
            }
            self.record_original(buffer, 0, history);
            return;
        }
        if buffer.channels() != self.channels
            || buffer
                .as_slice_immutable()
                .iter()
                .any(|channel| channel.iter().any(|sample| !sample.is_finite()))
        {
            self.fault = true;
            if host_aligned {
                self.delay_unity(buffer, 0);
            }
            self.record_original(buffer, 0, history);
            return;
        }

        // A fixed sub-block bounds queue records and keeps host block size out
        // of detector framing. Only offline rendering may wait for inference.
        for start in (0..buffer.samples()).step_by(256) {
            let end = (start + 256).min(buffer.samples());
            let Some(processed_end) = self.position.checked_add((end - start) as u64) else {
                self.fault = true;
                if host_aligned {
                    self.delay_unity(buffer, start);
                }
                self.record_original(buffer, start, history);
                return;
            };
            let channels = buffer.as_slice_immutable();
            let right = (self.channels == 2).then(|| &channels[1][start..end]);
            if !self.worker.submit(&channels[0][start..end], right)
                || (self.offline && !self.worker.wait_until_processed(processed_end))
            {
                self.fault = true;
                if host_aligned {
                    self.delay_unity(buffer, start);
                }
                self.record_original(buffer, start, history);
                return;
            }
            if self.pending.is_none() {
                self.pending = self.worker.pop_event();
            }
            for frame in start..end {
                let input = buffer
                    .as_slice_immutable()
                    .iter()
                    .fold(0.0_f32, |peak, channel| peak.max(channel[frame].abs()));
                self.gain.note_peak(input);
                // Future evidence may arrive early during offline rendering;
                // never apply it before its actual source-sample availability.
                for _ in 0..32 {
                    let Some(event) = self
                        .pending
                        .filter(|event| event.available_at <= self.position)
                    else {
                        break;
                    };
                    self.gain.observe(event);
                    self.pending = self.worker.pop_event();
                }
                if self.worker.failed() {
                    self.fault = true;
                    if host_aligned {
                        self.delay_unity(buffer, frame);
                    }
                    self.record_original(buffer, frame, history);
                    return;
                }
                let wet = self.gain.next_gain(self.position, true);
                let gain = if host_aligned {
                    let aim = if enabled { 1.0 } else { 0.0 };
                    let step = 1.0 / ((self.rate as f32) * 0.005).max(1.0);
                    self.bypass_mix += (aim - self.bypass_mix).clamp(-step, step);
                    1.0 + (wet - 1.0) * self.bypass_mix
                } else {
                    wet
                };
                self.minimum_gain = self.minimum_gain.min(gain);
                self.apply_delay(buffer, frame, gain);
                let output = buffer
                    .as_slice_immutable()
                    .iter()
                    .fold(0.0_f32, |peak, channel| peak.max(channel[frame].abs()));
                self.recorder.observe(input, output, gain, history);
                self.position += 1;
            }
        }
    }
}
