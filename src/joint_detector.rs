//! Joint evidence is prepared once; calibration and controls only derive decisions.
use crate::prepared_analysis::{DetectorFacts, check_cancelled};
use crate::{AutomixParameters, Decision, OwnershipSettings, Region, RenderBlocker};
use std::sync::atomic::AtomicBool;

const FRAME: usize = 1536;
const DECIMATION: usize = 6;
const BAND_FRAME: usize = FRAME / DECIMATION;
const BLOCK_FRAMES: usize = 8;
const MAX_LAG: isize = 96; // 12 ms at 8 kHz, either polarity and direction.

#[derive(Debug)]
struct PairFacts {
    a: usize,
    b: usize,
    explained: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
struct Leakage {
    level_db: f32,
    residual_db: f32,
}

#[derive(Debug)]
pub(crate) struct JointFacts {
    levels: Vec<Vec<f32>>,
    floors: Vec<f32>,
    pairs: Vec<PairFacts>,
    leakage: Vec<Vec<Option<Leakage>>>,
}

impl JointFacts {
    pub(crate) fn retained_bytes(&self) -> usize {
        (self.levels.iter().map(Vec::capacity).sum::<usize>()
            + self.floors.capacity()
            + self
                .pairs
                .iter()
                .map(|p| p.explained.capacity())
                .sum::<usize>())
            * std::mem::size_of::<f32>()
            + self
                .leakage
                .iter()
                .map(|v| v.capacity() * std::mem::size_of::<Option<Leakage>>())
                .sum::<usize>()
    }
    pub(crate) fn prepare(
        tracks: &[&[f32]],
        cancelled: Option<&AtomicBool>,
    ) -> Result<Self, RenderBlocker> {
        let frames = tracks[0].len().div_ceil(FRAME);
        let mut levels = vec![Vec::with_capacity(frames); tracks.len()];
        let mut pairs: Vec<_> = (0..tracks.len())
            .flat_map(|a| {
                (a + 1..tracks.len()).map(move |b| PairFacts {
                    a,
                    b,
                    explained: Vec::with_capacity(frames),
                })
            })
            .collect();
        let mut filters: Vec<_> = tracks.iter().map(|_| BandFilter::new()).collect();
        // Only a 256 ms filtered block is live. No full-session sidechain PCM.
        for start in (0..tracks[0].len()).step_by(FRAME * BLOCK_FRAMES) {
            check_cancelled(cancelled)?;
            let end = (start + FRAME * BLOCK_FRAMES).min(tracks[0].len());
            let band: Vec<Vec<f32>> = tracks
                .iter()
                .zip(&mut filters)
                .enumerate()
                .map(|(track, (samples, filter))| {
                    let filtered: Vec<_> = samples[start..end]
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &x)| {
                            let y = filter.sample(x);
                            (i % DECIMATION == 0).then_some(y)
                        })
                        .collect();
                    levels[track].extend(filtered.chunks(BAND_FRAME).map(|frame| {
                        db(frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32)
                    }));
                    filtered
                })
                .collect();
            if band.iter().flatten().any(|x| !x.is_finite()) {
                return Err(RenderBlocker(
                    "speech-band preparation produced nonfinite evidence".into(),
                ));
            }
            for pair in &mut pairs {
                check_cancelled(cancelled)?;
                let a = &band[pair.a];
                let b = &band[pair.b];
                // ponytail: one lag per 256 ms block; faster-moving microphones
                // need shorter blocks or multiple lag candidates.
                let lag = (-MAX_LAG..=MAX_LAG)
                    .map(|lag| (lag, correlation(a, b, 0, a.len(), lag, 16)))
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                    .map_or(0, |(lag, _)| lag);
                for frame in 0..a.len().div_ceil(BAND_FRAME) {
                    let left = (frame * BAND_FRAME).saturating_sub(BAND_FRAME);
                    let right = ((frame + 2) * BAND_FRAME).min(a.len());
                    let explained = (lag - 1..=lag + 1)
                        .map(|l| correlation(a, b, left, right, l, 1))
                        .fold(0.0_f32, f32::max);
                    pair.explained.push(explained);
                }
            }
        }
        let floors = levels
            .iter()
            .map(|v| quantile(v.clone(), 0.1).max(-120.0))
            .collect();
        if levels.iter().flatten().any(|x| !x.is_finite()) {
            return Err(RenderBlocker("speech-band levels are nonfinite".into()));
        }
        let mut result = Self {
            levels,
            floors,
            pairs,
            leakage: vec![vec![None; tracks.len()]; tracks.len()],
        };
        for pair in &result.pairs {
            for (source, target) in [(pair.a, pair.b), (pair.b, pair.a)] {
                let selected = (0..frames).filter(|&frame| {
                    result.levels[source][frame] > result.floors[source] + 6.0
                        && result.levels[target][frame] - result.levels[source][frame] < -8.0
                        && pair.explained[frame] >= 0.25
                });
                result.leakage[source][target] = result.learn(pair, source, target, selected);
            }
        }
        Ok(result)
    }

    fn learn(
        &self,
        pair: &PairFacts,
        source: usize,
        target: usize,
        frames: impl Iterator<Item = usize>,
    ) -> Option<Leakage> {
        let mut levels = Vec::new();
        let mut residuals = Vec::new();
        for frame in frames {
            let difference = self.levels[target][frame] - self.levels[source][frame];
            levels.push(difference);
            residuals.push(difference + db((1.0 - pair.explained[frame]).max(1e-5)));
        }
        (levels.len() >= 3).then(|| Leakage {
            level_db: quantile(levels, 0.5),
            residual_db: quantile(residuals, 0.5),
        })
    }

    pub(crate) fn regions(
        &self,
        facts: &DetectorFacts,
        parameters: AutomixParameters,
        settings: &OwnershipSettings,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Vec<Region>, RenderBlocker> {
        check_cancelled(cancelled)?;
        let frames = self.levels[0].len();
        let tracks = self.levels.len();
        let mut floors = self.floors.clone();
        let mut leakage = self.leakage.clone();
        let mut room = vec![Vec::new(); tracks];
        let mut solo = vec![Vec::new(); tracks];
        // Engine/cache boundaries validate source identity and ranges before this call.
        for example in &settings.examples {
            let selected = example.start.div_ceil(FRAME)..example.end / FRAME;
            match example.kind {
                crate::CalibrationKind::RoomTone => {
                    room[example.track].extend(selected.map(|f| self.levels[example.track][f]))
                }
                crate::CalibrationKind::SoloSpeech => solo[example.track].extend(selected),
            }
        }
        for track in 0..tracks {
            if !room[track].is_empty() {
                floors[track] = quantile(std::mem::take(&mut room[track]), 0.5).max(-120.0);
            }
            solo[track].sort_unstable();
            solo[track].dedup();
            if !solo[track].is_empty()
                && solo[track]
                    .iter()
                    .filter(|&&f| self.levels[track][f] > -100.0)
                    .count()
                    < 3
            {
                return Err(RenderBlocker(
                    "solo-speech example contains no usable speech-band energy".into(),
                ));
            }
        }
        for pair in &self.pairs {
            for (source, target) in [(pair.a, pair.b), (pair.b, pair.a)] {
                if !solo[source].is_empty() {
                    // Do not treat unrelated audible sound in a declared solo
                    // example as the room's tolerated copy residual.
                    let audible: Vec<_> = solo[source]
                        .iter()
                        .copied()
                        .filter(|&f| {
                            self.levels[source][f] > -100.0
                                && self.levels[target][f] > floors[target] + 6.0
                        })
                        .map(|f| pair.explained[f])
                        .collect();
                    if audible.len() >= 3 && quantile(audible, 0.5) < 0.1 {
                        return Err(RenderBlocker(format!(
                            "Solo-speech example for track {} does not show clear copied speech on track {}. Choose a clearer solo interval or remove this example.",
                            source + 1,
                            target + 1
                        )));
                    }
                    leakage[source][target] = self.learn(
                        pair,
                        source,
                        target,
                        solo[source]
                            .iter()
                            .copied()
                            .filter(|&f| self.levels[source][f] > -100.0),
                    );
                    // A known direct microphone overrides the opposite automatic level ordering.
                    if solo[target].is_empty() {
                        leakage[target][source] = None;
                    }
                }
            }
        }
        let mut active = vec![vec![false; frames]; tracks];
        let mut copied = vec![vec![false; frames]; tracks];
        let mut speech = vec![false; frames];
        for frame in 0..frames {
            if frame % 1024 == 0 {
                check_cancelled(cancelled)?;
            }
            speech[frame] =
                (0..tracks).any(|t| facts.probabilities[t][frame] >= parameters.speech_sensitivity);
            for track in 0..tracks {
                active[track][frame] = facts.probabilities[track][frame]
                    >= parameters.speech_sensitivity
                    && self.levels[track][frame]
                        > floors[track] + parameters.bleed_discrimination_db;
            }
            for pair in &self.pairs {
                for (source, target) in [(pair.a, pair.b), (pair.b, pair.a)] {
                    if let Some(model) = leakage[source][target] {
                        let difference = self.levels[target][frame] - self.levels[source][frame];
                        let residual = difference + db((1.0 - pair.explained[frame]).max(1e-5));
                        // Unlabelled overlap cannot teach its own suppression threshold.
                        // Only a declared solo example can establish acoustic residual.
                        let residual_is_safe = !solo[source].is_empty()
                            || pair.explained[frame] >= 0.9999
                            || self.levels[source][frame] + residual
                                <= floors[target] + parameters.bleed_discrimination_db;
                        // Cross-microphone speech alone is not a target voice.
                        // A correlated copy with excess independent residual is:
                        // preserve it even when the quieter voice misses its own VAD.
                        let source_has_voice = facts.probabilities[source][frame]
                            >= parameters.speech_sensitivity
                            && self.levels[source][frame]
                                > floors[source] + parameters.bleed_discrimination_db;
                        if source_has_voice && pair.explained[frame] >= 0.25
                            && self.levels[source][frame] + residual > floors[target] + parameters.bleed_discrimination_db
                            // A rough acoustic fit cannot distinguish room variation
                            // from a new voice below VAD. Require a well-fit example.
                            && (solo[source].is_empty() || model.residual_db <= model.level_db - 10.0
                                && residual > model.residual_db + 3.0)
                        {
                            active[target][frame] = true;
                        }
                        if source_has_voice
                            && residual_is_safe
                            && pair.explained[frame] >= 0.25
                            && (!solo[source].is_empty() || difference < -8.0)
                            && difference <= model.level_db + 3.0
                            && residual <= model.residual_db + 3.0
                        {
                            copied[target][frame] = true;
                        }
                    }
                }
            }
        }
        for track in 0..tracks {
            check_cancelled(cancelled)?;
            // A single isolated copied frame does not close a microphone.
            let stable: Vec<_> = (0..frames)
                .map(|f| {
                    copied[track][f]
                        && (f > 0 && copied[track][f - 1] || f + 1 < frames && copied[track][f + 1])
                })
                .collect();
            for (f, &copy) in stable.iter().enumerate() {
                active[track][f] &= !copy;
            }
            let padding = (parameters.speech_padding_ms / 32.0).round() as usize;
            active[track] = crate::vad::preserve_nearby(&active[track], padding);
            // Apply padding after removing copied candidates. A track's own
            // nearby independent/ambiguous speech must retain its word boundaries.
        }
        let mut regions: Vec<Region> = Vec::new();
        for frame in 0..frames {
            if frame % 1024 == 0 {
                check_cancelled(cancelled)?;
            }
            let owners: Vec<_> = (0..tracks).filter(|&t| active[t][frame]).collect();
            let decision = if !owners.is_empty() {
                Decision::Owned(owners)
            } else if !speech[frame]
                && (0..tracks).all(|t| {
                    self.levels[t][frame] <= floors[t] + parameters.bleed_discrimination_db
                })
            {
                Decision::NoSpeech
            } else {
                Decision::Uncertain
            };
            let end = ((frame + 1) * FRAME).min(facts.sample_count);
            if let Some(previous) = regions.last_mut()
                && previous.decision == decision
            {
                previous.end = end;
            } else {
                regions.push(Region {
                    start: frame * FRAME,
                    end,
                    decision,
                });
            }
        }
        Ok(regions)
    }
}

