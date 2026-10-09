//! Trusted-reference ownership models, separate from fixed extractors and gain edits.
use crate::model::{DiarizationEvidence, ProbabilityHistory};
use crate::{
    AcousticEvidenceCache, AnalysisAudioIdentity, OwnershipModel, OwnershipTrainingExample,
    OwnershipTrainingSettings, PreparedAnalysisAudio, Project, RenderBlocker, SpeakerId,
    SpeakerMapping, TICKS_PER_SECOND, TrackId,
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8},
    },
};
pub const OWNERSHIP_FEATURE_RECIPE: &str =
    "mapped-own-neural-mean32ms-raw-silero-acoustic08-uniontruth-centers-fulltime-classcap2048-v1";
pub const OWNERSHIP_FRAME_TICKS: u64 = TICKS_PER_SECOND * 32 / 1000;
const NEURAL_STEP: u64 = TICKS_PER_SECOND / 100;
const BLOCK: usize = 1024;
#[derive(Clone, Copy, Debug, Default)]
pub struct OwnershipCacheStats {
    pub fixed_vad_preparations: u64,
    pub model_fits: u64,
    pub probability_passes: u64,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TrackOwnershipModel {
    pub track: TrackId,
    pub speaker: SpeakerId,
    pub model: OwnershipModel,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionOwnershipModels {
    pub recipe: String,
    pub cache_key: String,
    pub mapping_key: String,
    pub acoustic_key: String,
    pub fixed_vad_key: String,
    pub sources: Vec<AnalysisAudioIdentity>,
    pub models: Vec<TrackOwnershipModel>,
}
impl SessionOwnershipModels {
    pub fn to_json(&self) -> Result<String, RenderBlocker> {
        self.validate()?;
        serde_json::to_string(self).map_err(|e| RenderBlocker(e.to_string()))
    }
    pub fn from_json(json: &str) -> Result<Self, RenderBlocker> {
        let result: Self = serde_json::from_str(json).map_err(|e| RenderBlocker(e.to_string()))?;
        result.validate()?;
        Ok(result)
    }
    fn validate(&self) -> Result<(), RenderBlocker> {
        if self.recipe != OWNERSHIP_FEATURE_RECIPE
            || self.models.is_empty()
            || self.models.len() > 8
            || self.sources.len() != self.models.len()
            || [
                &self.cache_key,
                &self.mapping_key,
                &self.acoustic_key,
                &self.fixed_vad_key,
            ]
            .iter()
            .any(|key| key.len() != 64 || !key.bytes().all(|b| b.is_ascii_hexdigit()))
            || self.models.iter().enumerate().any(|(i, m)| {
                self.models[..i]
                    .iter()
                    .any(|other| other.track == m.track || other.speaker == m.speaker)
            })
        {
            return Err(RenderBlocker(
                "Invalid source-bound ownership model metadata.".into(),
            ));
        }
        for model in &self.models {
            model.model.to_json()?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct OwnershipEstimate {
    pub track: TrackId,
    pub probability: Option<f32>,
    pub preservation_reason: Option<String>,
}
#[derive(Clone, Debug)]
pub struct SessionOwnership {
    pub cache_key: String,
    pub duration: u64,
    pub models: Option<SessionOwnershipModels>,
    tracks: Vec<TrackId>,
    probabilities: Vec<Vec<f32>>,
    warnings: Vec<Vec<u8>>,
    unresolved: Option<String>,
    reference_identity: String,
    raw_speech: Vec<Vec<f32>>,
    assigned_activity: Vec<Vec<f32>>,
    global_activity: Vec<f32>,
}
impl SessionOwnership {
    pub(crate) fn source_identity(project: &Project) -> String {
        hash(&serde_json::json!(
            project
                .sources
                .iter()
                .map(|s| (s.track_id, &s.fingerprint, s.metadata))
                .collect::<Vec<_>>()
        ))
    }
    /// Current trusted truth and source bindings, excluding gain or manual edits.
    pub(crate) fn reference_identity(project: &Project) -> String {
        hash(
            &serde_json::json!({"sources":project.sources.iter().map(|s|(s.track_id,s.speaker_id,&s.fingerprint,s.metadata.grid)).collect::<Vec<_>>(),"references":project.reference_labels()}),
        )
    }
    pub fn temporal_evidence(
        &self,
        project: &Project,
    ) -> Result<crate::TemporalEvidenceGrid, RenderBlocker> {
        self.temporal_evidence_cancellable(project, None)
    }
    pub fn temporal_evidence_cancellable(
        &self,
        project: &Project,
        cancelled: Option<&AtomicBool>,
    ) -> Result<crate::TemporalEvidenceGrid, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        if self.reference_identity != Self::reference_identity(project) {
            return Err(RenderBlocker(
                "Ownership analysis does not match current sources and references.".into(),
            ));
        }
        let mut rows = vec![Vec::with_capacity(self.frame_count()); self.tracks.len()];
        for frame in 0..self.frame_count() {
            if frame % 1024 == 0 {
                crate::prepared_analysis::check_cancelled(cancelled)?;
            }
            for (track, estimate) in self
                .estimates_at(frame as u64 * OWNERSHIP_FRAME_TICKS)?
                .into_iter()
                .enumerate()
            {
                rows[track].push(crate::TemporalEvidence {
                    probability: estimate.probability,
                    raw_speech: self
                        .raw_speech
                        .get(track)
                        .and_then(|r| r.get(frame))
                        .copied(),
                    assigned_activity: self
                        .assigned_activity
                        .get(track)
                        .and_then(|r| r.get(frame))
                        .copied(),
                    preservation_reason: estimate.preservation_reason,
                });
            }
        }
        let mut result = crate::TemporalEvidenceGrid::new(
            self.tracks.clone(),
            self.duration,
            rows,
            if self.global_activity.is_empty() {
                vec![0.0; self.frame_count()]
            } else {
                self.global_activity.clone()
            },
        )?;
        result.bind_upstream_identity(&self.cache_key);
        crate::prepared_analysis::check_cancelled(cancelled)?;
        Ok(result)
    }
    pub fn frame_count(&self) -> usize {
        self.duration.div_ceil(OWNERSHIP_FRAME_TICKS) as usize
    }
    pub fn estimates_at(&self, time: u64) -> Result<Vec<OwnershipEstimate>, RenderBlocker> {
        if time >= self.duration {
            return Err(RenderBlocker(
                "Ownership estimate is outside the native recording.".into(),
            ));
        }
        let frame = (time / OWNERSHIP_FRAME_TICKS) as usize;
        Ok(self
            .tracks
            .iter()
            .enumerate()
            .map(|(track, &id)| {
                let warning = self
                    .warnings
                    .get(track)
                    .and_then(|rows| rows.get(frame))
                    .copied()
                    .unwrap_or(1);
                let reason = self.unresolved.clone().or_else(|| match warning {
                    1 => Some("Incomplete source-time evidence; preserve this region.".into()),
                    2 => Some("Poor or changing bleed evidence; preserve for review.".into()),
                    _ => None,
                });
                OwnershipEstimate {
                    track: id,
                    probability: if self.unresolved.is_some() || warning == 1 {
                        None
                    } else {
                        self.probabilities
                            .get(track)
                            .and_then(|rows| rows.get(frame))
                            .copied()
                    },
                    preservation_reason: reason,
                }
            })
            .collect())
    }
}
struct FixedVad {
    key: String,
    probabilities: Vec<Vec<f32>>,
}
#[derive(Default)]
pub struct SessionOwnershipCache {
    acoustics: AcousticEvidenceCache,
    vad: Option<Arc<FixedVad>>,
    models: Option<Arc<SessionOwnershipModels>>,
    accepted: Option<Arc<SessionOwnership>>,
    stats: OwnershipCacheStats,
}
impl SessionOwnershipCache {
    pub fn stats(&self) -> OwnershipCacheStats {
        self.stats
    }
    pub fn last_completed(&self) -> Option<&Arc<SessionOwnership>> {
        self.accepted.as_ref()
    }
    /// Restores a structurally validated cache; Analyze still checks every current identity.
    pub fn restore_models(&mut self, models: SessionOwnershipModels) -> Result<(), RenderBlocker> {
        models.validate()?;
        self.models = Some(Arc::new(models));
        Ok(())
    }
    pub fn analyze(
        &mut self,
        project: &Project,
        audio: &[PreparedAnalysisAudio],
        evidence: &DiarizationEvidence,
        settings: OwnershipTrainingSettings,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Arc<SessionOwnership>, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let acoustics = self.acoustics.analyze(project, audio, cancelled)?;
        let mapping = crate::resolve_speaker_mapping(project, audio, evidence, cancelled)?;
        if !mapping.readiness.ready() {
            return Err(RenderBlocker(format!(
                "Ownership references need correction: {:?}",
                mapping.readiness.issues
            )));
        }
        let duration = project
            .sources
            .iter()
            .map(|source| source.metadata.grid.duration())
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .unwrap_or(0);
        let key = hash(
            &serde_json::json!({"recipe":OWNERSHIP_FEATURE_RECIPE,"model":crate::OWNERSHIP_MODEL_RECIPE,"mapping":mapping.cache_key,"acoustics":acoustics.cache_key,"settings":settings}),
        );
        if mapping.channels.values().any(Option::is_none) {
            return Ok(Arc::new(SessionOwnership {
                cache_key: key,
                duration,
                models: None,
                tracks: project.sources.iter().map(|s| s.track_id).collect(),
                probabilities: vec![],
                warnings: vec![],
                unresolved: Some(
                    "Assigned speaker identity is missing or ambiguous; preserve for review."
                        .into(),
                ),
                reference_identity: SessionOwnership::reference_identity(project),
                raw_speech: vec![],
                assigned_activity: vec![],
                global_activity: vec![],
            }));
        }
        if let Some(accepted) = &self.accepted
            && accepted.cache_key == key
        {
            return Ok(accepted.clone());
        }
        let vad_key = hash(
            &serde_json::json!({"recipe":"silero-direct16k512-v1","model_sha256":format!("{:x}",Sha256::digest(include_bytes!("../assets/silero_vad.onnx"))),"runtime_build":"ort2.0.0-rc.10","sources":audio.iter().map(|a|(&a.identity,a.audio_sha256())).collect::<Vec<_>>()}),
        );
        let vad = if let Some(vad) = &self.vad
            && vad.key == vad_key
        {
            vad.clone()
        } else {
            let count = audio
                .iter()
                .map(|a| a.identity.frame_count)
                .max()
                .unwrap_or(0)
                .checked_mul(3)
                .and_then(|count| usize::try_from(count).ok())
                .ok_or_else(|| RenderBlocker("VAD time coordinate overflow.".into()))?;
            let facts =
                crate::vad::prepare_analysis_tracks(audio, count, &AtomicU8::new(0), cancelled)?;
            if facts
                .probabilities
                .iter()
                .flatten()
                .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
            {
                return Err(RenderBlocker(
                    "Fixed VAD contains invalid probability evidence.".into(),
                ));
            }
            let vad = Arc::new(FixedVad {
                key: vad_key.clone(),
                probabilities: facts.probabilities,
            });
            self.vad = Some(vad.clone());
            self.stats.fixed_vad_preparations += 1;
            vad
        };
        let (truth, anchors) = truth_frames(project)?;
        let selected: Vec<_> = project
            .sources
            .iter()
            .map(|source| selected_training_frames(&truth, &anchors, source.speaker_id))
            .collect();
        let frames = duration.div_ceil(OWNERSHIP_FRAME_TICKS) as usize;
        let mut examples: Vec<Vec<OwnershipTrainingExample>> = vec![vec![]; project.sources.len()];
        for start in (0..frames).step_by(BLOCK) {
            crate::prepared_analysis::check_cancelled(cancelled)?;
            let end = (start + BLOCK).min(frames);
            if !truth.range(start..end).next().is_some() {
                continue;
            }
            let neural = neural_rows(&evidence.history, start, end)?;
            for (frame, active) in truth.range(start..end) {
                let time = *frame as u64 * OWNERSHIP_FRAME_TICKS;
                let acoustic = acoustics.features_at(time)?;
                for (track, source) in project.sources.iter().enumerate() {
                    if !selected[track].contains(frame) {
                        continue;
                    }
                    examples[track].push(OwnershipTrainingExample {
                        features: features(
                            track,
                            &mapping,
                            &neural[*frame - start],
                            &vad,
                            &acoustic[track],
                            *frame,
                            source.speaker_id,
                        )?,
                        owns_speech: active.contains(&source.speaker_id),
                    });
                }
            }
        }
        let names = feature_names(project);
        let cached = self.models.as_ref().filter(|models| {
            models.cache_key == key
                && models.mapping_key == mapping.cache_key
                && models.acoustic_key == acoustics.cache_key
                && models.fixed_vad_key == vad_key
                && models.sources == audio.iter().map(|a| a.identity.clone()).collect::<Vec<_>>()
                && models.models.iter().zip(&project.sources).enumerate().all(
                    |(i, (model, source))| {
                        model.track == source.track_id
                            && model.speaker == source.speaker_id
                            && model.model.matches_recipe(&names[i], settings)
                    },
                )
        });
        let mut new_fits = 0;
        let model_set = if let Some(models) = cached {
            for (track, model) in models.models.iter().enumerate() {
                model
                    .model
                    .validate_truth_rows(&capped_rows(&examples[track]))?;
            }
            models.as_ref().clone()
        } else {
            let mut models = vec![];
            for (track, source) in project.sources.iter().enumerate() {
                crate::prepared_analysis::check_cancelled(cancelled)?;
                let rows = capped_rows(&examples[track]);
                let model =
                    OwnershipModel::train(names[track].clone(), &rows, settings, cancelled)?;
                models.push(TrackOwnershipModel {
                    track: source.track_id,
                    speaker: source.speaker_id,
                    model,
                });
                new_fits += 1;
            }
            SessionOwnershipModels {
                recipe: OWNERSHIP_FEATURE_RECIPE.into(),
                cache_key: key.clone(),
                mapping_key: mapping.cache_key.clone(),
                acoustic_key: acoustics.cache_key.clone(),
                fixed_vad_key: vad_key,
                sources: audio.iter().map(|a| a.identity.clone()).collect(),
                models,
            }
        };
        let mut result = SessionOwnership {
            cache_key: key,
            duration,
            models: Some(model_set),
            tracks: project.sources.iter().map(|s| s.track_id).collect(),
            probabilities: vec![Vec::with_capacity(frames); audio.len()],
            warnings: vec![Vec::with_capacity(frames); audio.len()],
            unresolved: None,
            reference_identity: SessionOwnership::reference_identity(project),
            raw_speech: vad.probabilities.clone(),
            assigned_activity: vec![Vec::with_capacity(frames); audio.len()],
            global_activity: Vec::with_capacity(frames),
        };
        for start in (0..frames).step_by(BLOCK) {
            crate::prepared_analysis::check_cancelled(cancelled)?;
            let end = (start + BLOCK).min(frames);
            let neural = neural_rows(&evidence.history, start, end)?;
            for frame in start..end {
                result
                    .global_activity
                    .push(neural[frame - start].iter().copied().fold(0.0, f32::max));
                let acoustic = acoustics.features_at(frame as u64 * OWNERSHIP_FRAME_TICKS);
                for (track, source) in project.sources.iter().enumerate() {
                    result.assigned_activity[track]
                        .push(neural[frame - start][mapping.channels[&source.speaker_id].unwrap()]);
                    if let Ok(acoustic) = &acoustic {
                        let x = features(
                            track,
                            &mapping,
                            &neural[frame - start],
                            &vad,
                            &acoustic[track],
                            frame,
                            source.speaker_id,
                        )?;
                        let probability = result.models.as_ref().unwrap().models[track]
                            .model
                            .estimate(&x)? as f32;
                        let poorly_explained = acoustic[track].bleed.iter().any(|bleed| {
                            let speaker = project
                                .sources
                                .iter()
                                .find(|source| source.track_id == bleed.source)
                                .unwrap()
                                .speaker_id;
                            let activity =
                                neural[frame - start][mapping.channels[&speaker].unwrap()];
                            activity >= 0.5
                                && (bleed.reliability < 0.7
                                    || bleed.excess_residual_db.is_some_and(|excess| excess > 6.0))
                        });
                        result.probabilities[track].push(probability);
                        result.warnings[track].push(if poorly_explained { 2 } else { 0 });
                    } else {
                        result.probabilities[track].push(0.0);
                        result.warnings[track].push(1);
                    }
                }
            }
        }
        // Validate mutable disk boundaries once more before installing a completed derivation.
        self.acoustics.analyze(project, audio, cancelled)?;
        crate::resolve_speaker_mapping(project, audio, evidence, cancelled)?;
        crate::prepared_analysis::check_cancelled(cancelled)?;
        self.stats.model_fits += new_fits;
        self.stats.probability_passes += 1;
        self.models = Some(Arc::new(result.models.as_ref().unwrap().clone()));
        let result = Arc::new(result);
        self.accepted = Some(result.clone());
        Ok(result)
    }
}
fn hash(value: &serde_json::Value) -> String {
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}
type TruthFrames = BTreeMap<usize, Vec<SpeakerId>>;
fn truth_frames(project: &Project) -> Result<(TruthFrames, BTreeSet<usize>), RenderBlocker> {
    let mut intervals: Vec<_> = project
        .reference_labels()
        .iter()
        .map(|label| {
            (
                (label.range.start.div_ceil(OWNERSHIP_FRAME_TICKS)) as usize,
                (label.range.end / OWNERSHIP_FRAME_TICKS) as usize,
                label.active_speakers.clone(),
            )
        })
        .collect();
    intervals.sort_by_key(|row| row.0);
    let mut unions: Vec<(usize, usize, Vec<SpeakerId>)> = vec![];
    for (start, end, active) in intervals {
        if let Some(previous) = unions
            .last_mut()
            .filter(|previous| start <= previous.1 && active == previous.2)
        {
            previous.1 = previous.1.max(end);
        } else {
            unions.push((start, end, active));
        }
    }
    let mut truth = BTreeMap::new();
    let mut anchors = BTreeSet::new();
    for (start, end, active) in unions {
        let count = end.saturating_sub(start).min(2048);
        if count > 0 {
            anchors.insert(start + (end - start) / 2);
        }
        for i in 0..count {
            let frame = start + i * (end - start) / count;
            if truth
                .insert(frame, active.clone())
                .is_some_and(|old| old != active)
            {
                return Err(RenderBlocker("Contradictory human ownership truth; replace the overlapping reference explicitly.".into()));
            }
        }
    }
    Ok((truth, anchors))
}
fn selected_training_frames(
    truth: &TruthFrames,
    anchors: &BTreeSet<usize>,
    speaker: SpeakerId,
) -> BTreeSet<usize> {
    let mut selected = BTreeSet::new();
    for owns in [false, true] {
        let class: Vec<_> = truth
            .iter()
            .filter(|(_, active)| active.contains(&speaker) == owns)
            .map(|(&frame, _)| frame)
            .collect();
        let count = class.len().min(2048);
        let centers: Vec<_> = class
            .iter()
            .copied()
            .filter(|frame| anchors.contains(frame))
            .collect();
        let center_count = centers.len().min(count);
        let mut chosen = BTreeSet::new();
        for i in 0..center_count {
            chosen.insert(centers[i * centers.len() / center_count]);
        }
        let remaining = count - chosen.len();
        let others: Vec<_> = class
            .iter()
            .copied()
            .filter(|frame| !chosen.contains(frame))
            .collect();
        for i in 0..remaining {
            chosen.insert(others[i * others.len() / remaining]);
        }
        selected.extend(chosen);
    }
    selected
}
fn capped_rows(rows: &[OwnershipTrainingExample]) -> Vec<OwnershipTrainingExample> {
    let mut result = vec![];
    for owns in [false, true] {
        let class: Vec<_> = rows.iter().filter(|row| row.owns_speech == owns).collect();
        let count = class.len().min(2048);
        for i in 0..count {
            result.push(class[i * class.len() / count].clone());
        }
    }
    result
}
fn neural_rows(
    history: &ProbabilityHistory,
    start: usize,
    end: usize,
) -> Result<Vec<[f32; 8]>, RenderBlocker> {
    let first = start as u64 * OWNERSHIP_FRAME_TICKS / NEURAL_STEP;
    let last = (end as u64 * OWNERSHIP_FRAME_TICKS)
        .div_ceil(NEURAL_STEP)
        .min(history.frames);
    let values = history.read_frames(first..last).map_err(RenderBlocker)?;
    Ok((start..end)
        .map(|frame| {
            let left = frame as u64 * OWNERSHIP_FRAME_TICKS / NEURAL_STEP;
            let right = ((frame + 1) as u64 * OWNERSHIP_FRAME_TICKS)
                .div_ceil(NEURAL_STEP)
                .min(history.frames);
            let mut row = [0.0; 8];
            for sample in left..right {
                for channel in 0..8 {
                    row[channel] +=
                        values[((sample - first) * 8) as usize + channel] / (right - left) as f32;
                }
            }
            row
        })
        .collect())
}
fn features(
    track: usize,
    mapping: &SpeakerMapping,
    neural: &[f32; 8],
    vad: &FixedVad,
    acoustics: &crate::TrackAcousticFeatures,
    frame: usize,
    speaker: SpeakerId,
) -> Result<Vec<f64>, RenderBlocker> {
    let channel = mapping.channels[&speaker]
        .ok_or_else(|| RenderBlocker("Unresolved assigned speaker; preserve for review.".into()))?;
    let raw = vad
        .probabilities
        .get(track)
        .and_then(|p| p.get(frame))
        .ok_or_else(|| {
            RenderBlocker("Missing fixed speech evidence; preserve for review.".into())
        })?;
    let mut values = vec![f64::from(neural[channel]), f64::from(*raw)];
    values.extend(acoustics.normalized_values().into_iter().map(f64::from));
    if values.iter().any(|v| !v.is_finite()) {
        return Err(RenderBlocker("Ownership feature is not finite.".into()));
    }
    Ok(values)
}
fn feature_names(project: &Project) -> Vec<Vec<String>> {
    project
        .sources
        .iter()
        .map(|source| {
            let mut names = vec![
                "mapped_own_activity".into(),
                "raw_vad".into(),
                "local_noise_relative_db/60".into(),
                "relative_dominance_db/60".into(),
            ];
            for other in project
                .sources
                .iter()
                .filter(|other| other.track_id != source.track_id)
            {
                for name in [
                    "explanation",
                    "expected_db/60",
                    "excess_db/60",
                    "reliability",
                    "identified_mask",
                ] {
                    names.push(format!("track{}:{name}", other.track_id.0));
                }
            }
            names
        })
        .collect()
}
