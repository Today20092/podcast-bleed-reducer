//! Trusted speaker references checked on Analyze workers, never audio/UI callbacks.
use crate::{
    PreparedAnalysisAudio, Project, ReferenceLabel, RenderBlocker, Rounding, SourceTimebase,
    SpeakerId,
};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

pub const REFERENCE_MAPPING_RECIPE: &str =
    "trusted-solo-mean0.6-margin0.2-coverage0.75-room0.3-one-to-one-v1";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IdentityReview {
    pub speaker: Option<SpeakerId>,
    pub label: Option<u64>,
    pub reason: String,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpeakerMapping {
    pub channels: BTreeMap<SpeakerId, Option<usize>>,
    pub issues: Vec<IdentityReview>,
    pub readiness: ReferenceReadiness,
    pub cache_key: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReferenceReadiness {
    pub missing_speakers: Vec<SpeakerId>,
    pub room_tone: bool,
    pub usable_labels: Vec<u64>,
    pub issues: Vec<String>,
}
impl ReferenceReadiness {
    pub fn ready(&self) -> bool {
        self.missing_speakers.is_empty() && self.room_tone && self.issues.is_empty()
    }
}

pub fn reference_readiness(
    project: &Project,
    audio: &[PreparedAnalysisAudio],
    cancelled: Option<&AtomicBool>,
) -> Result<ReferenceReadiness, RenderBlocker> {
    if audio.len() != project.sources.len()
        || audio.iter().zip(&project.sources).any(|(audio, source)| {
            audio.identity.sample_rate != 16_000
                || audio.identity.source_fingerprint != source.fingerprint
                || audio.identity.source != source.metadata
        })
    {
        return Err(RenderBlocker("Reference audio differs from the current source bindings; prepare current analysis copies.".into()));
    }
    let mut result = ReferenceReadiness {
        missing_speakers: project
            .sources
            .iter()
            .map(|source| source.speaker_id)
            .collect(),
        room_tone: false,
        usable_labels: vec![],
        issues: vec![],
    };
    let mut floors: Vec<Option<f64>> = vec![None; audio.len()];
    for label in project
        .reference_labels()
        .iter()
        .filter(|label| label.active_speakers.is_empty())
    {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        if !project.reference_label_is_current(label) {
            result.issues.push(format!(
                "Reference {} has changed source bindings; relink and review it.",
                label.id
            ));
            continue;
        }
        for (track, source) in project.sources.iter().enumerate() {
            if label.reference_track.is_none_or(|id| id == source.track_id) {
                let rms = reference_rms(&audio[track], label, cancelled)?;
                floors[track] = Some(floors[track].map_or(rms, |old| old.max(rms)));
            }
        }
        result.usable_labels.push(label.id);
    }
    result.room_tone = floors.iter().all(Option::is_some);
    for label in project
        .reference_labels()
        .iter()
        .filter(|label| !label.active_speakers.is_empty())
    {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        if !project.reference_label_is_current(label) {
            result.issues.push(format!(
                "Reference {} has changed source bindings; relink and review it.",
                label.id
            ));
            continue;
        }
        let mut usable = true;
        for speaker in &label.active_speakers {
            let track = project
                .sources
                .iter()
                .position(|source| source.speaker_id == *speaker)
                .ok_or_else(|| RenderBlocker("Unknown reference speaker.".into()))?;
            let rms = reference_rms(&audio[track], label, cancelled)?;
            if rms < 1e-7 || floors[track].is_some_and(|floor| rms <= floor.max(1e-8) * 2.0) {
                result.issues.push(format!("Reference {} lacks usable signal above room tone for Speaker {}; choose another speech example.",label.id,speaker.0));
                usable = false;
            }
        }
        if usable {
            result.usable_labels.push(label.id);
            if label.active_speakers.len() == 1 {
                result
                    .missing_speakers
                    .retain(|speaker| *speaker != label.active_speakers[0]);
            }
        }
    }
    Ok(result)
}

fn reference_rms(
    audio: &PreparedAnalysisAudio,
    label: &ReferenceLabel,
    cancelled: Option<&AtomicBool>,
) -> Result<f64, RenderBlocker> {
    let grid = SourceTimebase::new(audio.identity.sample_rate, audio.identity.frame_count)?;
    let mut start = grid.frame_at_time(label.range.start, Rounding::Ceil)? as usize;
    let end = grid.frame_at_time(label.range.end, Rounding::Floor)? as usize;
    let mut count = 0_u64;
    let mut mean = 0.0;
    let mut square = 0.0;
    while start < end {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let next = start.saturating_add(4096).min(end);
        for sample in audio.read_frames(start..next)? {
            count += 1;
            let delta = f64::from(sample) - mean;
            mean += delta / count as f64;
            square += delta * (f64::from(sample) - mean);
        }
        start = next;
    }
    if count == 0 {
        return Err(RenderBlocker(
            "Reference contains no complete analysis samples.".into(),
        ));
    }
    Ok((square / count as f64).max(0.0).sqrt())
}
