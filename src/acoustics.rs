//! Reference-derived acoustic evidence; never an audio subtractor or ownership decision.
use crate::joint_detector::JointFacts;
use crate::{
    PreparedAnalysisAudio, Project, RenderBlocker, TICKS_PER_SECOND, TimePosition, TrackId,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};
pub const ACOUSTIC_RECIPE: &str = "speechband32ms-trusted-scalar-or-heldout-ridge2tap-lag12ms-v2";
const STEP: u64 = TICKS_PER_SECOND * 32 / 1000;
#[derive(Clone, Copy, Debug, Default)]
pub struct AcousticCacheStats {
    pub fixed_preparations: u64,
    pub reference_fits: u64,
}
#[derive(Clone, Debug, serde::Serialize)]
pub enum TransferStatus {
    Reliable,
    BelowRoomFloor,
    Unreliable,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct AcousticTransfer {
    pub estimator: String,
    pub source: TrackId,
    pub target: TrackId,
    /// Median target-minus-source speech-band power in dB from trusted solos.
    pub level_db: Option<f32>,
    /// Scalar residual power in dB relative to source; FIR excess uses its own held-out baseline.
    pub residual_db: Option<f32>,
    /// Development fit qualification, not a calibrated ownership probability.
    pub reliability: f32,
    pub status: TransferStatus,
    pub reason: Option<String>,
}
#[derive(Debug)]
struct FixedAcoustics {
    key: String,
    joint: JointFacts,
    tracks: Vec<TrackId>,
    duration: TimePosition,
}
#[derive(Clone, Debug)]
pub struct SessionAcoustics {
    pub cache_key: String,
    pub recipe: String,
    pub development_comparisons: Vec<crate::AcousticEstimatorComparison>,
    fir_features: BTreeMap<(TrackId, TrackId), Vec<f32>>,
    fixed: Arc<FixedAcoustics>,
    floors: Vec<f32>,
    transfers: Vec<AcousticTransfer>,
}
#[derive(Clone, Debug)]
pub struct BleedFeatures {
    pub source: TrackId,
    pub explanation: f32,
    pub expected_level_db: Option<f32>,
    pub excess_residual_db: Option<f32>,
    pub reliability: f32,
}
#[derive(Clone, Debug)]
pub struct TrackAcousticFeatures {
    pub track: TrackId,
    pub local_noise_relative_db: f32,
    pub relative_dominance_db: f32,
    pub bleed: Vec<BleedFeatures>,
}
impl TrackAcousticFeatures {
    pub fn normalized_values(&self) -> Vec<f32> {
        let mut values = vec![
            (self.local_noise_relative_db / 60.0).clamp(-2.0, 2.0),
            (self.relative_dominance_db / 60.0).clamp(-2.0, 2.0),
        ];
        for bleed in &self.bleed {
            values.extend([
                bleed.explanation,
                bleed.expected_level_db.unwrap_or(0.0) / 60.0,
                bleed.excess_residual_db.unwrap_or(0.0) / 60.0,
                bleed.reliability,
                f32::from(bleed.expected_level_db.is_some()),
            ]);
        }
        values
    }
}
impl SessionAcoustics {
    pub fn transfer(&self, source: TrackId, target: TrackId) -> Option<&AcousticTransfer> {
        self.transfers
            .iter()
            .find(|fit| fit.source == source && fit.target == target)
    }
    pub fn features_at(
        &self,
        time: TimePosition,
    ) -> Result<Vec<TrackAcousticFeatures>, RenderBlocker> {
        if time >= self.fixed.duration {
            return Err(RenderBlocker(
                "Acoustic feature position is outside the recording.".into(),
            ));
        }
        let frame = (time / STEP) as usize;
        if self.fixed.joint.acoustic_levels(0).get(frame).is_none() {
            return Err(RenderBlocker("No complete acoustic frame covers the terminal analysis roundoff; preserve this source tail.".into()));
        }
        let levels: Vec<_> = (0..self.fixed.tracks.len())
            .map(|track| self.fixed.joint.acoustic_levels(track)[frame])
            .collect();
        Ok(self
            .fixed
            .tracks
            .iter()
            .enumerate()
            .map(|(target, &track)| {
                let competing = levels
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != target)
                    .map(|(_, level)| *level)
                    .fold(-120.0_f32, f32::max);
                let bleed = self
                    .fixed
                    .tracks
                    .iter()
                    .enumerate()
                    .filter(|(source, _)| *source != target)
                    .map(|(source, &id)| {
                        let explained = self.fixed.joint.acoustic_pair(source, target).unwrap()
                            [frame]
                            .clamp(0.0, 1.0);
                        let fit = self.transfer(id, track).unwrap();
                        let residual = levels[target] - levels[source] + db(1.0 - explained);
                        BleedFeatures {
                            source: id,
                            explanation: explained,
                            expected_level_db: fit.level_db,
                            excess_residual_db: self
                                .fir_features
                                .get(&(id, track))
                                .map(|frames| frames[frame])
                                .or_else(|| {
                                    fit.residual_db
                                        .map(|baseline| (residual - baseline).clamp(-120.0, 120.0))
                                }),
                            reliability: fit.reliability,
                        }
                    })
                    .collect();
                TrackAcousticFeatures {
                    track,
                    local_noise_relative_db: (levels[target] - self.floors[target])
                        .clamp(-120.0, 120.0),
                    relative_dominance_db: (levels[target] - competing).clamp(-120.0, 120.0),
                    bleed,
                }
            })
            .collect())
    }
}
#[derive(Default)]
pub struct AcousticEvidenceCache {
    fixed: Option<Arc<FixedAcoustics>>,
    derived: Option<Arc<SessionAcoustics>>,
    stats: AcousticCacheStats,
}
impl AcousticEvidenceCache {
    pub fn stats(&self) -> AcousticCacheStats {
        self.stats
    }
    /// Reuses immutable completed evidence after revalidating the current sources and labels.
    pub fn analyze(
        &mut self,
        project: &Project,
        audio: &[PreparedAnalysisAudio],
        cancelled: Option<&AtomicBool>,
    ) -> Result<Arc<SessionAcoustics>, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        project.require_alignment()?;
        for copy in audio {
            if crate::project::fingerprint_cancellable(copy.path(), cancelled)?
                != copy.audio_sha256()
            {
                return Err(RenderBlocker("Analysis payload changed; regenerate current analysis copies before acoustic fitting.".into()));
            }
        }
        let readiness = crate::reference_readiness(project, audio, cancelled)?;
        if !readiness.ready() {
            return Err(RenderBlocker(format!(
                "Usable solo references for every speaker and room tone are required: {} missing speakers; {}",
                readiness.missing_speakers.len(),
                readiness.issues.join("; ")
            )));
        }
        let key = hash(
            serde_json::json!({"recipe":ACOUSTIC_RECIPE,"sources":audio.iter().map(|copy|(&copy.identity,copy.audio_sha256())).collect::<Vec<_>>(),"tracks":project.sources.iter().map(|source|source.track_id).collect::<Vec<_>>() }),
        );
        let fixed = if let Some(fixed) = &self.fixed
            && fixed.key == key
        {
            fixed.clone()
        } else {
            Arc::new(FixedAcoustics {
                key,
                joint: JointFacts::prepare_analysis_tracks(audio, cancelled)?,
                tracks: project
                    .sources
                    .iter()
                    .map(|source| source.track_id)
                    .collect(),
                duration: project
                    .sources
                    .iter()
                    .map(|source| source.metadata.grid.duration())
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .min()
                    .unwrap_or(0),
            })
        };
        let derived_key = hash(
            serde_json::json!({"fixed":fixed.key,"labels":project.reference_labels(),"speakers":project.sources.iter().map(|source|(source.track_id,source.speaker_id)).collect::<Vec<_>>() }),
        );
        if let Some(derived) = &self.derived
            && derived.cache_key == derived_key
        {
            return Ok(derived.clone());
        }
        let mut rooms = vec![vec![]; audio.len()];
        let mut solos = BTreeMap::<TrackId, Vec<usize>>::new();
        for label in project.reference_labels() {
            let frames =
                label.range.start.div_ceil(STEP) as usize..(label.range.end / STEP) as usize;
            if label.active_speakers.is_empty() {
                for (track, source) in project.sources.iter().enumerate() {
                    if label.reference_track.is_none_or(|id| id == source.track_id) {
                        rooms[track].extend(
                            frames
                                .clone()
                                .map(|frame| fixed.joint.acoustic_levels(track)[frame]),
                        );
                    }
                }
            } else if label.active_speakers.len() == 1 {
                let source = project
                    .sources
                    .iter()
                    .find(|source| source.speaker_id == label.active_speakers[0])
                    .unwrap();
                solos.entry(source.track_id).or_default().extend(frames);
            }
        }
        let floors: Vec<_> = rooms
            .into_iter()
            .map(|values| median(values).unwrap_or(-120.0).max(-120.0))
            .collect();
        let mut transfers = vec![];
        let mut candidates = vec![];
        let mut comparisons = vec![];
        for (source, &id) in fixed.tracks.iter().enumerate() {
            crate::prepared_analysis::check_cancelled(cancelled)?;
            let mut selected = solos.get(&id).cloned().unwrap_or_default();
            selected.sort_unstable();
            selected.dedup();
            for (target, &other) in fixed
                .tracks
                .iter()
                .enumerate()
                .filter(|(target, _)| *target != source)
            {
                let selected: Vec<_> = selected
                    .iter()
                    .copied()
                    .filter(|&frame| {
                        fixed.joint.acoustic_levels(source)[frame] > floors[source] + 6.0
                    })
                    .collect();
                let mut fit = AcousticTransfer {
                    estimator: "unavailable".into(),
                    source: id,
                    target: other,
                    level_db: None,
                    residual_db: None,
                    reliability: 0.0,
                    status: TransferStatus::Unreliable,
                    reason: Some("Insufficient reliable solo acoustic evidence.".into()),
                };
                if selected.len() >= 3 {
                    let levels: Vec<_> = selected
                        .iter()
                        .map(|&frame| {
                            fixed.joint.acoustic_levels(target)[frame]
                                - fixed.joint.acoustic_levels(source)[frame]
                        })
                        .collect();
                    let explanations: Vec<_> = selected
                        .iter()
                        .map(|&frame| fixed.joint.acoustic_pair(source, target).unwrap()[frame])
                        .collect();
                    let explained = median(explanations).unwrap();
                    let level = median(levels.clone()).unwrap();
                    let spread = levels
                        .iter()
                        .map(|value| (value - level).abs())
                        .fold(0.0_f32, f32::max);
                    if selected.iter().all(|&frame| {
                        fixed.joint.acoustic_levels(target)[frame] <= floors[target] + 3.0
                    }) {
                        fit.status = TransferStatus::BelowRoomFloor;
                        fit.reliability = 1.0;
                        fit.reason=Some("Leakage is below the measured room floor; a coefficient is not identified.".into());
                    } else if explained >= 0.75
                        && spread <= 6.0
                        && explained * (1.0 - spread / 12.0) >= 0.7
                    {
                        fit.level_db = Some(level.clamp(-120.0, 120.0));
                        fit.residual_db = median(
                            selected
                                .iter()
                                .map(|&frame| {
                                    fixed.joint.acoustic_levels(target)[frame]
                                        - fixed.joint.acoustic_levels(source)[frame]
                                        + db(1.0
                                            - fixed.joint.acoustic_pair(source, target).unwrap()
                                                [frame])
                                })
                                .collect(),
                        )
                        .map(|value| value.clamp(-120.0, 120.0));
                        fit.reliability = (explained * (1.0 - spread / 12.0)).clamp(0.0, 1.0);
                        fit.status = TransferStatus::Reliable;
                        fit.estimator = "scalar-with-delayed-pair-evidence".into();
                        fit.reason = None;
                    }
                }
                if let Some(mut candidate) =
                    crate::acoustic_transfer::qualify(project, audio, source, target, cancelled)?
                {
                    if selected.len() < 3 || matches!(fit.status, TransferStatus::BelowRoomFloor) {
                        candidate.comparison.fir_selected = false;
                    }
                    if candidate.comparison.fir_selected {
                        fit.estimator = "regularized-two-tap-transfer".into();
                        fit.status = TransferStatus::Reliable;
                        fit.reason = None;
                        fit.reliability = candidate.comparison.worst_reference_explained;
                        if fit.level_db.is_none() {
                            let differences = selected
                                .iter()
                                .map(|&frame| {
                                    fixed.joint.acoustic_levels(target)[frame]
                                        - fixed.joint.acoustic_levels(source)[frame]
                                })
                                .collect();
                            fit.level_db =
                                median(differences).map(|value| value.clamp(-120.0, 120.0));
                        }
                        comparisons.push(candidate.comparison.clone());
                        candidates.push((source, target, candidate));
                    } else {
                        comparisons.push(candidate.comparison);
                    }
                }
                transfers.push(fit);
            }
        }
        crate::prepared_analysis::check_cancelled(cancelled)?;
        if project
            .reference_labels()
            .iter()
            .any(|label| !project.reference_label_is_current(label))
        {
            return Err(RenderBlocker(
                "Source changed during acoustic fitting; relink and review references.".into(),
            ));
        }
        let residuals = crate::acoustic_transfer::residual_frames(audio, &candidates, cancelled)?;
        let fir_features = candidates
            .iter()
            .zip(residuals)
            .map(|((source, target, _), frames)| {
                ((fixed.tracks[*source], fixed.tracks[*target]), frames)
            })
            .collect();
        for copy in audio {
            if crate::project::fingerprint_cancellable(copy.path(), cancelled)?
                != copy.audio_sha256()
            {
                return Err(RenderBlocker(
                    "Analysis payload changed during acoustic fitting; regenerate analysis copies."
                        .into(),
                ));
            }
        }
        let result = Arc::new(SessionAcoustics {
            development_comparisons: comparisons,
            fir_features,
            cache_key: derived_key,
            recipe: ACOUSTIC_RECIPE.into(),
            fixed: fixed.clone(),
            floors,
            transfers,
        });
        if self.fixed.as_ref().is_none_or(|old| old.key != fixed.key) {
            self.stats.fixed_preparations += 1;
        }
        self.stats.reference_fits += 1;
        self.fixed = Some(fixed);
        self.derived = Some(result.clone());
        Ok(result)
    }
}
fn median(mut values: Vec<f32>) -> Option<f32> {
    if values.is_empty() {
        None
    } else {
        values.sort_by(f32::total_cmp);
        Some(values[values.len() / 2])
    }
}
fn db(power: f32) -> f32 {
    10.0 * power.max(1e-12).log10()
}
fn hash(value: serde_json::Value) -> String {
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}
