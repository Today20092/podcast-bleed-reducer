//! Trusted references and anonymous diarization channels stay separate from
//! microphone ownership. These checks run on Analyze workers, never audio/UI callbacks.
use crate::{
    PreparedAnalysisAudio, Project, ReferenceLabel, RenderBlocker, Rounding, SourceTimebase,
    SpeakerId,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
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

pub fn resolve_speaker_mapping(
    project: &Project,
    audio: &[PreparedAnalysisAudio],
    evidence: &crate::model::DiarizationEvidence,
    cancelled: Option<&AtomicBool>,
) -> Result<SpeakerMapping, RenderBlocker> {
    crate::prepared_analysis::check_cancelled(cancelled)?;
    if evidence.schema != 2
        || evidence.sources
            != audio
                .iter()
                .map(|audio| audio.identity.clone())
                .collect::<Vec<_>>()
        || evidence.input_sha256
            != audio
                .iter()
                .map(|audio| audio.audio_sha256().to_string())
                .collect::<Vec<_>>()
        || evidence.sample_count
            != audio
                .iter()
                .map(|audio| audio.identity.frame_count)
                .max()
                .unwrap_or(0)
        || evidence.history.frames < evidence.sample_count.div_ceil(160)
        || evidence.history.frames > evidence.sample_count.div_ceil(160) + 1
        || !evidence.reported_seconds_per_frame.is_finite()
        || (evidence.reported_seconds_per_frame - 0.01).abs() > 1e-9
        || !evidence.mix_peak.is_finite()
        || evidence.mix_gains.len() != audio.len()
        || evidence
            .mix_gains
            .iter()
            .any(|gain| !gain.is_finite() || *gain <= 0.0)
    {
        return Err(RenderBlocker(
            "Diarization evidence does not match the complete current analysis copies.".into(),
        ));
    }
    let never_cancel = AtomicBool::new(false);
    evidence
        .history
        .verify_cancelled(cancelled.unwrap_or(&never_cancel))
        .map_err(RenderBlocker)?;
    let readiness = reference_readiness(project, audio, cancelled)?;
    let mut labels = project.reference_labels().to_vec();
    for label in &mut labels {
        label.active_speakers.sort_unstable();
        label.sources.sort_by_key(|source| source.track);
    }
    labels.sort_by_key(|label| label.id);
    let key = serde_json::json!({ "recipe":REFERENCE_MAPPING_RECIPE,"labels":labels,
        "assignments":project.sources.iter().map(|source| (source.track_id,source.speaker_id,&source.fingerprint)).collect::<Vec<_>>(),
        "sources":evidence.sources,"payloads":evidence.input_sha256,"history":evidence.history.sha256,
        "runtime":evidence.runtime,"model":(&evidence.model_revision,&evidence.model_sha256),
        "preset":evidence.preset,"mix":(&evidence.mix_recipe,evidence.mix_peak,&evidence.mix_gains) });
    let mut result = SpeakerMapping {
        channels: project
            .sources
            .iter()
            .map(|source| (source.speaker_id, None))
            .collect(),
        issues: vec![],
        readiness,
        cache_key: format!("{:x}", Sha256::digest(key.to_string().as_bytes())),
    };
    let mut candidates = BTreeMap::new();
    let mut unresolved = BTreeSet::new();
    for label in project
        .reference_labels()
        .iter()
        .filter(|label| result.readiness.usable_labels.contains(&label.id))
    {
        if label.active_speakers.len() > 1 {
            continue;
        }
        let (means, strong, frames) = probability_summary(&evidence.history, label, cancelled)?;
        if label.active_speakers.is_empty() {
            if means.iter().any(|mean| *mean > 0.3) {
                result.readiness.issues.push(format!("Reference {} has speech activity in the selected room-tone example; verify it or select another.",label.id));
            }
            continue;
        }
        let speaker = label.active_speakers[0];
        let mut order: Vec<_> = (0..8).collect();
        order.sort_by(|&a, &b| means[b].total_cmp(&means[a]));
        let channel = order[0];
        let reason = if means[channel] < 0.6
            || means[channel] - means[order[1]] < 0.2
            || strong[channel] as f64 / (frames as f64) < 0.75
        {
            Some(
                "Solo reference has missing, ambiguous or unstable anonymous speaker activity; identity remains unresolved.",
            )
        } else if candidates.get(&speaker).is_some_and(|old| *old != channel) {
            Some(
                "Solo references disagree across the recording; recurrence identity remains unresolved.",
            )
        } else {
            candidates.insert(speaker, channel);
            None
        };
        if let Some(reason) = reason {
            unresolved.insert(speaker);
            result.issues.push(IdentityReview {
                speaker: Some(speaker),
                label: Some(label.id),
                reason: reason.into(),
            });
        }
    }
    for (&speaker, &channel) in &candidates {
        if candidates
            .iter()
            .any(|(other, other_channel)| *other != speaker && *other_channel == channel)
        {
            unresolved.insert(speaker);
            result.issues.push(IdentityReview {speaker:Some(speaker),label:None,reason:"Different assigned speakers match the same anonymous channel; both identities remain unresolved.".into()});
        }
    }
    for (speaker, channel) in &mut result.channels {
        if !unresolved.contains(speaker) {
            *channel = candidates.get(speaker).copied();
        }
        if channel.is_none()
            && !result
                .issues
                .iter()
                .any(|issue| issue.speaker == Some(*speaker))
        {
            result.issues.push(IdentityReview {
                speaker: Some(*speaker),
                label: None,
                reason: "No usable current solo identity reference for this assigned speaker."
                    .into(),
            });
        }
    }
    Ok(result)
}

fn probability_summary(
    history: &crate::model::ProbabilityHistory,
    label: &ReferenceLabel,
    cancelled: Option<&AtomicBool>,
) -> Result<([f64; 8], [u64; 8], u64), RenderBlocker> {
    let cadence = crate::TICKS_PER_SECOND / 100;
    let mut start = label.range.start.div_ceil(cadence);
    let end = (label.range.end / cadence).min(history.frames);
    if start >= end {
        return Err(RenderBlocker(
            "Reference contains no complete diarization frames.".into(),
        ));
    }
    let frames = end - start;
    let mut sums = [0.0; 8];
    let mut strong = [0; 8];
    while start < end {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let next = start.saturating_add(16_384).min(end);
        let values = history.read_frames(start..next).map_err(RenderBlocker)?;
        for row in values.as_chunks::<8>().0 {
            for channel in 0..8 {
                sums[channel] += f64::from(row[channel]);
                if row[channel] >= 0.6
                    && row
                        .iter()
                        .enumerate()
                        .filter(|(other, _)| *other != channel)
                        .all(|(_, other)| row[channel] - *other >= 0.2)
                {
                    strong[channel] += 1;
                }
            }
        }
        start = next;
    }
    Ok((sums.map(|sum| sum / frames as f64), strong, frames))
}
