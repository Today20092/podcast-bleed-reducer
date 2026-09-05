use crate::prepared_analysis::{DetectorFacts, check_cancelled};
use crate::{Decision, Region, RenderBlocker};
use ort::{inputs, session::Session, value::Tensor};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

pub(crate) const SPEECH_PROBABILITY: f32 = 0.5;
pub(crate) const ENERGY_MARGIN_DB: f32 = 6.0;
const SOURCE_FRAME: usize = 1_536;
const MODEL_FRAME: usize = 512;
const CONTEXT: usize = 64;
#[cfg(test)]
const PRESERVATION_FRAMES: usize = 16;
const MODEL: &[u8] = include_bytes!("../assets/silero_vad.onnx");
// Bump whenever model, preprocessing, energy calculation or frame alignment changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Preprocessing {
    #[default]
    Legacy,
    AntiAlias,
}

impl Preprocessing {
    pub(crate) fn identity(self) -> &'static str {
        match self {
            Self::Legacy => "silero-v5-1A153A22/stride3-rms1536-v1",
            Self::AntiAlias => "silero-v5-1A153A22/blackman127-fc7000-centered-zero-rms1536-v1",
        }
    }
}

// 127-tap, unity-DC Blackman-windowed sinc at 7 kHz. Centering compensates
// the 63-source-sample delay; zero extension applies only at file boundaries.
// Read across 1536-sample model frames so the filter never resets between them.
struct AntiAlias {
    taps: [f32; 127],
}

impl AntiAlias {
    fn new() -> Self {
        let mut taps = std::array::from_fn(|i| {
            let x = i as f64 - 63.0;
            let cutoff = 7_000.0 / 48_000.0;
            let sinc = if x == 0.0 {
                2.0 * cutoff
            } else {
                (std::f64::consts::TAU * cutoff * x).sin() / (std::f64::consts::PI * x)
            };
            let phase = std::f64::consts::TAU * i as f64 / 126.0;
            (sinc * (0.42 - 0.5 * phase.cos() + 0.08 * (2.0 * phase).cos())) as f32
        });
        let sum: f32 = taps.iter().sum();
        for tap in &mut taps {
            *tap /= sum;
        }
        Self { taps }
    }

    fn sample(&self, samples: &[f32], center: usize) -> f32 {
        let mut sum = samples.get(center).copied().unwrap_or(0.0) * self.taps[63];
        for distance in 1..=63 {
            let left = center.checked_sub(distance).and_then(|i| samples.get(i));
            let right = samples.get(center + distance);
            sum += (left.copied().unwrap_or(0.0) + right.copied().unwrap_or(0.0))
                * self.taps[63 + distance];
        }
        sum
    }
}

pub(crate) fn analyze_with_parameters(
    tracks: &[&[f32]],
    speech_probability: f32,
    energy_margin_db: f32,
    preservation_frames: usize,
) -> Result<Vec<Region>, RenderBlocker> {
    Ok(
        prepare(tracks, &AtomicU8::new(0), None, Preprocessing::Legacy)?.regions(
            speech_probability,
            energy_margin_db,
            preservation_frames,
        ),
    )
}

pub(crate) fn prepare(
    tracks: &[&[f32]],
    progress: &AtomicU8,
    cancelled: Option<&AtomicBool>,
    preprocessing: Preprocessing,
) -> Result<DetectorFacts, RenderBlocker> {
    let frame_count = tracks[0].len().div_ceil(SOURCE_FRAME);
    let mut probabilities = Vec::with_capacity(tracks.len());
    let mut energies = Vec::with_capacity(tracks.len());
    for (track, samples) in tracks.iter().enumerate() {
        probabilities.push(silero(samples, frame_count, preprocessing, |frame| {
            check_cancelled(cancelled)?;
            progress.store(
                (20 + 50 * (track * frame_count + frame) / (tracks.len() * frame_count)) as u8,
                Ordering::Relaxed,
            );
            Ok(())
        })?);
        energies.push(
            samples
                .chunks(SOURCE_FRAME)
                .map(|frame| {
                    let rms = (frame.iter().map(|sample| sample * sample).sum::<f32>()
                        / frame.len() as f32)
                        .sqrt();
                    20.0 * rms.max(1.0e-10).log10()
                })
                .collect::<Vec<_>>(),
        );
    }
    let noise_floors = energies
        .iter()
        .map(|track| {
            let mut sorted = track.clone();
            sorted.sort_by(f32::total_cmp);
            sorted[sorted.len() / 5]
        })
        .collect::<Vec<_>>();
    Ok(DetectorFacts {
        sample_count: tracks[0].len(),
        probabilities,
        energies,
        noise_floors,
    })
}

impl DetectorFacts {
    pub(crate) fn regions(
        &self,
        speech_probability: f32,
        energy_margin_db: f32,
        preservation_frames: usize,
    ) -> Vec<Region> {
        let Self {
            sample_count,
            probabilities,
            energies,
            noise_floors,
        } = self;
        let frame_count = sample_count.div_ceil(SOURCE_FRAME);
        let active = probabilities
            .iter()
            .enumerate()
            .map(|(track, probability)| {
                preserve_nearby(
                    &(0..frame_count)
                        .map(|frame| {
                            probability[frame] >= speech_probability
                                && energies[track][frame] >= noise_floors[track] + energy_margin_db
                        })
                        .collect::<Vec<_>>(),
                    preservation_frames,
                )
            })
            .collect::<Vec<_>>();

        let mut regions: Vec<Region> = Vec::new();
        for frame in 0..frame_count {
            let speech = (0..probabilities.len())
                .filter(|&track| probabilities[track][frame] >= speech_probability)
                .collect::<Vec<_>>();
            let owners = (0..probabilities.len())
                .filter(|&track| active[track][frame])
                .collect::<Vec<_>>();
            let decision = if !owners.is_empty() {
                Decision::Owned(owners)
            } else if speech.is_empty()
                && energies
                    .iter()
                    .enumerate()
                    .all(|(track, energy)| energy[frame] < noise_floors[track] + energy_margin_db)
            {
                Decision::NoSpeech
            } else {
                Decision::Uncertain
            };
            let start = frame * SOURCE_FRAME;
            let end = ((frame + 1) * SOURCE_FRAME).min(*sample_count);
            if let Some(previous) = regions.last_mut()
                && previous.decision == decision
            {
                previous.end = end;
            } else {
                regions.push(Region {
                    start,
                    end,
                    decision,
                });
            }
        }
        regions
    }
}

