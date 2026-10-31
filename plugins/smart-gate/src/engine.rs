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
    }

    pub(crate) fn failed(&self) -> bool {
        self.fault || self.worker.failed()
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
        history: &crate::history::History,
    ) {
        self.minimum_gain = 1.0;
        self.gain.set_range_db(range);
        if !enabled {
            if self.enabled {
                self.reset();
            }
            self.record_original(buffer, 0, history);
            return;
        }
        if !self.enabled {
            self.reset();
            self.enabled = true;
        }
        if self.fault || self.worker.failed() {
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
            self.record_original(buffer, 0, history);
            return;
        }

        // A fixed sub-block bounds queue records and keeps host block size out
        // of detector framing. Only offline rendering may wait for inference.
        for start in (0..buffer.samples()).step_by(256) {
            let end = (start + 256).min(buffer.samples());
            let Some(processed_end) = self.position.checked_add((end - start) as u64) else {
                self.fault = true;
                self.record_original(buffer, start, history);
                return;
            };
            let channels = buffer.as_slice_immutable();
            let right = (self.channels == 2).then(|| &channels[1][start..end]);
            if !self.worker.submit(&channels[0][start..end], right)
                || (self.offline && !self.worker.wait_until_processed(processed_end))
            {
                self.fault = true;
                self.record_original(buffer, start, history);
                return;
            }
            if self.pending.is_none() {
                self.pending = self.worker.pop_event();
            }
            for frame in start..end {
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
                    self.record_original(buffer, frame, history);
                    return;
                }
                let gain = self.gain.next_gain(self.position, true);
                self.minimum_gain = self.minimum_gain.min(gain);
                let input = buffer
                    .as_slice_immutable()
                    .iter()
                    .fold(0.0_f32, |peak, channel| peak.max(channel[frame].abs()));
                if gain != 1.0 {
                    for channel in buffer.as_slice() {
                        channel[frame] *= gain;
                    }
                }
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
