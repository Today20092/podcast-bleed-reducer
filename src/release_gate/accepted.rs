//! Frozen held-out truth is read by evaluation, never by analysis or training.
use crate::{Project, RenderBlocker, SourceTimebase, TimeRange, TrackId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::atomic::AtomicBool};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationClaim {
    Controlled,
    KnownCase,
    Independent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationKind {
    Preserve,
    Bleed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationCase {
    QuietSpeech,
    Interruption,
    Laughter,
    Overlap,
    RoomTone,
    Movement,
    ChangingDelay,
    HistoricalFailure,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvaluationSource {
    track: TrackId,
    fingerprint: String,
    grid: SourceTimebase,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputLabel {
    track: TrackId,
    start: u64,
    end: u64,
    kind: EvaluationKind,
    case: EvaluationCase,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputManifest {
    version: u32,
    claim: EvaluationClaim,
    provenance: String,
    prior_use: Vec<String>,
    reference_identity: String,
    sources: Vec<EvaluationSource>,
    labels: Vec<InputLabel>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EvaluationLabel {
    pub track: TrackId,
    pub range: TimeRange,
    pub kind: EvaluationKind,
    pub cases: BTreeSet<EvaluationCase>,
}

pub struct FrozenEvaluationManifest {
    input: InputManifest,
    labels: Vec<EvaluationLabel>,
    digest: String,
}
impl FrozenEvaluationManifest {
    pub fn reference_identity(project: &Project) -> String {
        crate::SessionOwnership::reference_identity(project)
    }
    pub fn read(
        project: &Project,
        json: &str,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Self, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let input: InputManifest =
            serde_json::from_str(json).map_err(|e| RenderBlocker(e.to_string()))?;
        if input.version != 1
            || input.provenance.trim().is_empty()
            || input.labels.is_empty()
            || input.prior_use.iter().any(|use_| use_.trim().is_empty())
            || (input.claim == EvaluationClaim::Independent && !input.prior_use.is_empty())
        {
            return Err(RenderBlocker("Evaluation provenance is missing, unsupported or previously used material is claimed independent.".into()));
        }
        project.require_alignment()?;
        project.validate_sources_cancellable(cancelled)?;
        if input.reference_identity != Self::reference_identity(project)
            || input.sources.len() != project.sources.len()
        {
            return Err(RenderBlocker(
                "Frozen references or source set changed.".into(),
            ));
        }
        let mut tracks = BTreeSet::new();
        for source in &input.sources {
            let actual = project
                .sources
                .iter()
                .find(|s| s.track_id == source.track)
                .ok_or_else(|| RenderBlocker("Evaluation source has an unknown track.".into()))?;
            if !tracks.insert(source.track)
                || source.fingerprint != actual.fingerprint
                || source.grid != actual.metadata.grid
            {
                return Err(RenderBlocker(
                    "Frozen source identity or native grid changed.".into(),
                ));
            }
        }
        let mut labels = Vec::<EvaluationLabel>::new();
        for (index, label) in input.labels.iter().enumerate() {
            if index % 256 == 0 {
                crate::prepared_analysis::check_cancelled(cancelled)?;
            }
            let source = project
                .sources
                .iter()
                .find(|s| s.track_id == label.track)
                .ok_or_else(|| RenderBlocker("Evaluation label has an unknown track.".into()))?;
            let range = TimeRange::new(label.start, label.end)?;
            range.native_frames(source.metadata.grid)?;
            if project
                .reference_labels()
                .iter()
                .any(|r| r.range.start < range.end && range.start < r.range.end)
            {
                return Err(RenderBlocker(
                    "Held-out truth overlaps development references or forced optimizer truth."
                        .into(),
                ));
            }
            labels.push(EvaluationLabel {
                track: label.track,
                range,
                kind: label.kind,
                cases: BTreeSet::from([label.case]),
            });
        }
        labels.sort_by_key(|label| (label.track, label.range.start, label.range.end));
        let mut normalized = Vec::<EvaluationLabel>::new();
        for label in labels {
            if let Some(previous) = normalized.last_mut()
                && previous.track == label.track
                && previous.range.end >= label.range.start
            {
                if previous.kind != label.kind && previous.range.end > label.range.start {
                    return Err(RenderBlocker(
                        "Preserve and bleed truth contradict on the same physical source interval."
                            .into(),
                    ));
                }
                if previous.kind == label.kind {
                    previous.range.end = previous.range.end.max(label.range.end);
                    previous.cases.extend(label.cases);
                    continue;
                }
            }
            normalized.push(label);
        }
        for source in &project.sources {
            native_intervals(&normalized, source.track_id, source.metadata.grid)?;
        }
        let digest = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&input).map_err(|e| RenderBlocker(e.to_string()))?)
        );
        Ok(Self {
            input,
            labels: normalized,
            digest,
        })
    }
    pub fn labels(&self) -> &[EvaluationLabel] {
        &self.labels
    }
    pub fn digest(&self) -> &str {
        &self.digest
    }
    pub fn claim(&self) -> EvaluationClaim {
        self.input.claim
    }
    pub fn validate(
        &self,
        project: &Project,
        cancelled: Option<&AtomicBool>,
    ) -> Result<(), RenderBlocker> {
        Self::read(
            project,
            &serde_json::to_string(&self.input).map_err(|e| RenderBlocker(e.to_string()))?,
            cancelled,
        )
        .map(|_| ())
    }
}

fn native_intervals(
    labels: &[EvaluationLabel],
    track: TrackId,
    grid: SourceTimebase,
) -> Result<Vec<(std::ops::Range<usize>, EvaluationKind)>, RenderBlocker> {
    let mut intervals = Vec::<(std::ops::Range<usize>, EvaluationKind)>::new();
    for label in labels.iter().filter(|l| l.track == track) {
        let frames = label.range.native_frames(grid)?;
        if let Some((previous, kind)) = intervals.last_mut()
            && previous.end >= frames.start
        {
            if *kind != label.kind && previous.end > frames.start {
                return Err(RenderBlocker(
                    "Preserve and bleed truth contradict on the same native sample.".into(),
                ));
            }
            if *kind == label.kind {
                previous.end = previous.end.max(frames.end);
                continue;
            }
        }
        intervals.push((frames, label.kind));
    }
    Ok(intervals)
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct AppliedGainEvidence {
    pub tracks: Vec<TrackGainEvidence>,
    pub preservation_samples: u64,
    pub attenuated_preservation_samples: u64,
    /// Distinct simultaneously preserved tracks, measured on the integer shared clock.
    pub overlap_ticks: u64,
    pub attenuated_overlap_ticks: u64,
    pub bleed_samples: u64,
    pub bleed_input_rms: f64,
    pub bleed_output_rms: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrackGainEvidence {
    pub track: TrackId,
    pub grid: SourceTimebase,
    pub preservation_samples: u64,
    pub attenuated_preservation_samples: u64,
    pub bleed_samples: u64,
    pub bleed_input_rms: f64,
    pub bleed_output_rms: f64,
}

/// Measure applied gain even where PCM is zero; source decoding stays bounded.
pub fn measure_applied_gain(
    project: &Project,
    manifest: &FrozenEvaluationManifest,
    envelopes: &[crate::GainEnvelope],
    cancelled: Option<&AtomicBool>,
) -> Result<AppliedGainEvidence, RenderBlocker> {
    manifest.validate(project, cancelled)?;
    if envelopes.len() != project.sources.len()
        || envelopes
            .iter()
            .zip(&project.sources)
            .any(|(e, s)| e.source_grid() != s.metadata.grid)
    {
        return Err(RenderBlocker(
            "Evaluation envelopes do not match the frozen native sources.".into(),
        ));
    }
    let mut result = AppliedGainEvidence::default();
    let (mut input_square, mut output_square) = (0.0_f64, 0.0_f64);
    let mut changed = Vec::<TimeRange>::new();
    for (track, source) in project.sources.iter().enumerate() {
        let mut measured = TrackGainEvidence {
            track: source.track_id,
            grid: source.metadata.grid,
            preservation_samples: 0,
            attenuated_preservation_samples: 0,
            bleed_samples: 0,
            bleed_input_rms: 0.0,
            bleed_output_rms: 0.0,
        };
        let (mut track_input_square, mut track_output_square) = (0.0_f64, 0.0_f64);
        let intervals = native_intervals(&manifest.labels, source.track_id, source.metadata.grid)?;
        let mut position = 0_usize;
        let mut interval = 0_usize;
        let mut changed_start = None;
        let decoded = crate::audio_io::decode_chunks(&source.path, cancelled, |rate, samples| {
            if rate != source.metadata.grid.sample_rate {
                return Err(RenderBlocker("Decoded source rate changed.".into()));
            }
            for &sample in samples {
                while intervals
                    .get(interval)
                    .is_some_and(|(range, _)| range.end <= position)
                {
                    interval += 1;
                }
                let kind = intervals
                    .get(interval)
                    .filter(|(range, _)| range.contains(&position))
                    .map(|(_, kind)| *kind);
                let mut attenuated = false;
                if let Some(kind) = kind {
                    let gain = envelopes[track].gain_at(position);
                    if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
                        return Err(RenderBlocker(
                            "Evaluation gain is not finite attenuation.".into(),
                        ));
                    }
                    match kind {
                        EvaluationKind::Preserve => {
                            measured.preservation_samples += 1;
                            if gain != 1.0 {
                                measured.attenuated_preservation_samples += 1;
                                attenuated = true;
                            }
                        }
                        EvaluationKind::Bleed => {
                            measured.bleed_samples += 1;
                            track_input_square += f64::from(sample).powi(2);
                            track_output_square += (f64::from(sample) * f64::from(gain)).powi(2);
                        }
                    }
                }
                if attenuated {
                    changed_start.get_or_insert(position);
                } else if let Some(start) = changed_start.take() {
                    changed.push(TimeRange {
                        start: source
                            .metadata
                            .grid
                            .time_at_frame(start as u64, crate::Rounding::Floor)?,
                        end: source
                            .metadata
                            .grid
                            .time_at_frame(position as u64, crate::Rounding::Ceil)?,
                    });
                }
                position += 1;
            }
            Ok(())
        })?;
        if decoded.metadata != source.metadata
            || position as u64 != source.metadata.grid.frame_count
        {
            return Err(RenderBlocker("Decoded source geometry changed.".into()));
        }
        if let Some(start) = changed_start {
            changed.push(TimeRange {
                start: source
                    .metadata
                    .grid
                    .time_at_frame(start as u64, crate::Rounding::Floor)?,
                end: source.metadata.grid.duration()?,
            });
        }
        if measured.bleed_samples != 0 {
            measured.bleed_input_rms = (track_input_square / measured.bleed_samples as f64).sqrt();
            measured.bleed_output_rms =
                (track_output_square / measured.bleed_samples as f64).sqrt();
        }
        result.preservation_samples += measured.preservation_samples;
        result.attenuated_preservation_samples += measured.attenuated_preservation_samples;
        result.bleed_samples += measured.bleed_samples;
        input_square += track_input_square;
        output_square += track_output_square;
        result.tracks.push(measured);
    }
    changed.sort_unstable_by_key(|r| r.start);
    let mut union = Vec::<TimeRange>::new();
    for range in changed {
        if let Some(previous) = union.last_mut()
            && previous.end >= range.start
        {
            previous.end = previous.end.max(range.end);
        } else {
            union.push(range);
        }
    }
    let mut events = std::collections::BTreeMap::<u64, Vec<(TrackId, bool)>>::new();
    for label in manifest
        .labels
        .iter()
        .filter(|label| label.kind == EvaluationKind::Preserve)
    {
        events
            .entry(label.range.start)
            .or_default()
            .push((label.track, true));
        events
            .entry(label.range.end)
            .or_default()
            .push((label.track, false));
    }
    let events: Vec<_> = events.into_iter().collect();
    let mut active = std::collections::BTreeMap::<TrackId, usize>::new();
    for window in events.windows(2) {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        for &(track, starts) in &window[0].1 {
            let count = active.entry(track).or_default();
            if starts {
                *count += 1;
            } else {
                *count -= 1;
            }
        }
        if active.values().filter(|&&count| count != 0).count() < 2 {
            continue;
        }
        let (start, end) = (window[0].0, window[1].0);
        result.overlap_ticks += end - start;
        let first = union.partition_point(|range| range.end <= start);
        for range in union[first..].iter().take_while(|range| range.start < end) {
            result.attenuated_overlap_ticks += range.end.min(end) - range.start.max(start);
        }
    }
    if result.bleed_samples != 0 {
        result.bleed_input_rms = (input_square / result.bleed_samples as f64).sqrt();
        result.bleed_output_rms = (output_square / result.bleed_samples as f64).sqrt();
    }
    manifest.validate(project, cancelled)?;
    Ok(result)
}