pub(crate) fn preserve_nearby(detected: &[bool], preservation_frames: usize) -> Vec<bool> {
    (0..detected.len())
        .map(|frame| {
            detected[frame.saturating_sub(preservation_frames)
                ..frame
                    .saturating_add(preservation_frames)
                    .saturating_add(1)
                    .min(detected.len())]
                .iter()
                .any(|active| *active)
        })
        .collect()
}

fn silero(
    samples: &[f32],
    frame_count: usize,
    preprocessing: Preprocessing,
    mut checkpoint: impl FnMut(usize) -> Result<(), RenderBlocker>,
) -> Result<Vec<f32>, RenderBlocker> {
    checkpoint(0)?;
    let mut session = Session::builder()
        .and_then(|builder| builder.commit_from_memory(MODEL))
        .map_err(|error| RenderBlocker(format!("Silero detector could not start: {error}")))?;
    let mut state = vec![0.0_f32; 256];
    let mut context = vec![0.0_f32; CONTEXT];
    let mut probabilities = Vec::with_capacity(frame_count);
    let filter = (preprocessing == Preprocessing::AntiAlias).then(AntiAlias::new);
    for (frame, source) in samples.chunks(SOURCE_FRAME).enumerate() {
        checkpoint(frame)?;
        let mut audio = context.clone();
        audio.extend((0..MODEL_FRAME).map(|index| {
            if index * 3 >= source.len() {
                0.0
            } else if let Some(filter) = &filter {
                filter.sample(samples, frame * SOURCE_FRAME + index * 3)
            } else {
                source[index * 3]
            }
        }));
        context.copy_from_slice(&audio[MODEL_FRAME..]);
        let outputs = session
            .run(inputs![
                "input" => Tensor::from_array((vec![1_usize, MODEL_FRAME + CONTEXT], audio.into_boxed_slice())).map_err(detector_error)?,
                "state" => Tensor::from_array((vec![2_usize, 1, 128], state.into_boxed_slice())).map_err(detector_error)?,
                "sr" => Tensor::from_array((Vec::<usize>::new(), vec![16_000_i64].into_boxed_slice())).map_err(detector_error)?
            ])
            .map_err(detector_error)?;
        let (_, probability) = outputs["output"]
            .try_extract_tensor::<f32>()
            .map_err(detector_error)?;
        let (_, next_state) = outputs["stateN"]
            .try_extract_tensor::<f32>()
            .map_err(detector_error)?;
        probabilities.push(probability[0]);
        state = next_state.to_vec();
    }
    Ok(probabilities)
}

fn detector_error(error: ort::Error) -> RenderBlocker {
    RenderBlocker(format!("Silero detector failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{PRESERVATION_FRAMES, preserve_nearby};

    #[test]
    fn filtered_decimation_preserves_speech_band_and_rejects_aliases_without_time_shift() {
        let filter = super::AntiAlias::new();
        let rms = |frequency: f32| {
            let samples: Vec<_> = (0..48_001)
                .map(|i| {
                    (std::f64::consts::TAU * f64::from(frequency) * i as f64 / 48_000.0 + 0.37)
                        .sin() as f32
                })
                .collect();
            let power: f64 = (100..15_900)
                .map(|i| f64::from(filter.sample(&samples, i * 3)).powi(2))
                .sum();
            (power / 15_800.0).sqrt()
        };
        for frequency in [100.0, 1_000.0, 6_000.0] {
            assert!((rms(frequency) * 2.0_f64.sqrt() - 1.0).abs() < 0.01);
        }
        for frequency in [8_000.0, 9_000.0, 12_000.0, 20_000.0] {
            assert!(rms(frequency) < 0.001, "alias at {frequency} Hz");
        }
        let mut impulse = vec![0.0; 3_073];
        impulse[1_536] = 1.0;
        let output: Vec<_> = (0..impulse.len().div_ceil(3))
            .map(|i| filter.sample(&impulse, i * 3))
            .collect();
        assert_eq!(output.len(), 1_025);
        assert_eq!(
            output
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .unwrap()
                .0,
            512
        );
        assert!((filter.sample(&vec![1.0; 200], 99) - 1.0).abs() < 1.0e-6);
        assert_eq!(filter.sample(&[], 0), 0.0);
        assert!(filter.sample(&[1.0], 0).is_finite());
    }

    #[test]
    fn preserves_short_gaps_and_boundaries_around_detected_speech() {
        let mut detected = vec![false; 40];
        detected[20] = true;
        let preserved = preserve_nearby(&detected, PRESERVATION_FRAMES);
        assert!(preserved[4]);
        assert!(preserved[36]);
        assert!(!preserved[3]);
        assert!(!preserved[37]);
    }
}
