//! Validated source facts retained without PCM; one cache belongs to one editor session.
use crate::{
    AnalysisPreview, AutomixParameters, OwnershipSettings, Protection, Region, RenderBlocker,
    engine::{gain_envelope, load_and_validate},
    joint_detector::JointFacts,
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
    joint: JointFacts,
    feature_millis: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JointDiagnostics {
    pub feature_preparation_ms: u128,
    /// Numeric feature payload, excluding Vec/allocation headers and legacy VAD facts.
    pub retained_feature_bytes: usize,
    pub pair_count: usize,
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
    preprocessing: vad::Preprocessing,
    prepared: Option<Arc<PreparedAnalysis>>,
    ownership: Option<(AutomixParameters, OwnershipSettings, Arc<Vec<Region>>)>,
    stats: PreviewCacheStats,
}

impl PreviewCache {
    pub(crate) fn requires_source_validation(
        &self,
        paths: &[PathBuf],
        settings: &OwnershipSettings,
    ) -> Result<bool, RenderBlocker> {
        Ok(self
            .prepared
            .as_ref()
            .is_none_or(|p| source_stamps(paths).as_ref().ok() != Some(&p.sources))
            || self
                .ownership
                .as_ref()
                .is_none_or(|(_, previous, _)| previous != settings))
    }
    pub fn stats(&self) -> PreviewCacheStats {
        self.stats
    }

    pub fn joint_diagnostics(&self) -> Option<JointDiagnostics> {
        self.prepared.as_ref().map(|p| JointDiagnostics {
            feature_preparation_ms: p.feature_millis,
            retained_feature_bytes: p.joint.retained_bytes(),
            pair_count: p.facts.probabilities.len() * p.facts.probabilities.len().saturating_sub(1)
                / 2,
        })
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
        self.analyze_with_ownership(
            paths,
            protections,
            parameters,
            &OwnershipSettings::default(),
            progress,
            cancelled,
        )
    }

    #[allow(clippy::too_many_arguments)] // Shared editor/render contract.
    pub fn analyze_with_ownership(
        &mut self,
        paths: &[PathBuf],
        protections: &[Protection],
        parameters: AutomixParameters,
        ownership: &OwnershipSettings,
        progress: &AtomicU8,
        cancelled: Option<&AtomicBool>,
    ) -> Result<AnalysisPreview, RenderBlocker> {
        check_cancelled(cancelled)?;
        validate_parameters(parameters)?;
        progress.store(1, Ordering::Relaxed);
        let sources = source_stamps(paths)?;
        validate_calibration(ownership, &sources, None)?;
        if self.prepared.as_ref().is_none_or(|prepared| {
            prepared.sources != sources || prepared.preparation_id != self.preprocessing.identity()
        }) {
            self.prepared = None;
            self.ownership = None;
            progress.store(5, Ordering::Relaxed);
            let (stems, _) = load_and_validate(paths)?;
            validate_calibration(ownership, &sources, Some(stems[0].samples.len()))?;
            check_cancelled(cancelled)?;
            let facts = vad::prepare(
                &stems
                    .iter()
                    .map(|stem| stem.samples.as_slice())
                    .collect::<Vec<_>>(),
                progress,
                cancelled,
                self.preprocessing,
            )?;
            let started = std::time::Instant::now();
            let joint = JointFacts::prepare(
                &stems
                    .iter()
                    .map(|s| s.samples.as_slice())
                    .collect::<Vec<_>>(),
                cancelled,
            )?;
            let feature_millis = started.elapsed().as_millis();
            if source_stamps(paths)? != sources {
                return Err(RenderBlocker(
                    "source changed during analysis preparation; retry".into(),
                ));
            }
            check_cancelled(cancelled)?;
            self.stats.preparations += 1;
            self.stats.inference_frames += facts.probabilities.iter().map(Vec::len).sum::<usize>();
            self.prepared = Some(Arc::new(PreparedAnalysis {
                sources: sources.clone(),
                preparation_id: self.preprocessing.identity(),
                facts,
                joint,
                feature_millis,
            }));
        }
        let prepared = self.prepared.as_ref().unwrap();
        let sample_count = prepared.facts.sample_count;
        validate_calibration(ownership, &sources, Some(sample_count))?;
        if protections.iter().any(|p| {
            p.start >= p.end
                || (p.end != u64::MAX && p.end > crate::timebase::detector_time(sample_count))
                || p.track.is_some_and(|track| track >= paths.len())
        }) {
            return Err(RenderBlocker("protection is outside the session".into()));
        }
        check_cancelled(cancelled)?;
        if self
            .ownership
            .as_ref()
            .is_none_or(|(previous, previous_settings, _)| {
                previous_settings != ownership
                    || previous.speech_sensitivity != parameters.speech_sensitivity
                    || previous.bleed_discrimination_db != parameters.bleed_discrimination_db
                    || previous.speech_padding_ms != parameters.speech_padding_ms
            })
        {
            progress.store(70, Ordering::Relaxed);
            let regions = if ownership.enabled {
                prepared
                    .joint
                    .regions(&prepared.facts, parameters, ownership, cancelled)?
            } else {
                prepared.facts.regions(
                    parameters.speech_sensitivity,
                    parameters.bleed_discrimination_db,
                    (parameters.speech_padding_ms * crate::timebase::DETECTOR_SAMPLE_RATE as f32
                        / 1000.0
                        / 1_536.0)
                        .round() as usize,
                )
            };
            check_cancelled(cancelled)?;
            self.ownership = Some((parameters, ownership.clone(), Arc::new(regions)));
            self.stats.ownership_derivations += 1;
        }
        progress.store(80, Ordering::Relaxed);
        let regions = &self.ownership.as_ref().unwrap().2;
        let mut envelopes = Vec::with_capacity(paths.len());
        for track in 0..paths.len() {
            check_cancelled(cancelled)?;
            envelopes.push(gain_envelope(
                track,
                sample_count,
                regions,
                protections,
                10.0_f32.powf(parameters.attenuation_db / 20.0),
                (parameters.open_ms * crate::timebase::DETECTOR_SAMPLE_RATE as f32 / 1000.0)
                    .max(1.0),
                (parameters.close_ms * crate::timebase::DETECTOR_SAMPLE_RATE as f32 / 1000.0)
                    .max(1.0),
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

pub(crate) fn source_stamps(paths: &[PathBuf]) -> Result<Vec<SourceStamp>, RenderBlocker> {
    paths.iter().map(|path| SourceStamp::read(path)).collect()
}

// Source lengths are equal after engine validation; before loading, validate all
// constraints that do not require the audio length using the shared boundary.
pub(crate) fn validate_calibration(
    settings: &OwnershipSettings,
    sources: &[SourceStamp],
    sample_count: Option<usize>,
) -> Result<(), RenderBlocker> {
    settings.validate_durations(
        sources,
        &vec![sample_count.map_or(u64::MAX, crate::timebase::detector_time); sources.len()],
    )
}

pub(crate) fn validate_parameters(p: AutomixParameters) -> Result<(), RenderBlocker> {
    if ![
        p.speech_sensitivity,
        p.bleed_discrimination_db,
        p.attenuation_db,
        p.speech_padding_ms * crate::timebase::DETECTOR_SAMPLE_RATE as f32 / 1000.0,
        p.open_ms * crate::timebase::DETECTOR_SAMPLE_RATE as f32 / 1000.0,
        p.close_ms * crate::timebase::DETECTOR_SAMPLE_RATE as f32 / 1000.0,
    ]
    .iter()
    .all(|v| v.is_finite())
        || !(0.0..=1.0).contains(&p.speech_sensitivity)
        || p.bleed_discrimination_db < 0.0
        || p.attenuation_db > 0.0
        || p.speech_padding_ms < 0.0
        || p.open_ms < 0.0
        || p.close_ms < 0.0
    {
        return Err(RenderBlocker(
            "automix parameters must be finite, attenuation-only, and within valid ranges".into(),
        ));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocessing_change_invalidates_detector_and_ownership_then_restores_legacy() {
        use bwavfile::{WaveFmt, WaveWriter};
        let path =
            std::env::temp_dir().join(format!("detector-identity-{}.wav", std::process::id()));
        let writer = WaveWriter::create(&path, WaveFmt::new_pcm_mono(48_000, 16)).unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(&vec![0.0_f32; 3_073]).unwrap();
        frames.end().unwrap();
        let mut cache = PreviewCache::default();
        let progress = AtomicU8::new(0);
        let paths = std::slice::from_ref(&path);
        let parameters = AutomixParameters::default();
        let legacy = cache
            .analyze(paths, &[], parameters, &progress, None)
            .unwrap();
        cache.preprocessing = vad::Preprocessing::AntiAlias;
        cache
            .analyze(paths, &[], parameters, &progress, None)
            .unwrap();
        cache
            .analyze(paths, &[], parameters, &progress, None)
            .unwrap();
        assert_eq!(cache.stats().preparations, 2);
        assert_eq!(cache.stats().ownership_derivations, 2);
        cache.preprocessing = vad::Preprocessing::Legacy;
        assert_eq!(
            legacy,
            cache
                .analyze(paths, &[], parameters, &progress, None)
                .unwrap()
        );
        assert_eq!(cache.stats().preparations, 3);
        std::fs::remove_file(path).unwrap();
    }
}
