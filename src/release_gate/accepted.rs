//! Frozen held-out truth is read by evaluation, never by analysis or training.
use crate::{Project, RenderBlocker, SourceTimebase, TimeRange, TrackId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::atomic::AtomicBool};

/// Score the same completed analysis used by the editor without installing a project result.
/// Held-out truth is consumed only after analysis has completed.
#[derive(Debug, Serialize)]
pub struct EditorGainEvidence {
    pub analysis_identity: String,
    pub automatic: AppliedGainEvidence,
    pub effective: AppliedGainEvidence,
    pub ownership: Vec<HeldOutOwnershipEvidence>,
}
impl EditorGainEvidence {
    pub fn release_status(&self) -> &'static str {
        if self.automatic.attenuated_preservation_samples > 0
            || self.automatic.attenuated_overlap_ticks > 0
        {
            "failed"
        } else {
            "incomplete"
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HeldOutOwnershipEvidence {
    pub track: TrackId,
    pub kind: EvaluationKind,
    pub samples: u64,
    pub probability_samples: u64,
    pub mean_probability: Option<f64>,
    pub owned_samples: u64,
    pub absent_samples: u64,
    pub uncertain_unity_samples: u64,
}

fn measure_ownership(
    project: &Project,
    manifest: &FrozenEvaluationManifest,
    analysis: &crate::EditorAnalysisResult,
    cancel: &AtomicBool,
) -> Result<Vec<HeldOutOwnershipEvidence>, RenderBlocker> {
    let mut result = Vec::new();
    for source in &project.sources {
        let index = analysis
            .temporal
            .tracks
            .iter()
            .position(|track| *track == source.track_id)
            .ok_or_else(|| {
                RenderBlocker("Evaluation analysis is missing a source track.".into())
            })?;
        let intervals = native_intervals(&manifest.labels, source.track_id, source.metadata.grid)?;
        for kind in [EvaluationKind::Preserve, EvaluationKind::Bleed] {
            let mut row = HeldOutOwnershipEvidence {
                track: source.track_id,
                kind,
                samples: 0,
                probability_samples: 0,
                mean_probability: None,
                owned_samples: 0,
                absent_samples: 0,
                uncertain_unity_samples: 0,
            };
            let mut weighted = 0.0_f64;
            for (range, _) in intervals.iter().filter(|(_, k)| *k == kind) {
                let mut position = range.start as u64;
                while position < range.end as u64 {
                    crate::prepared_analysis::check_cancelled(Some(cancel))?;
                    let time = source
                        .metadata
                        .grid
                        .time_at_frame(position, crate::Rounding::Floor)?;
                    let boundary = ((time / crate::OWNERSHIP_FRAME_TICKS + 1)
                        * crate::OWNERSHIP_FRAME_TICKS)
                        .min(source.metadata.grid.duration()?);
                    let end = source
                        .metadata
                        .grid
                        .frame_at_time(boundary, crate::Rounding::Ceil)?
                        .max(position + 1)
                        .min(range.end as u64);
                    let count = end - position;
                    row.samples += count;
                    if time < analysis.temporal.duration {
                        let probability = analysis
                            .ownership
                            .estimates_at(time)?
                            .into_iter()
                            .find(|e| e.track == source.track_id)
                            .and_then(|e| e.probability);
                        if let Some(p) = probability {
                            row.probability_samples += count;
                            weighted += f64::from(p) * count as f64;
                        }
                        match analysis.temporal.states_at(time)?[index] {
                            crate::TemporalTrackState::Owned => row.owned_samples += count,
                            crate::TemporalTrackState::Absent => row.absent_samples += count,
                            crate::TemporalTrackState::UncertainUnity => {
                                row.uncertain_unity_samples += count
                            }
                        }
                    } else {
                        row.uncertain_unity_samples += count;
                    }
                    position = end;
                }
            }
            if row.probability_samples > 0 {
                row.mean_probability = Some(weighted / row.probability_samples as f64);
            }
            if row.samples > 0 {
                result.push(row);
            }
        }
    }
    Ok(result)
}

/// Existing headless entry points share the editor's required-model worker.
/// This command produces evidence; it cannot grant listening or release approval.
pub fn evaluate_project_command(
    args: &[std::ffi::OsString],
) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() != 5 || args[0] != "--project" {
        return Err("--project <project.cleanslate> <frozen-manifest.json> <new-output.json> <cache-directory>".into());
    }
    let project = Project::open(std::path::PathBuf::from(&args[1]))?;
    let saved_project = std::fs::read(project.path())?;
    if std::path::Path::new(&args[3]).exists() {
        return Err("Evaluation report already exists; choose a new output path.".into());
    }
    let cancel = AtomicBool::new(false);
    let manifest = FrozenEvaluationManifest::read(
        &project,
        &std::fs::read_to_string(&args[2])?,
        Some(&cancel),
    )?;
    let helper = std::env::current_exe()?.with_file_name(if cfg!(windows) {
        "model-check.exe"
    } else {
        "model-check"
    });
    let mut verification = std::process::Command::new(helper);
    verification
        .arg("--verify-only")
        .arg("--runtime")
        .arg(crate::model::runtime_directory().map_err(RenderBlocker)?);
    let runtime =
        crate::model::verify_native_runtime(verification, &cancel).map_err(RenderBlocker)?;
    let mut worker = crate::EditorAnalysisWorker::default();
    let analysis = worker.analyze(
        &project,
        std::path::Path::new(&args[4]),
        Some(&runtime),
        &std::sync::atomic::AtomicU8::new(0),
        &cancel,
    )?;
    let score = measure_editor_result(&project, &manifest, &analysis, &cancel)?;
    manifest.validate(&project, Some(&cancel))?;
    crate::verify_installed_editor_runtime(&runtime, &cancel)?;
    if std::fs::read(project.path())? != saved_project {
        return Err("Saved project changed during evaluation; no report published.".into());
    }
    let report = serde_json::json!({
        "version":1,"manifest_sha256":manifest.digest(),"claim":manifest.claim(),
        "analysis_recipe":analysis.recipe,"runtime":analysis.runtime,"gain":score,
        "parameters":project.parameters(),"manual_constraints":project.gain_constraints(),
        "release_status":score.release_status(),"human_listening_performed":false,
        "pending":["frozen baseline comparison","independent real truth","owner listening and timing approval"]
    });
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?
        .write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
    Ok(())
}

pub fn measure_editor_result(
    project: &Project,
    manifest: &FrozenEvaluationManifest,
    analysis: &crate::EditorAnalysisResult,
    cancel: &AtomicBool,
) -> Result<EditorGainEvidence, RenderBlocker> {
    manifest.validate(project, Some(cancel))?;
    if !analysis.snapshot.matches(project) {
        return Err(RenderBlocker(
            "Analysis no longer matches frozen evaluation sources and references.".into(),
        ));
    }
    let effective = crate::EffectiveEnvelopeCache::default().update(
        project,
        &analysis.temporal,
        crate::editor_workflow::project_fade_settings(project)?,
        Some(cancel),
    )?;
    if !effective.analysis_current {
        return Err(RenderBlocker(
            "Evaluation requires current supervised analysis.".into(),
        ));
    }
    let automatic = measure_gain_query(project, manifest, Some(cancel), |track, sample| {
        let time = project.sources[track]
            .metadata
            .grid
            .time_at_frame(sample as u64, crate::Rounding::Floor)?;
        Ok(effective.automatic[track].gain_at_time(time))
    })?;
    let applied = measure_applied_gain(project, manifest, &effective.envelopes, Some(cancel))?;
    Ok(EditorGainEvidence {
        analysis_identity: analysis.snapshot.key.clone(),
        automatic,
        effective: applied,
        ownership: measure_ownership(project, manifest, analysis, cancel)?,
    })
}

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
    measure_gain_query(project, manifest, cancelled, |track, position| {
        Ok(envelopes[track].gain_at(position))
    })
}

fn measure_gain_query(
    project: &Project,
    manifest: &FrozenEvaluationManifest,
    cancelled: Option<&AtomicBool>,
    gain_at: impl Fn(usize, usize) -> Result<f32, RenderBlocker>,
) -> Result<AppliedGainEvidence, RenderBlocker> {
    manifest.validate(project, cancelled)?;
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
                    let gain = gain_at(track, position)?;
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
