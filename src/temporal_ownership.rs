//! Conservative offline context; probability evidence is immutable and gain-independent.
use crate::{OWNERSHIP_FRAME_TICKS, RenderBlocker, TrackId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, atomic::AtomicBool};

pub const TEMPORAL_RECIPE: &str = "independent-bands-consistent-activity-gap-preservation-v1";
#[derive(Clone, Debug, Serialize)]
pub struct TemporalEvidence {
    pub probability: Option<f32>,
    pub raw_speech: Option<f32>,
    pub assigned_activity: Option<f32>,
    pub preservation_reason: Option<String>,
}
#[derive(Clone, Debug)]
pub struct TemporalEvidenceGrid {
    tracks: Vec<TrackId>,
    duration: u64,
    rows: Vec<Vec<TemporalEvidence>>,
    global_activity: Vec<f32>,
    identity: String,
}
impl TemporalEvidenceGrid {
    pub fn new(
        tracks: Vec<TrackId>,
        duration: u64,
        rows: Vec<Vec<TemporalEvidence>>,
        global_activity: Vec<f32>,
    ) -> Result<Self, RenderBlocker> {
        let frames = usize::try_from(duration.div_ceil(OWNERSHIP_FRAME_TICKS))
            .map_err(|_| RenderBlocker("Ownership timeline is too large.".into()))?;
        if duration == 0
            || tracks.is_empty()
            || tracks.len() > 8
            || rows.len() != tracks.len()
            || tracks
                .iter()
                .enumerate()
                .any(|(i, id)| tracks[..i].contains(id))
            || rows.iter().any(|r| r.len() != frames)
            || global_activity.len() != frames
            || rows.iter().flatten().any(|r| {
                [r.probability, r.raw_speech, r.assigned_activity]
                    .into_iter()
                    .flatten()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
            })
            || global_activity
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err(RenderBlocker(
                "Invalid source-time ownership evidence.".into(),
            ));
        }
        let bytes = serde_json::to_vec(&(&tracks, duration, &rows, &global_activity))
            .map_err(|e| RenderBlocker(e.to_string()))?;
        let identity = format!("{:x}", Sha256::digest(bytes));
        Ok(Self {
            tracks,
            duration,
            rows,
            global_activity,
            identity,
        })
    }
    pub fn duration(&self) -> u64 {
        self.duration
    }
    pub(crate) fn bind_upstream_identity(&mut self, identity: &str) {
        self.identity = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&self.identity, crate::OWNERSHIP_FEATURE_RECIPE, identity))
                    .unwrap()
            )
        );
    }
    pub fn evidence_at(&self, time: u64) -> Result<Vec<TemporalEvidence>, RenderBlocker> {
        if time >= self.duration {
            return Err(RenderBlocker(
                "Evidence query is outside the native recording.".into(),
            ));
        }
        Ok(self
            .rows
            .iter()
            .map(|rows| rows[(time / OWNERSHIP_FRAME_TICKS) as usize].clone())
            .collect())
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TemporalSettings {
    pub owner_high: f32,
    pub owner_low: f32,
    pub activity_threshold: f32,
    pub quiet_threshold: f32,
    pub preserved_gap_frames: usize,
}
impl Default for TemporalSettings {
    fn default() -> Self {
        Self {
            owner_high: 0.8,
            owner_low: 0.2,
            activity_threshold: 0.5,
            quiet_threshold: 0.2,
            preserved_gap_frames: 2,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalTrackState {
    Owned,
    Absent,
    UncertainUnity,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalWarning {
    pub start: u64,
    pub end: u64,
    pub tracks: Vec<TrackId>,
    pub reason: String,
}
#[derive(Clone, Debug)]
pub struct TemporalOwnership {
    pub cache_key: String,
    pub duration: u64,
    pub warnings: Vec<TemporalWarning>,
    pub tracks: Vec<TrackId>,
    states: Vec<Vec<TemporalTrackState>>,
    evidence: Arc<TemporalEvidenceGrid>,
    pub regions: Vec<TemporalRegion>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalRegion {
    pub start: u64,
    pub end: u64,
    pub states: Vec<TemporalTrackState>,
    pub human_truth: bool,
}
impl TemporalOwnership {
    pub fn states_at(&self, time: u64) -> Result<Vec<TemporalTrackState>, RenderBlocker> {
        if time >= self.duration {
            return Err(RenderBlocker(
                "Ownership query is outside the native recording.".into(),
            ));
        }
        let index = self.regions.partition_point(|region| region.end <= time);
        if let Some(region) = self
            .regions
            .get(index)
            .filter(|region| region.start <= time && time < region.end)
        {
            return Ok(region.states.clone());
        }
        Ok(self
            .states
            .iter()
            .map(|rows| rows[(time / OWNERSHIP_FRAME_TICKS) as usize])
            .collect())
    }
    pub fn evidence(&self) -> &TemporalEvidenceGrid {
        &self.evidence
    }
    /// Conservative adapter for the retained renderer: any unresolved track preserves the region.
    pub fn engine_regions(&self) -> Vec<crate::Region> {
        self.regions
            .iter()
            .map(|region| crate::Region {
                start: region.start,
                end: region.end,
                decision: if region.states.contains(&TemporalTrackState::UncertainUnity) {
                    crate::Decision::Uncertain
                } else {
                    let owners: Vec<_> = region
                        .states
                        .iter()
                        .enumerate()
                        .filter_map(|(i, s)| (*s == TemporalTrackState::Owned).then_some(i))
                        .collect();
                    if owners.is_empty() {
                        crate::Decision::NoSpeech
                    } else {
                        crate::Decision::Owned(owners)
                    }
                },
            })
            .collect()
    }
}
#[derive(Default)]
pub struct TemporalOwnershipCache {
    accepted: Option<Arc<TemporalOwnership>>,
}
impl TemporalOwnershipCache {
    pub fn last_completed(&self) -> Option<&Arc<TemporalOwnership>> {
        self.accepted.as_ref()
    }
    /// Worker boundary: current source audio and authoritative labels are checked before reuse and installation.
    pub fn optimize(
        &mut self,
        project: &crate::Project,
        ownership: &crate::SessionOwnership,
        settings: TemporalSettings,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Arc<TemporalOwnership>, RenderBlocker> {
        project
            .validate_sources_cancellable(cancelled)
            .map_err(|e| RenderBlocker(e.to_string()))?;
        if project
            .reference_labels()
            .iter()
            .any(|label| !project.reference_label_is_current(label))
        {
            return Err(RenderBlocker(
                "Review changed reference source bindings before analysis.".into(),
            ));
        }
        let grid = ownership.temporal_evidence_cancellable(project, cancelled)?;
        self.optimize_project_evidence(project, &grid, settings, cancelled)
    }
    /// Supplied evidence is a development/public integration seam, not a model or source validation substitute.
    pub fn optimize_project_evidence(
        &mut self,
        project: &crate::Project,
        grid: &TemporalEvidenceGrid,
        settings: TemporalSettings,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Arc<TemporalOwnership>, RenderBlocker> {
        project
            .require_alignment()
            .map_err(|e| RenderBlocker(e.to_string()))?;
        project
            .validate_sources_cancellable(cancelled)
            .map_err(|e| RenderBlocker(e.to_string()))?;
        let duration = project
            .sources
            .iter()
            .map(|s| s.metadata.grid.duration())
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .unwrap_or(0);
        if grid.duration != duration
            || grid.tracks
                != project
                    .sources
                    .iter()
                    .map(|s| s.track_id)
                    .collect::<Vec<_>>()
            || project
                .reference_labels()
                .iter()
                .any(|label| !project.reference_label_is_current(label))
        {
            return Err(RenderBlocker(
                "Temporal evidence differs from current source bindings.".into(),
            ));
        }
        let binding = crate::SessionOwnership::reference_identity(project);
        let key = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(TEMPORAL_RECIPE, &grid.identity, &binding, settings)).unwrap()
            )
        );
        if let Some(old) = &self.accepted
            && old.cache_key == key
        {
            crate::prepared_analysis::check_cancelled(cancelled)?;
            return Ok(old.clone());
        }
        let mut provisional = Self::default();
        let computed = provisional.optimize_evidence(grid, settings, cancelled)?;
        let mut result = (*computed).clone();
        result.cache_key = key;
        let mut boundaries = std::collections::BTreeSet::from([0, duration]);
        for frame in 0..grid.global_activity.len() {
            boundaries.insert(frame as u64 * OWNERSHIP_FRAME_TICKS);
        }
        for label in project.reference_labels() {
            boundaries.insert(label.range.start);
            boundaries.insert(label.range.end);
        }
        let boundaries: Vec<_> = boundaries.into_iter().collect();
        let mut labels: Vec<_> = project.reference_labels().iter().collect();
        labels.sort_by_key(|label| label.range.start);
        let mut truth_ranges: Vec<(u64, u64, Vec<crate::SpeakerId>)> = vec![];
        for label in labels {
            if let Some(last) = truth_ranges
                .last_mut()
                .filter(|last| label.range.start < last.1)
            {
                if last.2 != label.active_speakers {
                    return Err(RenderBlocker("Contradictory human ownership constraints; replace the conflicting reference.".into()));
                }
                last.1 = last.1.max(label.range.end);
            } else {
                truth_ranges.push((
                    label.range.start,
                    label.range.end,
                    label.active_speakers.clone(),
                ));
            }
        }
        let mut truth_index = 0;
        result.regions.clear();
        for (index, range) in boundaries.windows(2).enumerate() {
            if index % 1024 == 0 {
                crate::prepared_analysis::check_cancelled(cancelled)?;
            }
            if range[0] >= duration {
                continue;
            }
            while truth_index < truth_ranges.len() && truth_ranges[truth_index].1 <= range[0] {
                truth_index += 1;
            }
            let truth = truth_ranges
                .get(truth_index)
                .filter(|truth| truth.0 <= range[0]);
            let states = if let Some(label) = truth {
                project
                    .sources
                    .iter()
                    .map(|source| {
                        if label.2.contains(&source.speaker_id) {
                            TemporalTrackState::Owned
                        } else {
                            TemporalTrackState::Absent
                        }
                    })
                    .collect()
            } else {
                computed.states_at(range[0])?
            };
            let region = TemporalRegion {
                start: range[0],
                end: range[1].min(duration),
                states,
                human_truth: truth.is_some(),
            };
            if let Some(last) = result.regions.last_mut().filter(|last| {
                last.end == region.start
                    && last.states == region.states
                    && last.human_truth == region.human_truth
            }) {
                last.end = region.end;
            } else {
                result.regions.push(region);
            }
        }
        // Human truth only removes statistical warnings inside its exact half-open range.
        result.warnings.clear();
        for (index, warning) in computed.warnings.iter().enumerate() {
            if index % 1024 == 0 {
                crate::prepared_analysis::check_cancelled(cancelled)?;
            }
            let first = result.regions.partition_point(|r| r.end <= warning.start);
            for region in result.regions[first..]
                .iter()
                .take_while(|r| r.start < warning.end)
                .filter(|r| !r.human_truth)
            {
                let clipped = TemporalWarning {
                    start: region.start.max(warning.start),
                    end: region.end.min(warning.end),
                    tracks: warning.tracks.clone(),
                    reason: warning.reason.clone(),
                };
                if let Some(last) = result.warnings.last_mut().filter(|last| {
                    last.end == clipped.start
                        && last.tracks == clipped.tracks
                        && last.reason == clipped.reason
                }) {
                    last.end = clipped.end;
                } else {
                    result.warnings.push(clipped);
                }
            }
        }
        project
            .validate_sources_cancellable(cancelled)
            .map_err(|e| RenderBlocker(e.to_string()))?;
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let result = Arc::new(result);
        self.accepted = Some(result.clone());
        Ok(result)
    }
    pub fn optimize_evidence(
        &mut self,
        grid: &TemporalEvidenceGrid,
        settings: TemporalSettings,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Arc<TemporalOwnership>, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        if [
            settings.owner_high,
            settings.owner_low,
            settings.activity_threshold,
            settings.quiet_threshold,
        ]
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || settings.owner_low >= settings.owner_high
            || settings.quiet_threshold >= settings.activity_threshold
            || settings.preserved_gap_frames > 32
        {
            return Err(RenderBlocker("Invalid temporal ownership settings.".into()));
        }
        let key = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(TEMPORAL_RECIPE, &grid.identity, settings)).unwrap()
            )
        );
        if let Some(old) = &self.accepted
            && old.cache_key == key
        {
            return Ok(old.clone());
        }
        let frames = grid.global_activity.len();
        let mut states = vec![vec![TemporalTrackState::Absent; frames]; grid.tracks.len()];
        let mut reasons = vec![vec![None; frames]; grid.tracks.len()];
        for (track, rows) in grid.rows.iter().enumerate() {
            for (frame, row) in rows.iter().enumerate() {
                if frame % 1024 == 0 {
                    crate::prepared_analysis::check_cancelled(cancelled)?;
                }
                let quiet_global = grid.global_activity[frame] <= settings.quiet_threshold;
                let (state, reason) = match (row.probability, row.raw_speech, row.assigned_activity)
                {
                    _ if row.preservation_reason.is_some() => (
                        TemporalTrackState::UncertainUnity,
                        row.preservation_reason.clone(),
                    ),
                    (Some(p), Some(raw), Some(activity))
                        if p >= settings.owner_high
                            && activity >= settings.activity_threshold
                            && raw >= settings.activity_threshold =>
                    {
                        (TemporalTrackState::Owned, None)
                    }
                    (Some(p), Some(raw), Some(activity))
                        if p <= settings.owner_low
                            && activity <= settings.quiet_threshold
                            && (raw <= settings.quiet_threshold || !quiet_global) =>
                    {
                        (TemporalTrackState::Absent, None)
                    }
                    _ => (
                        TemporalTrackState::UncertainUnity,
                        Some("Ownership and speech evidence need review.".into()),
                    ),
                };
                states[track][frame] = state;
                reasons[track][frame] = reason;
            }
            // Context may preserve an interior gap; it can never erase a positive frame.
            let owned: Vec<_> = states[track]
                .iter()
                .enumerate()
                .filter_map(|(i, s)| (*s == TemporalTrackState::Owned).then_some(i))
                .collect();
            for edges in owned.windows(2) {
                if edges[1] - edges[0] - 1 <= settings.preserved_gap_frames {
                    for frame in edges[0] + 1..edges[1] {
                        if states[track][frame] == TemporalTrackState::Absent {
                            states[track][frame] = TemporalTrackState::UncertainUnity;
                            reasons[track][frame] =
                                Some("Preserve the brief gap between speech regions.".into());
                        }
                    }
                }
            }
        }
        // Empty ownership is insufficient to prove silence when any evidence remains active.
        for frame in 0..frames {
            if frame % 1024 == 0 {
                crate::prepared_analysis::check_cancelled(cancelled)?;
            }
            if states
                .iter()
                .all(|rows| rows[frame] == TemporalTrackState::Absent)
                && (grid.global_activity[frame] > settings.quiet_threshold
                    || grid.rows.iter().any(|rows| {
                        rows[frame]
                            .raw_speech
                            .is_none_or(|p| p > settings.quiet_threshold)
                    }))
            {
                for track in 0..states.len() {
                    states[track][frame] = TemporalTrackState::UncertainUnity;
                    reasons[track][frame] =
                        Some("Speech is present without a reliable owner.".into());
                }
            }
        }
        let mut warnings: Vec<TemporalWarning> = vec![];
        for (track, rows) in reasons.iter().enumerate() {
            for (frame, reason) in rows.iter().enumerate() {
                if frame % 1024 == 0 {
                    crate::prepared_analysis::check_cancelled(cancelled)?;
                }
                if let Some(reason) = reason {
                    let start = frame as u64 * OWNERSHIP_FRAME_TICKS;
                    let end = (start + OWNERSHIP_FRAME_TICKS).min(grid.duration);
                    if let Some(previous) = warnings.last_mut().filter(|w| {
                        w.end == start
                            && w.reason == *reason
                            && w.tracks == vec![grid.tracks[track]]
                    }) {
                        previous.end = end;
                    } else {
                        warnings.push(TemporalWarning {
                            start,
                            end,
                            tracks: vec![grid.tracks[track]],
                            reason: reason.clone(),
                        });
                    }
                }
            }
        }
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let regions = (0..frames)
            .map(|frame| TemporalRegion {
                start: frame as u64 * OWNERSHIP_FRAME_TICKS,
                end: ((frame + 1) as u64 * OWNERSHIP_FRAME_TICKS).min(grid.duration),
                states: states.iter().map(|rows| rows[frame]).collect(),
                human_truth: false,
            })
            .collect();
        let result = Arc::new(TemporalOwnership {
            cache_key: key,
            duration: grid.duration,
            warnings,
            tracks: grid.tracks.clone(),
            states,
            evidence: Arc::new(grid.clone()),
            regions,
        });
        crate::prepared_analysis::check_cancelled(cancelled)?;
        self.accepted = Some(result.clone());
        Ok(result)
    }
}