fn db(power: f32) -> f32 {
    10.0 * power.max(1e-20).log10()
}

fn quantile(mut values: Vec<f32>, fraction: f32) -> f32 {
    if values.is_empty() {
        return -200.0;
    }
    let index = ((values.len() - 1) as f32 * fraction) as usize;
    *values.select_nth_unstable_by(index, f32::total_cmp).1
}

fn correlation(a: &[f32], b: &[f32], start: usize, end: usize, lag: isize, step: usize) -> f32 {
    let start = start.max((-lag).max(0) as usize);
    let end = end.min(b.len().saturating_sub(lag.max(0) as usize));
    if end.saturating_sub(start) < 32 {
        return 0.0;
    }
    let (mut xx, mut yy, mut xy) = (0.0_f32, 0.0_f32, 0.0_f32);
    for i in (start..end).step_by(step) {
        let (x, y) = (a[i], b[(i as isize + lag) as usize]);
        xx += x * x;
        yy += y * y;
        xy += x * y;
    }
    (xy * xy / (xx * yy).max(1e-30)).clamp(0.0, 1.0)
}

struct BandFilter {
    high: Biquad,
    low: Biquad,
}
impl BandFilter {
    fn new() -> Self {
        Self {
            high: Biquad::new(150.0, true),
            low: Biquad::new(3000.0, false),
        }
    }
    fn sample(&mut self, x: f32) -> f32 {
        self.low.sample(self.high.sample(x))
    }
}
struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
    state: [f32; 2],
}
impl Biquad {
    fn new(hz: f32, high: bool) -> Self {
        let w = std::f32::consts::TAU * hz / 48_000.0;
        let c = w.cos();
        let alpha = w.sin() / 2.0_f32.sqrt();
        let b = if high {
            [(1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0]
        } else {
            [(1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0]
        };
        Self {
            b: b.map(|v| v / (1.0 + alpha)),
            a: [-2.0 * c / (1.0 + alpha), (1.0 - alpha) / (1.0 + alpha)],
            state: [0.0; 2],
        }
    }
    fn sample(&mut self, x: f32) -> f32 {
        let y = self.b[0] * x + self.state[0];
        self.state[0] = self.b[1] * x - self.a[0] * y + self.state[1];
        self.state[1] = self.b[2] * x - self.a[1] * y;
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal(count: usize, phase: f32) -> Vec<f32> {
        (0..count)
            .map(|i| {
                if i < 48_000 {
                    0.0
                } else {
                    let t = i as f32;
                    0.15 * ((t * 0.071 + phase).sin()
                        + 0.4 * (t * 0.019 + (t * 0.0007).sin() + phase).sin())
                }
            })
            .collect()
    }

    fn facts(tracks: &[&[f32]]) -> DetectorFacts {
        DetectorFacts {
            sample_count: tracks[0].len(),
            probabilities: tracks
                .iter()
                .map(|_| vec![0.99; tracks[0].len().div_ceil(1536)])
                .collect(),
            energies: tracks
                .iter()
                .map(|samples| {
                    samples
                        .chunks(1536)
                        .map(|frame| {
                            10.0 * (frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32)
                                .max(1e-20)
                                .log10()
                        })
                        .collect()
                })
                .collect(),
            noise_floors: vec![-100.0; tracks.len()],
        }
    }

    #[test]
    fn delayed_inverted_speech_copy_loses_ownership() {
        let direct = signal(144_000, 0.0);
        let copy: Vec<_> = (0..direct.len())
            .map(|i| -0.25 * direct.get(i.wrapping_sub(156)).copied().unwrap_or(0.0))
            .collect();
        let tracks = [direct.as_slice(), copy.as_slice()];
        let regions = JointFacts::prepare(&tracks, None)
            .unwrap()
            .regions(
                &facts(&tracks),
                AutomixParameters::default(),
                &OwnershipSettings {
                    enabled: true,
                    ..Default::default()
                },
                None,
            )
            .unwrap();
        assert_eq!(
            regions
                .iter()
                .find(|r| r.start <= 96_000 && r.end > 96_000)
                .unwrap()
                .decision,
            Decision::Owned(vec![0])
        );
    }

    #[test]
    fn solo_example_resolves_reversed_recording_gain_and_preserves_quiet_overlap() {
        use crate::{CalibrationExample, CalibrationKind};
        let direct = signal(240_000, 0.0);
        let copy: Vec<_> = (0..direct.len())
            .map(|i| {
                1.6 * direct.get(i.wrapping_sub(114)).copied().unwrap_or(0.0)
                    + if i >= 144_000 {
                        0.06 * (i as f32 * 0.047).sin()
                    } else {
                        0.0
                    }
            })
            .collect();
        let tracks = [direct.as_slice(), copy.as_slice()];
        let joint = JointFacts::prepare(&tracks, None).unwrap();
        let settings = OwnershipSettings {
            enabled: true,
            examples: vec![CalibrationExample {
                kind: CalibrationKind::SoloSpeech,
                track: 0,
                start: 60_000,
                end: 120_000,
            }],
            ..Default::default()
        };
        let mut evidence = facts(&tracks);
        // Quiet independent speech can fall below its own microphone's VAD threshold.
        evidence.probabilities[1].fill(0.01);
        let regions = joint
            .regions(&evidence, AutomixParameters::default(), &settings, None)
            .unwrap();
        let at = |sample| {
            &regions
                .iter()
                .find(|r| r.start <= sample && r.end > sample)
                .unwrap()
                .decision
        };
        assert_eq!(at(96_000), &Decision::Owned(vec![0]));
        assert_eq!(at(192_000), &Decision::Owned(vec![0, 1]));
    }

    #[test]
    fn guided_filtered_bleed_is_reduced_while_a_quiet_third_speaker_stays_owned() {
        use crate::{CalibrationExample, CalibrationKind};
        let direct = signal(144_000, 0.0);
        let mut low = 0.0;
        let copy: Vec<_> = (0..direct.len())
            .map(|i| {
                low += 0.06 * (direct.get(i.wrapping_sub(162)).copied().unwrap_or(0.0) - low);
                -0.2 * low
            })
            .collect();
        let quiet: Vec<_> = (0..direct.len())
            .map(|i| {
                if i < 96_000 {
                    0.0
                } else {
                    0.007 * (i as f32 * 0.047).sin()
                }
            })
            .collect();
        let tracks = [direct.as_slice(), copy.as_slice(), quiet.as_slice()];
        let joint = JointFacts::prepare(&tracks, None).unwrap();
        let settings = OwnershipSettings {
            enabled: true,
            examples: vec![CalibrationExample {
                kind: CalibrationKind::SoloSpeech,
                track: 0,
                start: 60_000,
                end: 84_000,
            }],
            ..Default::default()
        };
        let regions = joint
            .regions(
                &facts(&tracks),
                AutomixParameters::default(),
                &settings,
                None,
            )
            .unwrap();
        assert_eq!(
            regions
                .iter()
                .find(|r| r.start <= 96_000 && r.end > 96_000)
                .unwrap()
                .decision,
            Decision::Owned(vec![0, 2])
        );
    }

    #[test]
    fn room_tone_teaches_a_floor_when_automatic_background_estimate_is_too_high() {
        use crate::{CalibrationExample, CalibrationKind};
        let track: Vec<_> = (0..144_000)
            .map(|i| (i as f32 * 0.047).sin() * if i < 12_000 { 0.001 } else { 0.03 })
            .collect();
        let tracks = [track.as_slice()];
        let joint = JointFacts::prepare(&tracks, None).unwrap();
        let settings = OwnershipSettings {
            enabled: true,
            examples: vec![CalibrationExample {
                kind: CalibrationKind::RoomTone,
                track: 0,
                start: 0,
                end: 12_000,
            }],
            ..Default::default()
        };
        let at = |regions: Vec<Region>| {
            regions
                .into_iter()
                .find(|r| r.start <= 96_000 && r.end > 96_000)
                .unwrap()
                .decision
        };
        assert_eq!(
            at(joint
                .regions(
                    &facts(&tracks),
                    AutomixParameters::default(),
                    &OwnershipSettings::default(),
                    None
                )
                .unwrap()),
            Decision::Uncertain
        );
        assert_eq!(
            at(joint
                .regions(
                    &facts(&tracks),
                    AutomixParameters::default(),
                    &settings,
                    None
                )
                .unwrap()),
            Decision::Owned(vec![0])
        );
    }

    #[test]
    fn cancelled_feature_work_and_derivation_do_not_return_partial_decisions() {
        let track = signal(48_000, 0.0);
        let tracks = [track.as_slice()];
        let cancelled = AtomicBool::new(true);
        assert!(JointFacts::prepare(&tracks, Some(&cancelled)).is_err());
        let joint = JointFacts::prepare(&tracks, None).unwrap();
        assert!(
            joint
                .regions(
                    &facts(&tracks),
                    AutomixParameters::default(),
                    &OwnershipSettings::default(),
                    Some(&cancelled)
                )
                .is_err()
        );
    }

    #[test]
    fn automatic_estimation_cannot_learn_persistent_independent_speech_as_bleed() {
        let direct = signal(144_000, 0.0);
        let mixed: Vec<_> = (0..direct.len())
            .map(|i| {
                0.2 * direct.get(i.wrapping_sub(120)).copied().unwrap_or(0.0)
                    + if i >= 48_000 {
                        0.015 * (i as f32 * 0.047).sin()
                    } else {
                        0.0
                    }
            })
            .collect();
        let tracks = [direct.as_slice(), mixed.as_slice()];
        let joint = JointFacts::prepare(&tracks, None).unwrap();
        let regions = joint
            .regions(
                &facts(&tracks),
                AutomixParameters::default(),
                &OwnershipSettings::default(),
                None,
            )
            .unwrap();
        assert_eq!(
            regions
                .iter()
                .find(|r| r.start <= 96_000 && r.end > 96_000)
                .unwrap()
                .decision,
            Decision::Owned(vec![0, 1])
        );
    }

    #[test]
    fn quiet_voice_shaped_overlap_survives_gain_polarity_delay_and_room_noise() {
        use crate::{CalibrationExample, CalibrationKind};
        let direct = signal(192_000, 0.0);
        for (scale, delay) in [(0.15, 48usize), (-0.4, 156), (1.6, 480)] {
            let mut seed = 17u32;
            let mixed: Vec<_> = (0..direct.len())
                .map(|i| {
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    let noise = (seed as f64 / u32::MAX as f64 - 0.5) as f32 * 0.000002;
                    let voice = if (96_000..120_000).contains(&i) {
                        let t = i as f32 / 48_000.0;
                        // Harmonics with a pulsing envelope stand in for a quiet laugh/voice.
                        0.006
                            * (1.0 + (t * 27.0).sin())
                            * ((t * 1297.0).sin() + 0.3 * (t * 2594.0).sin())
                    } else {
                        0.0
                    };
                    scale * direct.get(i.wrapping_sub(delay)).copied().unwrap_or(0.0)
                        + voice
                        + noise
                })
                .collect();
            let tracks = [direct.as_slice(), mixed.as_slice()];
            let mut evidence = facts(&tracks);
            evidence.probabilities[0][..30].fill(0.0);
            evidence.probabilities[1].fill(0.01);
            let settings = OwnershipSettings {
                enabled: true,
                examples: vec![CalibrationExample {
                    kind: CalibrationKind::SoloSpeech,
                    track: 0,
                    start: 60_000,
                    end: 84_000,
                }],
                ..Default::default()
            };
            let regions = JointFacts::prepare(&tracks, None)
                .unwrap()
                .regions(&evidence, AutomixParameters::default(), &settings, None)
                .unwrap();
            let at = |sample| {
                &regions
                    .iter()
                    .find(|r| r.start <= sample && r.end > sample)
                    .unwrap()
                    .decision
            };
            assert_eq!(
                at(9_600),
                &Decision::NoSpeech,
                "scale={scale} delay={delay}"
            );
            assert_eq!(
                at(108_000),
                &Decision::Owned(vec![0, 1]),
                "scale={scale} delay={delay}"
            );
            assert_eq!(
                at(180_000),
                &Decision::Owned(vec![0]),
                "scale={scale} delay={delay}"
            );
        }
    }

    #[test]
    fn solo_example_with_independent_sound_does_not_train_a_copy_model() {
        use crate::{CalibrationExample, CalibrationKind};
        let direct = signal(144_000, 0.0);
        let independent: Vec<_> = (0..direct.len())
            .map(|i| {
                if i >= 48_000 {
                    0.03 * (i as f32 * 0.047).sin()
                } else {
                    0.0
                }
            })
            .collect();
        let tracks = [direct.as_slice(), independent.as_slice()];
        let settings = OwnershipSettings {
            enabled: true,
            examples: vec![CalibrationExample {
                kind: CalibrationKind::SoloSpeech,
                track: 0,
                start: 60_000,
                end: 84_000,
            }],
            ..Default::default()
        };
        let result = JointFacts::prepare(&tracks, None).unwrap().regions(
            &facts(&tracks),
            AutomixParameters::default(),
            &settings,
            None,
        );
        assert!(result.unwrap_err().to_string().contains("clearer solo"));
    }

    #[test]
    fn another_microphones_speech_does_not_turn_room_noise_into_an_owner() {
        let direct = signal(192_000, 0.0);
        let noise: Vec<_> = (0..direct.len())
            .map(|i| {
                if i >= 48_000 {
                    0.001 * (i as f32 * 0.037).sin()
                } else {
                    0.0
                }
            })
            .collect();
        let tracks = [direct.as_slice(), noise.as_slice()];
        let mut evidence = facts(&tracks);
        evidence.probabilities[1].fill(0.01);
        let regions = JointFacts::prepare(&tracks, None)
            .unwrap()
            .regions(
                &evidence,
                AutomixParameters::default(),
                &OwnershipSettings::default(),
                None,
            )
            .unwrap();
        assert_eq!(
            regions
                .iter()
                .find(|r| r.start <= 144_000 && r.end > 144_000)
                .unwrap()
                .decision,
            Decision::Owned(vec![0])
        );
    }

    #[test]
    fn independent_speech_padding_preserves_short_copied_gaps_in_overlap() {
        use crate::{CalibrationExample, CalibrationKind};
        let direct = signal(192_000, 0.0);
        let mixed: Vec<_> = (0..direct.len())
            .map(|i| {
                -0.25 * direct.get(i.wrapping_sub(156)).copied().unwrap_or(0.0)
                    + if (84_000..96_000).contains(&i) || (120_000..132_000).contains(&i) {
                        0.08 * (i as f32 * 0.047).sin()
                    } else {
                        0.0
                    }
            })
            .collect();
        let tracks = [direct.as_slice(), mixed.as_slice()];
        let settings = OwnershipSettings {
            enabled: true,
            examples: vec![CalibrationExample {
                kind: CalibrationKind::SoloSpeech,
                track: 0,
                start: 48_000,
                end: 67_200,
            }],
            ..Default::default()
        };
        let regions = JointFacts::prepare(&tracks, None)
            .unwrap()
            .regions(
                &facts(&tracks),
                AutomixParameters::default(),
                &settings,
                None,
            )
            .unwrap();
        assert_eq!(
            regions
                .iter()
                .find(|r| r.start <= 108_000 && r.end > 108_000)
                .unwrap()
                .decision,
            Decision::Owned(vec![0, 1])
        );
        assert_eq!(
            regions
                .iter()
                .find(|r| r.start <= 168_000 && r.end > 168_000)
                .unwrap()
                .decision,
            Decision::Owned(vec![0])
        );
    }

    #[test]
    #[ignore = "read-only reference development diagnostics"]
    fn reference_calibration_diagnostics() {
        use crate::{CalibrationExample, CalibrationKind};
        let paths: Vec<_> = (1..=3)
            .map(|t| {
                std::path::PathBuf::from(format!("C:/Users/User/Downloads/00005603_A0{t}.wav"))
            })
            .collect();
        let (stems, _) = crate::engine::load_and_validate(&paths).unwrap();
        let tracks: Vec<_> = stems.iter().map(|s| s.samples.as_slice()).collect();
        let joint = JointFacts::prepare(&tracks, None).unwrap();
        let examples = [(0, 7.386, 9.199), (2, 4.746, 6.919), (1, 47.868, 51.810)].map(
            |(track, start, end)| CalibrationExample {
                kind: CalibrationKind::SoloSpeech,
                track,
                start: (start * 48000.0) as usize,
                end: (end * 48000.0) as usize,
            },
        );
        println!("floors {:?}", joint.floors);
        for pair in &joint.pairs {
            for (source, target) in [(pair.a, pair.b), (pair.b, pair.a)] {
                let example = examples.iter().find(|e| e.track == source).unwrap();
                let model = joint
                    .learn(
                        pair,
                        source,
                        target,
                        example.start.div_ceil(FRAME)..example.end / FRAME,
                    )
                    .unwrap();
                println!("model {source}->{target}: {model:?}");
                if target == 2 {
                    for frame in
                        (111.337 * 48000.0) as usize / FRAME..(114.757 * 48000.0) as usize / FRAME
                    {
                        let difference = joint.levels[target][frame] - joint.levels[source][frame];
                        let residual = difference + db((1.0 - pair.explained[frame]).max(1e-5));
                        if pair.explained[frame] >= 0.25
                            && difference <= model.level_db + 3.0
                            && residual <= model.residual_db + 3.0
                        {
                            println!(
                                "copy {:.3} {source}->{target} source {:.1} target {:.1} explained {:.3} difference {:.1} residual {:.1}",
                                frame as f64 * 0.032,
                                joint.levels[source][frame],
                                joint.levels[target][frame],
                                pair.explained[frame],
                                difference,
                                residual
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "bounded synthetic feature timing; run with --ignored --nocapture"]
    fn bounded_joint_feature_benchmark() {
        use std::{hint::black_box, time::Instant};
        let seed = signal(144_000, 0.0);
        let direct: Vec<_> = seed.into_iter().cycle().take(1_440_000).collect();
        let copy: Vec<_> = (0..direct.len())
            .map(|i| -0.2 * direct.get(i.wrapping_sub(120)).copied().unwrap_or(0.0))
            .collect();
        let quiet: Vec<_> = direct
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                if x == 0.0 {
                    0.0
                } else {
                    0.01 * (i as f32 * 0.047).sin()
                }
            })
            .collect();
        let tracks = [direct.as_slice(), copy.as_slice(), quiet.as_slice()];
        let started = Instant::now();
        let joint = JointFacts::prepare(&tracks, None).unwrap();
        let preparation = started.elapsed();
        let facts = facts(&tracks);
        let mut warm = Vec::new();
        for padding in 0..20 {
            let started = Instant::now();
            black_box(
                joint
                    .regions(
                        &facts,
                        AutomixParameters {
                            speech_padding_ms: padding as f32 * 32.0,
                            ..Default::default()
                        },
                        &OwnershipSettings::default(),
                        None,
                    )
                    .unwrap(),
            );
            warm.push(started.elapsed());
        }
        warm.sort();
        println!(
            "synthetic_seconds=30 tracks=3 preparation={preparation:?} feature_payload_bytes={} warm_p95={:?}",
            joint.retained_bytes(),
            warm[18]
        );
        assert!(joint.retained_bytes() < 30_000);
    }
}
