//! Causal confidence-to-gain policy. No worker, audio storage or host calls.
use cleanslate_gate_detector::Event;

/// Engineering limits for an explicitly armed prototype, not preservation acceptance.
/// Laughter in 92 averaged approximately .009 and whispered speech .14; neither
/// threshold can certify their preservation. Bypass remains the product default.

// Prefer false-open/original audio to suppressing confidence near the .14 whisper mean.
const OPEN: f32 = 0.10;
const FLOOR: f32 = 0.501_187_2; // -6 dB, finite maximum prototype reduction.
/// Decibels above the pause floor that map the level ingredient to 1.
const LEVEL_FULL_DB: f32 = 12.0;
/// The floor may creep up inside this ratio, about +3 dB. A phrase above it holds.
const FLOOR_NEAR_RATIO: f32 = 1.412_537_5;
/// Envelope this far under the recent peak is a pause, about 15 dB.
const PAUSE_RATIO: f32 = 5.623_413;

pub struct GainPolicy {
    rate: u32,
    epoch: u64,
    evidence: Option<Event>,
    decision_activity: f32,
    fault: bool,
    open: bool,
    hold_until: u64,
    gain: f32,
    rise_remaining: u32,
    floor: f32,
    open_threshold: f32,
    close_threshold: f32,
    hold_samples: u64,
    attack_samples: u32,
    release_samples: u32,
    /// 0 keeps activity equal to the speech probability.
    level_bias: f32,
    env: f32,
    /// Recent peak. Falls slower than the envelope so a pause sits below it.
    crest: f32,
    noise_floor: f32,
    level_score: f32,
    level_ready: bool,
    env_coeff: f32,
    floor_rise: f32,
    floor_fall: f32,
    crest_fall: f32,
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
            decision_activity: 0.0,
            fault: false,
            open: true,
            hold_until: 0,
            gain: 1.0,
            rise_remaining: 0,
            floor: FLOOR,
            open_threshold: OPEN,
            close_threshold: OPEN,
            hold_samples: u64::from(rate) * 300 / 1000,
            attack_samples: rate / 200,
            release_samples: rate / 4,
            level_bias: 0.0,
            env: 0.0,
            crest: 0.0,
            noise_floor: 0.0,
            level_score: 0.0,
            level_ready: false,
            env_coeff: pole(rate, 10.0),
            floor_rise: pole(rate, 2_000.0),
            floor_fall: pole(rate, 30.0),
            crest_fall: pole(rate, 400.0),
        };
        policy.reset(0);
        Ok(policy)
    }

    /// Speech-only at 0. Higher values pull activity toward the noise-relative level.
    pub(crate) fn set_level_bias(&mut self, bias: f32) {
        self.level_bias = if bias.is_finite() {
            bias.clamp(0.0, 1.0)
        } else {
            0.0
        };
    }

    /// Causal peak against a floor learned from pauses.
    /// A drop of about 15 dB under the recent peak is a pause: the floor moves to
    /// that level in about 30 ms, including upward after an earlier digital silence.
    /// A phrase holds the floor. Inside +3 dB the floor can still creep over about 2 s.
    /// +12 dB over the floor maps to a level score of 1.
    /// A constant clip gain scales the envelope and the floor together.
    pub(crate) fn note_peak(&mut self, peak: f32) {
        if !peak.is_finite() {
            return;
        }
        let peak = peak.abs();
        if !self.level_ready {
            self.env = peak;
            self.crest = peak;
            self.noise_floor = peak.max(1.0e-8);
            self.level_score = 0.0;
            self.level_ready = true;
            return;
        }
        self.env += (peak - self.env) * self.env_coeff;
        let crest_coeff = if self.env > self.crest {
            self.env_coeff
        } else {
            self.crest_fall
        };
        self.crest += (self.env - self.crest) * crest_coeff;
        // A pause may lift a floor that latched onto digital silence.
        // A louder phrase must not become the room.
        let paused = self.env * PAUSE_RATIO < self.crest;
        let coeff = if self.env < self.noise_floor || paused {
            self.floor_fall
        } else if self.env <= self.noise_floor * FLOOR_NEAR_RATIO {
            self.floor_rise
        } else {
            0.0
        };
        self.noise_floor += (self.env - self.noise_floor) * coeff;
        self.noise_floor = self.noise_floor.max(1.0e-8);
        let ratio = (self.env / self.noise_floor).max(1.0e-8);
        let decibels = 20.0 * ratio.log10();
        self.level_score = (decibels / LEVEL_FULL_DB).clamp(0.0, 1.0);
    }

    pub(crate) fn level_score(&self) -> f32 {
        self.level_score
    }

    fn activity(&self, probability: f32) -> f32 {
        let bias = self.level_bias;
        if bias == 0.0 {
            probability
        } else {
            (probability + bias * (self.level_score - probability)).clamp(0.0, 1.0)
        }
    }

    pub(crate) fn set_timing(&mut self, open: f32, hold_ms: f32, attack_ms: f32, release_ms: f32) {
        self.open_threshold = if open.is_finite() {
            open.clamp(0.0, 1.0)
        } else {
            0.10
        };
        // The gold Voice line is the decision. A score under it closes.
        self.close_threshold = self.open_threshold;
        let samples = |ms: f32, min_ms: f32, max_ms: f32| {
            let ms = if ms.is_finite() {
                ms.clamp(min_ms, max_ms)
            } else {
                min_ms
            };
            (f64::from(self.rate) * f64::from(ms) / 1000.0).round() as u64
        };
        self.hold_samples = samples(hold_ms, 0.0, 1_000.0);
        self.attack_samples = samples(attack_ms, 1.0, 1_000.0).max(1) as u32;
        self.release_samples = samples(release_ms, 1.0, 1_000.0).max(1) as u32;
    }

    pub(crate) fn set_range_db(&mut self, range: f32) {
        self.floor = if range.is_finite() {
            if range >= 60.0 {
                0.0
            } else {
                10.0_f32.powf(-range.clamp(0.0, 60.0) / 20.0)
            }
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
        // Hold starts after speech. A quiet opening must be allowed to close.
        self.hold_until = 0;
        self.gain = 1.0;
        self.rise_remaining = 0;
        self.env = 0.0;
        self.crest = 0.0;
        self.noise_floor = 0.0;
        self.level_score = 0.0;
        self.level_ready = false;
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
        let activity = self.activity(event.probability);
        self.decision_activity = activity;
        if activity >= self.open_threshold {
            self.open = true;
            self.hold_until = event.available_at.saturating_add(self.hold_samples);
        } else if activity <= self.close_threshold {
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

    pub(crate) fn current_activity(&self, source_sample: u64) -> Option<f32> {
        self.evidence_age(source_sample)?;
        Some(self.decision_activity)
    }

    /// Call once per source sample; `enabled` requires explicit arm and no bypass.
    /// Fresh evidence expires at 100 ms. Its final 5 ms proactively restores unity.
    /// Otherwise, opening uses the configured attack and closing uses the configured
    /// release after hold. This policy adds no audio delay or host latency.
    pub fn next_gain(&mut self, source_sample: u64, enabled: bool) -> f32 {
        let expiry = u64::from(self.rate) / 10;
        let safety_samples = (expiry / 20).max(1);
        let Some(age) = self.evidence_age(source_sample) else {
            self.gain = 1.0;
            return self.gain;
        };
        let preserve = !enabled
            || self.open
            || source_sample < self.hold_until
            || age >= expiry - safety_samples;
        if preserve {
            let return_samples = if age >= expiry - safety_samples {
                safety_samples
            } else {
                u64::from(self.attack_samples)
            };
            if age >= expiry - safety_samples && self.rise_remaining > return_samples as u32 {
                self.rise_remaining = return_samples as u32;
            }
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
            self.gain =
                (self.gain - (1.0 - self.floor) / self.release_samples as f32).max(self.floor);
        }
        self.gain
    }
}

fn pole(rate: u32, ms: f32) -> f32 {
    let tau = rate as f32 * ms / 1000.0;
    1.0 - (-1.0 / tau).exp()
}
