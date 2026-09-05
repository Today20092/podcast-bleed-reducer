//! Validated source facts retained without PCM; one cache belongs to one editor session.
use crate::{
    AnalysisPreview, AutomixParameters, Protection, Region, RenderBlocker,
    engine::{gain_envelope, load_and_validate},
    vad,
    waveform::SourceStamp,
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

#[derive(Debug)]
pub(crate) struct DetectorFacts {
    pub sample_count: usize,
    pub probabilities: Vec<Vec<f32>>,
    pub energies: Vec<Vec<f32>>,
    pub noise_floors: Vec<f32>,
}

#[derive(Debug)]
struct PreparedAnalysis {
    sources: Vec<SourceStamp>,
    preparation_id: &'static str,
    facts: DetectorFacts,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreviewCacheStats {
    pub preparations: usize,
    pub inference_frames: usize,
    pub ownership_derivations: usize,
}

/// Cheap to clone for a worker: detector facts and ownership are immutable and shared.
#[derive(Clone, Debug, Default)]
pub struct PreviewCache {
    prepared: Option<Arc<PreparedAnalysis>>,
    ownership: Option<(AutomixParameters, Arc<Vec<Region>>)>,
    stats: PreviewCacheStats,
}

impl PreviewCache {
    pub fn stats(&self) -> PreviewCacheStats {
        self.stats
    }

    /// Rechecks ordered source stamps on every request. No audio is retained after preparation.
    /// Cancellation is checked per detector frame and between loading/derivation/curve stages.
    pub fn analyze(
        &mut self,
        paths: &[PathBuf],
        protections: &[Protection],
        parameters: AutomixParameters,
        progress: &AtomicU8,
        cancelled: Option<&AtomicBool>,
    ) -> Result<AnalysisPreview, RenderBlocker> {
        check_cancelled(cancelled)?;
        progress.store(1, Ordering::Relaxed);
        let sources = source_stamps(paths)?;
        if self.prepared.as_ref().is_none_or(|prepared| {
            prepared.sources != sources || prepared.preparation_id != vad::PREPARATION_ID
        }) {
            self.prepared = None;
            self.ownership = None;
            progress.store(5, Ordering::Relaxed);
            let (stems, _) = load_and_validate(paths)?;
            check_cancelled(cancelled)?;
            let facts = vad::prepare(
                &stems
                    .iter()
                    .map(|stem| stem.samples.as_slice())
                    .collect::<Vec<_>>(),
                progress,
                cancelled,
            )?;
            if source_stamps(paths)? != sources {
                return Err(RenderBlocker(
                    "source changed during analysis preparation; retry".into(),
                ));
            }
            check_cancelled(cancelled)?;
            self.stats.preparations += 1;
            self.stats.inference_frames += facts.probabilities.iter().map(Vec::len).sum::<usize>();
            self.prepared = Some(Arc::new(PreparedAnalysis {
                sources,
                preparation_id: vad::PREPARATION_ID,
                facts,
            }));
        }
        let prepared = self.prepared.as_ref().unwrap();
        let sample_count = prepared.facts.sample_count;
        if protections.iter().any(|p| {
            p.start > p.end
                || (p.end != usize::MAX && p.end > sample_count)
                || p.track.is_some_and(|track| track >= paths.len())
        }) {
            return Err(RenderBlocker("protection is outside the session".into()));
        }
        check_cancelled(cancelled)?;
        if self.ownership.as_ref().is_none_or(|(previous, _)| {
            previous.speech_sensitivity != parameters.speech_sensitivity
                || previous.bleed_discrimination_db != parameters.bleed_discrimination_db
                || previous.speech_padding_ms != parameters.speech_padding_ms
        }) {
            progress.store(70, Ordering::Relaxed);
            let regions = prepared.facts.regions(
                parameters.speech_sensitivity,
                parameters.bleed_discrimination_db,
                (parameters.speech_padding_ms * 48_000.0 / 1000.0 / 1_536.0).round() as usize,
            );
            check_cancelled(cancelled)?;
            self.ownership = Some((parameters, Arc::new(regions)));
            self.stats.ownership_derivations += 1;
        }
        progress.store(80, Ordering::Relaxed);
        let regions = &self.ownership.as_ref().unwrap().1;
        let mut envelopes = Vec::with_capacity(paths.len());
        for track in 0..paths.len() {
            check_cancelled(cancelled)?;
            envelopes.push(gain_envelope(
                track,
                sample_count,
                regions,
                protections,
                10.0_f32.powf(parameters.attenuation_db / 20.0),
                (parameters.open_ms * 48_000.0 / 1000.0).max(1.0),
                (parameters.close_ms * 48_000.0 / 1000.0).max(1.0),
            ));
        }
        check_cancelled(cancelled)?;
        if source_stamps(paths)? != prepared.sources {
            self.prepared = None;
            self.ownership = None;
            return Err(RenderBlocker(
                "source changed while deriving preview; retry".into(),
            ));
        }
        progress.store(100, Ordering::Relaxed);
        Ok(AnalysisPreview {
            stems: paths.to_vec(),
            regions: regions.as_ref().clone(),
            envelopes,
            protections: protections.to_vec(),
            parameters,
        })
    }
}

fn source_stamps(paths: &[PathBuf]) -> Result<Vec<SourceStamp>, RenderBlocker> {
    paths.iter().map(|path| SourceStamp::read(path)).collect()
}

pub(crate) fn check_cancelled(cancelled: Option<&AtomicBool>) -> Result<(), RenderBlocker> {
    if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        Err(RenderBlocker("preview cancelled".into()))
    } else {
        Ok(())
    }
}

pub fn preview_stage(progress: u8) -> &'static str {
    match progress {
        0..=4 => "Checking sources",
        5..=19 => "Loading and validating audio",
        20..=69 => "Preparing speech and energy evidence",
        70..=79 => "Updating ownership",
        80..=99 => "Updating gain curves",
        _ => "Preview ready",
    }
}
