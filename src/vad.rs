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
pub(crate) const PREPARATION_ID: &str = "silero-v5-1A153A22/stride3-rms1536-v1";

pub(crate) fn analyze_with_parameters(
    tracks: &[&[f32]],
    speech_probability: f32,
    energy_margin_db: f32,
    preservation_frames: usize,
) -> Result<Vec<Region>, RenderBlocker> {
    Ok(prepare(tracks, &AtomicU8::new(0), None)?.regions(
        speech_probability,
        energy_margin_db,
        preservation_frames,
    ))
}

pub(crate) fn prepare(
    tracks: &[&[f32]],
    progress: &AtomicU8,
    cancelled: Option<&AtomicBool>,
) -> Result<DetectorFacts, RenderBlocker> {
    let frame_count = tracks[0].len().div_ceil(SOURCE_FRAME);
    let mut probabilities = Vec::with_capacity(tracks.len());
    let mut energies = Vec::with_capacity(tracks.len());
    for (track, samples) in tracks.iter().enumerate() {
        probabilities.push(silero(samples, frame_count, |frame| {
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

fn preserve_nearby(detected: &[bool], preservation_frames: usize) -> Vec<bool> {
    (0..detected.len())
        .map(|frame| {
            detected[frame.saturating_sub(preservation_frames)
                ..(frame + preservation_frames + 1).min(detected.len())]
                .iter()
                .any(|active| *active)
        })
        .collect()
}

fn silero(
    samples: &[f32],
    frame_count: usize,
    mut checkpoint: impl FnMut(usize) -> Result<(), RenderBlocker>,
) -> Result<Vec<f32>, RenderBlocker> {
    checkpoint(0)?;
    let mut session = Session::builder()
        .and_then(|builder| builder.commit_from_memory(MODEL))
        .map_err(|error| RenderBlocker(format!("Silero detector could not start: {error}")))?;
    let mut state = vec![0.0_f32; 256];
    let mut context = vec![0.0_f32; CONTEXT];
    let mut probabilities = Vec::with_capacity(frame_count);
    for (frame, source) in samples.chunks(SOURCE_FRAME).enumerate() {
        checkpoint(frame)?;
        let mut audio = context.clone();
        audio.extend((0..MODEL_FRAME).map(|index| source.get(index * 3).copied().unwrap_or(0.0)));
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
