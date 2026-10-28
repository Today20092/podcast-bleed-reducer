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
    }

    pub(crate) fn process(&mut self, buffer: &mut Buffer, enabled: bool) {
        if !enabled {
            if self.enabled {
                self.reset();
            }
            return;
        }
        if !self.enabled {
            self.reset();
            self.enabled = true;
        }
        if self.fault || self.worker.failed() {
            return;
        }
        if buffer.channels() != self.channels
            || buffer
                .as_slice_immutable()
                .iter()
                .any(|channel| channel.iter().any(|sample| !sample.is_finite()))
        {
            self.fault = true;
            return;
        }

        // A fixed sub-block bounds queue records and keeps host block size out
        // of detector framing. Only offline rendering may wait for inference.
        for start in (0..buffer.samples()).step_by(256) {
            let end = (start + 256).min(buffer.samples());
            let Some(processed_end) = self.position.checked_add((end - start) as u64) else {
                self.fault = true;
                return;
            };
            let channels = buffer.as_slice_immutable();
            let right = (self.channels == 2).then(|| &channels[1][start..end]);
            if !self.worker.submit(&channels[0][start..end], right)
                || (self.offline && !self.worker.wait_until_processed(processed_end))
            {
                self.fault = true;
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
                    return;
                }
                let gain = self.gain.next_gain(self.position, true);
                if gain != 1.0 {
                    for channel in buffer.as_slice() {
                        channel[frame] *= gain;
                    }
                }
                self.position += 1;
            }
        }
    }
}
