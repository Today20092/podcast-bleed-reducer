//! Causal confidence-to-gain policy. No worker, audio storage or host calls.
use cleanslate_gate_detector::Event;

/// Engineering limits for an explicitly armed prototype, not preservation acceptance.
/// Laughter in 92 averaged approximately .009 and whispered speech .14; neither
/// threshold can certify their preservation. Bypass remains the product default.
const CLOSE: f32 = 0.01;
// Prefer false-open/original audio to suppressing confidence near the .14 whisper mean.
const OPEN: f32 = 0.10;
const FLOOR: f32 = 0.501_187_2; // -6 dB, finite maximum prototype reduction.

pub struct GainPolicy {
    rate: u32,
    epoch: u64,
    evidence: Option<Event>,
    fault: bool,
    open: bool,
    hold_until: u64,
    gain: f32,
    rise_remaining: u32,
    floor: f32,
}

impl GainPolicy {
    pub fn new(rate: u32) -> Result<Self, &'static str> {
        if ![44_100, 48_000, 96_000].contains(&rate) {
            return Err("unsupported Gate sample rate");
        }
        let mut policy = Self {
            rate,
            epoch: 0,
            evidence: None,
            fault: false,
            open: true,
            hold_until: 0,
            gain: 1.0,
            rise_remaining: 0,
            floor: FLOOR,
        };
        policy.reset(0);
        Ok(policy)
    }

    pub(crate) fn set_range_db(&mut self, range: f32) {
        self.floor = if range.is_finite() {
            10.0_f32.powf(-range.clamp(0.0, 6.0) / 20.0)
        } else {
            1.0
        };
        self.gain = self.gain.max(self.floor);
    }

    /// A new source/rate/reset epoch starts at original audio, not the old gain.
    pub fn reset(&mut self, epoch: u64) {
        self.epoch = epoch;
        self.evidence = None;
        self.fault = false;
        self.open = true;
        self.hold_until = u64::from(self.rate) * 300 / 1000;
        self.gain = 1.0;
        self.rise_remaining = 0;
    }

    /// Accept copied evidence in source order. Faults latch until a stream reset.
    pub fn observe(&mut self, event: Event) {
        let expected_end = self
            .evidence
            .map_or(Some(512), |old| old.end_16k.checked_add(512));
        let expected_available = event
            .end_16k
            .checked_mul(u64::from(self.rate))
            .map(|value| value.div_ceil(16_000));
        if self.fault
            || event.epoch != self.epoch
            || !event.probability.is_finite()
            || !(0.0..=1.0).contains(&event.probability)
            || Some(event.end_16k) != expected_end
            || Some(event.available_at) != expected_available
        {
            // Invalid evidence must preserve exactly. This deterministic step is
            // deliberately not claimed click-free or same-source continuous.
            self.fault = true;
            self.evidence = None;
            self.gain = 1.0;
            return;
        }
        if event.probability >= OPEN {
            self.open = true;
            self.hold_until = event
                .available_at
                .saturating_add(u64::from(self.rate) * 300 / 1000);
        } else if event.probability <= CLOSE {
            self.open = false;
        }
        self.evidence = Some(event);
    }

    /// Only accepted current-epoch evidence survives observe/reset validation.
    pub(crate) fn evidence_age(&self, source_sample: u64) -> Option<u64> {
        source_sample
            .checked_sub(self.evidence?.available_at)
            .filter(|age| *age < u64::from(self.rate) / 10)
    }

    /// Call once per source sample; `enabled` requires explicit arm and no bypass.
    /// Fresh evidence expires at 100 ms. Its final 5 ms proactively restores unity
    /// while still valid. Ordinary disable/open returns within 5 ms; close takes
    /// 250 ms after the 300 ms hold. This adds no audio delay or host latency.
    pub fn next_gain(&mut self, source_sample: u64, enabled: bool) -> f32 {
        let expiry = u64::from(self.rate) / 10;
        let return_samples = u64::from(self.rate) / 200;
        let Some(age) = self.evidence_age(source_sample) else {
            self.gain = 1.0;
            return self.gain;
        };
        let preserve = !enabled
            || self.open
            || source_sample < self.hold_until
            || age >= expiry - return_samples;
        if preserve {
            if self.rise_remaining == 0 {
                self.rise_remaining = return_samples as u32;
            }
            if self.rise_remaining == 1 || self.gain == 1.0 {
                self.gain = 1.0;
                self.rise_remaining = 0;
            } else {
                self.gain += (1.0 - self.gain) / self.rise_remaining as f32;
                self.rise_remaining -= 1;
            }
        } else {
            self.rise_remaining = 0;
            self.gain = (self.gain - (1.0 - self.floor) / (self.rate / 4) as f32).max(self.floor);
        }
        self.gain
    }
}
