use crate::{
    AutomixParameters, CalibrationExample, CalibrationKind, OwnershipSettings, Protection,
    SourceMetadata, SourceStamp, TICKS_PER_SECOND, TimePosition, TimeRange,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const PROJECT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStep {
    #[default]
    Import,
    Validate,
    Process,
    Review,
    Export,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RenderState {
    None,
    Current { identity: PathBuf },
    Stale,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ReviewState {
    Unreviewed,
    Reviewed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TrackId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SpeakerId(pub u64);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceIdentity {
    pub track_id: TrackId,
    pub speaker_id: SpeakerId,
    pub path: PathBuf,
    pub fingerprint: String,
    pub metadata: SourceMetadata,
    #[serde(skip, default)]
    pub available: bool,
    #[serde(skip)]
    loaded_stamp: Option<SourceStamp>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBinding {
    pub track: TrackId,
    pub fingerprint: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceLabel {
    pub id: u64,
    pub range: TimeRange,
    pub active_speakers: Vec<SpeakerId>,
    pub sources: Vec<SourceBinding>,
    // Retained detector room-floor adapter. New room labels apply to all tracks.
    #[serde(default)]
    pub reference_track: Option<TrackId>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvelopePoint {
    pub time: TimePosition,
    pub gain: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManualGainEdit {
    pub id: u64,
    pub source: SourceBinding,
    pub points: Vec<EnvelopePoint>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundProtection {
    pub track: Option<TrackId>,
    pub range: TimeRange,
    pub sources: Vec<SourceBinding>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisRecipe {
    pub decoder: String,
    pub resampler: String,
    pub feature_recipe: String,
    pub model_sha256: Option<String>,
    pub runtime_sha256: Option<String>,
    pub mix_recipe: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultState {
    None,
    Current { identity: String },
    Stale { identity: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    schema_version: u32,
    ticks_per_second: u64,
    #[serde(skip)]
    path: PathBuf,
    pub sources: Vec<SourceIdentity>,
    references: Vec<ReferenceLabel>,
    manual_edits: Vec<ManualGainEdit>,
    bound_protections: Vec<BoundProtection>,
    pending_rebind: Vec<TrackId>,
    parameters: AutomixParameters,
    ownership_enabled: bool,
    #[serde(skip)]
    ownership: OwnershipSettings,
    #[serde(skip)]
    protections: Vec<Protection>,
    #[serde(skip)]
    runtime_order: Vec<TrackId>,
    #[serde(skip)]
    calibration_notice: Option<String>,
    review_states: BTreeMap<String, ReviewState>,
    analysis_recipe: Option<AnalysisRecipe>,
    result: ResultState,
    render: RenderState,
    last_render: Option<PathBuf>,
    render_sources: Vec<SourceBinding>,
    step: WorkflowStep,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn io_error(error: crate::RenderBlocker) -> io::Error {
    invalid(error.to_string())
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

impl Project {
    pub fn create(path: impl Into<PathBuf>, paths: Vec<PathBuf>) -> io::Result<Self> {
        let sources = paths
            .into_iter()
            .enumerate()
            .map(|(index, path)| {
                SourceIdentity::read(path, TrackId(index as u64 + 1), SpeakerId(index as u64 + 1))
            })
            .collect::<io::Result<Vec<_>>>()?;
        let mut project = Self {
            schema_version: PROJECT_SCHEMA_VERSION,
            ticks_per_second: TICKS_PER_SECOND,
            path: path.into(),
            sources,
            references: vec![],
            manual_edits: vec![],
            bound_protections: vec![],
            pending_rebind: vec![],
            parameters: AutomixParameters::default(),
            ownership_enabled: true,
            ownership: OwnershipSettings::default(),
            protections: vec![],
            runtime_order: vec![],
            calibration_notice: None,
            review_states: BTreeMap::new(),
            analysis_recipe: None,
            result: ResultState::None,
            render: RenderState::None,
            last_render: None,
            render_sources: vec![],
            step: WorkflowStep::Validate,
        };
        project.rebuild_runtime()?;
        project.save()?;
        Ok(project)
    }

    pub fn open(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let mut project: Self = serde_json::from_slice(&fs::read(&path)?).map_err(invalid_json)?;
        project.path = path;
        project.validate_contract()?;
        for source in &mut project.sources {
            source.loaded_stamp = SourceStamp::read(&source.path).ok();
            source.available = source.matches_disk();
        }
        if project.sources.iter().any(|source| !source.available) {
            project.step = WorkflowStep::Import;
            project.invalidate("Source recordings changed or are missing. Human decisions remain recorded; relink and review before they apply.");
        } else if matches!(&project.render, RenderState::Current { identity } if !identity.is_file())
        {
            project.step = WorkflowStep::Process;
            project.render = RenderState::Stale;
        }
        project.rebuild_runtime()?;
        Ok(project)
    }

    pub fn save(&self) -> io::Result<()> {
        self.validate_contract()?;
        if !self.runtime_order.is_empty()
            && self.runtime_order != self.sources.iter().map(|s| s.track_id).collect::<Vec<_>>()
        {
            return Err(invalid(
                "Use reorder_tracks to change track order without transferring decisions.",
            ));
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(invalid_json)?;
        atomic_save(&self.path, &bytes)
    }

    fn transaction(&mut self, change: impl FnOnce(&mut Self) -> io::Result<()>) -> io::Result<()> {
        let mut next = self.clone();
        change(&mut next)?;
        next.validate_contract()?;
        next.rebuild_runtime()?;
        next.save()?;
        *self = next;
        Ok(())
    }

    fn source(&self, id: TrackId) -> io::Result<&SourceIdentity> {
        self.sources
            .iter()
            .find(|s| s.track_id == id)
            .ok_or_else(|| invalid("Unknown track ID."))
    }
    fn bindings(&self) -> Vec<SourceBinding> {
        self.sources
            .iter()
            .map(|s| SourceBinding {
                track: s.track_id,
                fingerprint: s.fingerprint.clone(),
            })
            .collect()
    }
    fn binding_current(&self, binding: &SourceBinding) -> bool {
        self.source(binding.track).is_ok_and(|s| {
            s.available
                && s.fingerprint == binding.fingerprint
                && !self.pending_rebind.contains(&s.track_id)
                && s.stamp_is_current()
        })
    }
    fn validate_bindings(&self, bindings: &[SourceBinding], range: TimeRange) -> io::Result<()> {
        TimeRange::new(range.start, range.end).map_err(io_error)?;
        if bindings.is_empty()
            || bindings
                .iter()
                .map(|b| b.track)
                .collect::<HashSet<_>>()
                .len()
                != bindings.len()
        {
            return Err(invalid("Source bindings must be nonempty and unique."));
        }
        for binding in bindings {
            let source = self.source(binding.track)?;
            if !valid_hash(&binding.fingerprint) {
                return Err(invalid("Malformed source fingerprint."));
            }
            // Changed-source records are kept dormant until review; their old bounds can
            // exceed replacement audio. Explicit rebinding must validate the new bounds.
            if source.fingerprint == binding.fingerprint
                && range.end > source.metadata.grid.duration().map_err(io_error)?
            {
                return Err(invalid("Human decision is outside its bound source."));
            }
        }
        Ok(())
    }

    fn validate_contract(&self) -> io::Result<()> {
        if self.schema_version != PROJECT_SCHEMA_VERSION
            || self.ticks_per_second != TICKS_PER_SECOND
        {
            return Err(invalid(
                "Unsupported project schema or timebase. Start a fresh project.",
            ));
        }
        if self.sources.is_empty() || self.sources.len() > 8 {
            return Err(invalid("A session requires one to eight tracks."));
        }
        let mut tracks = HashSet::new();
        let mut speakers = HashSet::new();
        for source in &self.sources {
            if source.track_id.0 == 0
                || source.speaker_id.0 == 0
                || !tracks.insert(source.track_id)
                || !speakers.insert(source.speaker_id)
            {
                return Err(invalid(
                    "Track and speaker IDs must be nonzero and one-to-one.",
                ));
            }
            if !valid_hash(&source.fingerprint)
                || source.metadata.channels != 1
                || source.metadata.grid.frame_count == 0
            {
                return Err(invalid(
                    "Source identity must describe nonempty mono audio.",
                ));
            }
            source.metadata.grid.duration().map_err(io_error)?;
        }
        if self.pending_rebind.iter().any(|id| !tracks.contains(id))
            || self.pending_rebind.iter().collect::<HashSet<_>>().len() != self.pending_rebind.len()
        {
            return Err(invalid("Invalid pending source review IDs."));
        }
        if self
            .render_sources
            .iter()
            .any(|binding| !tracks.contains(&binding.track) || !valid_hash(&binding.fingerprint))
            || self
                .render_sources
                .iter()
                .map(|binding| binding.track)
                .collect::<HashSet<_>>()
                .len()
                != self.render_sources.len()
            || (matches!(self.render, RenderState::Current { .. })
                && self.render_sources.len() != self.sources.len())
        {
            return Err(invalid("Render identity has invalid source bindings."));
        }
        crate::prepared_analysis::validate_parameters(self.parameters).map_err(io_error)?;
        let mut labels = HashSet::new();
        for label in &self.references {
            if label.id == 0 || !labels.insert(label.id) {
                return Err(invalid("Reference IDs must be nonzero and unique."));
            }
            self.validate_bindings(&label.sources, label.range)?;
            if label.range.end - label.range.start < TICKS_PER_SECOND / 4 {
                return Err(invalid("References require at least 0.25 seconds."));
            }
            if label
                .active_speakers
                .iter()
                .any(|id| !speakers.contains(id))
                || label.active_speakers.iter().collect::<HashSet<_>>().len()
                    != label.active_speakers.len()
            {
                return Err(invalid("Invalid active speaker set."));
            }
            if label
                .reference_track
                .is_some_and(|id| !tracks.contains(&id))
            {
                return Err(invalid("Invalid reference track."));
            }
            if label.active_speakers.iter().any(|id| {
                !self.sources.iter().any(|source| {
                    source.speaker_id == *id
                        && label
                            .sources
                            .iter()
                            .any(|binding| binding.track == source.track_id)
                })
            }) {
                return Err(invalid(
                    "Active speakers must be bound to their dedicated source tracks.",
                ));
            }
        }
        for (index, label) in self.references.iter().enumerate() {
            for other in &self.references[index + 1..] {
                if label.range.start < other.range.end && other.range.start < label.range.end {
                    let mut left = label.active_speakers.clone();
                    left.sort_unstable();
                    let mut right = other.active_speakers.clone();
                    right.sort_unstable();
                    if left != right {
                        return Err(invalid(
                            "Contradictory overlapping references require explicit replacement.",
                        ));
                    }
                }
            }
        }
        for protection in &self.bound_protections {
            self.validate_bindings(&protection.sources, protection.range)?;
            if protection
                .track
                .is_some_and(|id| !protection.sources.iter().any(|b| b.track == id))
            {
                return Err(invalid("Protection binding does not include its track."));
            }
        }
        let mut edits = HashSet::new();
        for edit in &self.manual_edits {
            if edit.id == 0 || !edits.insert(edit.id) || edit.points.len() < 2 {
                return Err(invalid("Manual edit IDs/points are invalid."));
            }
            let range = TimeRange::new(edit.points[0].time, edit.points.last().unwrap().time)
                .map_err(io_error)?;
            self.validate_bindings(std::slice::from_ref(&edit.source), range)?;
            if edit
                .points
                .iter()
                .any(|p| !p.gain.is_finite() || !(0.0..=1.0).contains(&p.gain))
                || edit.points.windows(2).any(|p| p[0].time >= p[1].time)
            {
                return Err(invalid(
                    "Manual envelope points must be ordered, finite and attenuation-only.",
                ));
            }
            for protection in &self.bound_protections {
                if protection.track.is_none_or(|t| t == edit.source.track)
                    && protection.range.start < range.end
                    && range.start < protection.range.end
                {
                    return Err(invalid(
                        "Conflicting manual edit and protection require explicit replacement.",
                    ));
                }
            }
        }
        for (index, edit) in self.manual_edits.iter().enumerate() {
            for other in &self.manual_edits[index + 1..] {
                if edit.source.track == other.source.track
                    && edit.points[0].time < other.points.last().unwrap().time
                    && other.points[0].time < edit.points.last().unwrap().time
                {
                    return Err(invalid(
                        "Overlapping manual edits require explicit replacement.",
                    ));
                }
            }
        }
        if let Some(recipe) = &self.analysis_recipe
            && (recipe.decoder.is_empty()
                || recipe.resampler.is_empty()
                || recipe.feature_recipe.is_empty()
                || recipe.mix_recipe.is_empty()
                || recipe.model_sha256.as_ref().is_some_and(|h| !valid_hash(h))
                || recipe
                    .runtime_sha256
                    .as_ref()
                    .is_some_and(|h| !valid_hash(h)))
        {
            return Err(invalid("Invalid analysis recipe provenance."));
        }
        if matches!(&self.result, ResultState::Current{identity}|ResultState::Stale{identity} if identity.is_empty())
        {
            return Err(invalid("Result identity cannot be empty."));
        }
        Ok(())
    }

    fn rebuild_runtime(&mut self) -> io::Result<()> {
        self.runtime_order = self.sources.iter().map(|s| s.track_id).collect();
        self.protections = self
            .bound_protections
            .iter()
            .filter(|p| p.sources.iter().all(|b| self.binding_current(b)))
            .flat_map(|p| {
                let tracks: Vec<_> = if p.track.is_none() && p.sources.len() == self.sources.len() {
                    vec![None]
                } else {
                    p.sources
                        .iter()
                        .filter(|binding| p.track.is_none_or(|id| id == binding.track))
                        .map(|binding| {
                            self.sources
                                .iter()
                                .position(|source| source.track_id == binding.track)
                        })
                        .collect()
                };
                tracks.into_iter().map(|track| Protection {
                    track,
                    start: p.range.start,
                    end: p.range.end,
                })
            })
            .collect();
        let ready = self.sources.iter().all(|s| s.available) && self.pending_rebind.is_empty();
        let mut examples = vec![];
        if ready {
            for label in &self.references {
                if label.sources.len() != self.sources.len()
                    || !label.sources.iter().all(|b| self.binding_current(b))
                {
                    continue;
                }
                if label.active_speakers.is_empty() {
                    for (track, source) in self.sources.iter().enumerate() {
                        if label.reference_track.is_none_or(|id| id == source.track_id) {
                            examples.push(CalibrationExample {
                                kind: CalibrationKind::RoomTone,
                                track,
                                start: label.range.start,
                                end: label.range.end,
                            });
                        }
                    }
                } else if label.active_speakers.len() == 1 {
                    let track = self
                        .sources
                        .iter()
                        .position(|s| s.speaker_id == label.active_speakers[0])
                        .ok_or_else(|| invalid("Unknown speaker."))?;
                    examples.push(CalibrationExample {
                        kind: CalibrationKind::SoloSpeech,
                        track,
                        start: label.range.start,
                        end: label.range.end,
                    });
                }
            }
        }
        self.ownership = OwnershipSettings {
            enabled: self.ownership_enabled,
            examples,
            sources: self
                .sources
                .iter()
                .filter_map(|s| s.loaded_stamp.clone())
                .collect(),
        };
        Ok(())
    }

    fn invalidate(&mut self, reason: &str) {
        self.calibration_notice = Some(reason.into());
        self.render = RenderState::Stale;
        if let ResultState::Current { identity } = &self.result {
            self.result = ResultState::Stale {
                identity: identity.clone(),
            };
        }
    }

    pub fn source_paths(&self) -> Vec<PathBuf> {
        self.sources.iter().map(|s| s.path.clone()).collect()
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn step(&self) -> WorkflowStep {
        self.step
    }
    pub fn protections(&self) -> &[Protection] {
        if self.sources.iter().all(SourceIdentity::stamp_is_current) {
            &self.protections
        } else {
            &[]
        }
    }
    pub fn parameters(&self) -> AutomixParameters {
        self.parameters
    }
    pub fn ownership_settings(&self) -> &OwnershipSettings {
        &self.ownership
    }
    pub fn calibration_notice(&self) -> Option<&str> {
        self.calibration_notice.as_deref()
    }
    pub fn reference_labels(&self) -> &[ReferenceLabel] {
        &self.references
    }
    pub fn manual_gain_edits(&self) -> &[ManualGainEdit] {
        &self.manual_edits
    }
    pub fn recorded_protections(&self) -> &[BoundProtection] {
        &self.bound_protections
    }
    pub fn pending_source_reviews(&self) -> &[TrackId] {
        &self.pending_rebind
    }
    pub fn result(&self) -> &ResultState {
        &self.result
    }
    pub fn analysis_recipe(&self) -> Option<&AnalysisRecipe> {
        self.analysis_recipe.as_ref()
    }
    pub fn review_states(&self) -> &BTreeMap<String, ReviewState> {
        &self.review_states
    }
    pub fn render(&self) -> &RenderState {
        static STALE: RenderState = RenderState::Stale;
        if matches!(self.render, RenderState::Current { .. }) && !self.render_bindings_current() {
            &STALE
        } else {
            &self.render
        }
    }
    pub fn review_identity(&self) -> Option<&Path> {
        self.render_bindings_current()
            .then_some(self.last_render.as_deref())
            .flatten()
    }

    fn render_bindings_current(&self) -> bool {
        self.render_sources
            .iter()
            .map(|b| b.track)
            .eq(self.sources.iter().map(|s| s.track_id))
            && self.render_sources.iter().all(|b| self.binding_current(b))
    }

    /// Strong validation at source loading, Analyze and final delivery boundaries;
    /// interactive edits use stamps instead of hashing a recording on each drag.
    pub fn validate_sources(&self) -> io::Result<()> {
        self.validate_sources_cancellable(None)
    }

    pub fn validate_sources_cancellable(
        &self,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> io::Result<()> {
        if !self.pending_rebind.is_empty() {
            return Err(invalid(
                "Review and rebind changed sources before analysis or export.",
            ));
        }
        for source in &self.sources {
            if !source.available
                || fingerprint_cancellable(&source.path, cancelled)? != source.fingerprint
                || SourceMetadata::read(&source.path).map_err(io_error)? != source.metadata
            {
                return Err(invalid(
                    "Source audio changed; relink and review its bound decisions.",
                ));
            }
        }
        Ok(())
    }

    pub fn set_ownership_settings(&mut self, settings: OwnershipSettings) -> io::Result<()> {
        if !settings.examples.is_empty() {
            let stamps = self
                .sources
                .iter()
                .map(|s| SourceStamp::read(&s.path).map_err(io_error))
                .collect::<io::Result<Vec<_>>>()?;
            if settings.sources != stamps
                || self
                    .sources
                    .iter()
                    .zip(&stamps)
                    .any(|(s, stamp)| !s.available || s.loaded_stamp.as_ref() != Some(stamp))
                || !self.pending_rebind.is_empty()
            {
                return Err(invalid(
                    "Sources changed; relink and review before adding references.",
                ));
            }
        }
        let mut references = vec![];
        for (index, example) in settings.examples.iter().enumerate() {
            let source = self
                .sources
                .get(example.track)
                .ok_or_else(|| invalid("Unknown calibration track."))?;
            references.push(ReferenceLabel {
                id: index as u64 + 1,
                range: TimeRange::new(example.start, example.end).map_err(io_error)?,
                active_speakers: if example.kind == CalibrationKind::SoloSpeech {
                    vec![source.speaker_id]
                } else {
                    vec![]
                },
                sources: self.bindings(),
                reference_track: Some(source.track_id),
            });
        }
        self.transaction(|next| {
            next.references = references;
            next.ownership_enabled = settings.enabled;
            next.invalidate("Reference labels changed; Analyze is required.");
            Ok(())
        })
    }
    pub fn replace_reference_labels(&mut self, labels: Vec<ReferenceLabel>) -> io::Result<()> {
        if labels
            .iter()
            .any(|l| !l.sources.iter().all(|b| self.binding_current(b)))
        {
            return Err(invalid("New labels require current source bindings."));
        }
        self.transaction(|next| {
            next.references = labels;
            next.invalidate("Reference labels changed; Analyze is required.");
            Ok(())
        })
    }
    pub fn replace_manual_gain_edits(&mut self, edits: Vec<ManualGainEdit>) -> io::Result<()> {
        if edits.iter().any(|e| !self.binding_current(&e.source)) {
            return Err(invalid("New gain edits require current source bindings."));
        }
        self.transaction(|next| {
            next.manual_edits = edits;
            next.render = RenderState::Stale;
            Ok(())
        })
    }
    pub fn replace_protections(&mut self, protections: Vec<Protection>) -> io::Result<()> {
        let mut bound = vec![];
        for protection in protections {
            let track = protection
                .track
                .map(|index| {
                    self.sources
                        .get(index)
                        .map(|s| s.track_id)
                        .ok_or_else(|| invalid("Unknown protection track."))
                })
                .transpose()?;
            let bindings: Vec<_> = self
                .bindings()
                .into_iter()
                .filter(|b| track.is_none_or(|id| b.track == id))
                .collect();
            if bindings.iter().any(|b| !self.binding_current(b)) {
                return Err(invalid("New protections require current source bindings."));
            }
            let end = if protection.end == u64::MAX {
                bindings
                    .iter()
                    .map(|b| {
                        self.source(b.track)?
                            .metadata
                            .grid
                            .duration()
                            .map_err(io_error)
                    })
                    .collect::<io::Result<Vec<_>>>()?
                    .into_iter()
                    .min()
                    .unwrap_or(0)
            } else {
                protection.end
            };
            bound.push(BoundProtection {
                track,
                range: TimeRange::new(protection.start, end).map_err(io_error)?,
                sources: bindings,
            });
        }
        self.transaction(|next| {
            next.bound_protections = bound;
            next.render = RenderState::Stale;
            Ok(())
        })
    }
    fn add_protections(&mut self, added: Vec<Protection>) -> io::Result<()> {
        if self.bound_protections.len() != self.protections.len() {
            return Err(invalid(
                "Review dormant protections before replacing manual constraints.",
            ));
        }
        let mut all = self.protections.clone();
        all.extend(added);
        self.replace_protections(all)
    }
    pub fn protect_track(&mut self, track: usize) -> io::Result<()> {
        self.add_protections(vec![Protection {
            track: Some(track),
            start: 0,
            end: u64::MAX,
        }])
    }
    pub fn protect_range(&mut self, start: TimePosition, end: TimePosition) -> io::Result<()> {
        self.add_protections(vec![Protection {
            track: None,
            start,
            end,
        }])
    }
    pub fn protect_region(
        &mut self,
        tracks: &[usize],
        start: TimePosition,
        end: TimePosition,
    ) -> io::Result<()> {
        self.add_protections(
            tracks
                .iter()
                .map(|&track| Protection {
                    track: Some(track),
                    start,
                    end,
                })
                .collect(),
        )
    }
    pub fn mark_validated(&mut self) -> io::Result<()> {
        self.transaction(|next| {
            next.step = WorkflowStep::Process;
            Ok(())
        })
    }
    pub fn set_parameters(&mut self, parameters: AutomixParameters) -> io::Result<()> {
        self.transaction(|next| {
            next.parameters = parameters;
            next.render = RenderState::Stale;
            Ok(())
        })
    }
    pub fn mark_render_stale(&mut self) -> io::Result<()> {
        self.transaction(|next| {
            next.render = RenderState::Stale;
            Ok(())
        })
    }
    pub fn set_review_state(
        &mut self,
        warning: impl Into<String>,
        state: ReviewState,
    ) -> io::Result<()> {
        self.transaction(|next| {
            next.review_states.insert(warning.into(), state);
            Ok(())
        })
    }
    pub fn mark_render_current(&mut self, identity: impl Into<PathBuf>) -> io::Result<()> {
        self.transaction(|next| {
            if !next.pending_rebind.is_empty()
                || next
                    .sources
                    .iter()
                    .any(|s| !s.available || !s.stamp_is_current())
            {
                return Err(invalid("Sources require review before accepting a render."));
            }
            let identity = identity.into();
            next.last_render = Some(identity.clone());
            next.render = RenderState::Current { identity };
            next.render_sources = next.bindings();
            next.step = WorkflowStep::Review;
            Ok(())
        })
    }
    pub fn set_result(
        &mut self,
        result: ResultState,
        recipe: Option<AnalysisRecipe>,
    ) -> io::Result<()> {
        self.transaction(|next| {
            next.result = result;
            next.analysis_recipe = recipe;
            Ok(())
        })
    }

    pub fn relink(&mut self, index: usize, path: PathBuf) -> io::Result<()> {
        let old = self
            .sources
            .get(index)
            .ok_or_else(|| invalid("Unknown source track."))?;
        let replacement = SourceIdentity::read(path, old.track_id, old.speaker_id)?;
        let changed =
            replacement.fingerprint != old.fingerprint || replacement.metadata != old.metadata;
        self.transaction(|next|{
            let id=replacement.track_id;next.sources[index]=replacement;
            if changed && !next.pending_rebind.contains(&id){next.pending_rebind.push(id);}
            if changed {next.invalidate("Replacement audio differs; recorded decisions are dormant until explicitly reviewed and rebound.");}
            next.step=WorkflowStep::Validate;Ok(())
        })
    }
    pub fn review_and_rebind_source(&mut self, track: TrackId) -> io::Result<()> {
        let source = self.source(track)?;
        if !source.available || !source.matches_disk() {
            return Err(invalid("Relink current audio before source review."));
        }
        let fingerprint = source.fingerprint.clone();
        self.transaction(|next| {
            for label in &mut next.references {
                for binding in &mut label.sources {
                    if binding.track == track {
                        binding.fingerprint = fingerprint.clone();
                    }
                }
            }
            for protection in &mut next.bound_protections {
                for binding in &mut protection.sources {
                    if binding.track == track {
                        binding.fingerprint = fingerprint.clone();
                    }
                }
            }
            for edit in &mut next.manual_edits {
                if edit.source.track == track {
                    edit.source.fingerprint = fingerprint.clone();
                }
            }
            next.pending_rebind.retain(|&id| id != track);
            next.invalidate("Source decisions rebound after explicit review; Analyze is required.");
            Ok(())
        })
    }
    pub fn reorder_tracks(&mut self, order: &[TrackId]) -> io::Result<()> {
        if order.len() != self.sources.len()
            || order.iter().collect::<HashSet<_>>().len() != order.len()
        {
            return Err(invalid("Track order must include each track once."));
        }
        let sources = order
            .iter()
            .map(|&id| self.source(id).cloned())
            .collect::<io::Result<Vec<_>>>()?;
        self.transaction(|next| {
            next.sources = sources;
            next.invalidate("Track order changed; source-bound results require regeneration.");
            Ok(())
        })
    }
    pub fn add_sources(&mut self, paths: Vec<PathBuf>) -> io::Result<()> {
        let first = self
            .sources
            .iter()
            .map(|s| s.track_id.0.max(s.speaker_id.0))
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| invalid("Track ID overflow."))?;
        let sources = paths
            .into_iter()
            .enumerate()
            .map(|(index, path)| {
                let id = first
                    .checked_add(index as u64)
                    .ok_or_else(|| invalid("Track ID overflow."))?;
                SourceIdentity::read(path, TrackId(id), SpeakerId(id))
            })
            .collect::<io::Result<Vec<_>>>()?;
        self.transaction(|next| {
            next.sources.extend(sources);
            next.step = WorkflowStep::Process;
            next.invalidate("Session tracks changed; Analyze is required.");
            Ok(())
        })
    }
}

impl SourceIdentity {
    fn stamp_is_current(&self) -> bool {
        self.loaded_stamp.as_ref().is_some_and(|stamp| {
            SourceStamp::read(&self.path).is_ok_and(|current| current == *stamp)
        })
    }
    fn read(path: PathBuf, track_id: TrackId, speaker_id: SpeakerId) -> io::Result<Self> {
        let stamp = SourceStamp::read(&path).map_err(io_error)?;
        let metadata = SourceMetadata::read(&path).map_err(io_error)?;
        let fingerprint = fingerprint(&path)?;
        if SourceStamp::read(&path).map_err(io_error)? != stamp {
            return Err(invalid("Source changed while recording its identity."));
        }
        Ok(Self {
            track_id,
            speaker_id,
            path,
            fingerprint,
            metadata,
            available: true,
            loaded_stamp: Some(stamp),
        })
    }
    fn matches_disk(&self) -> bool {
        fingerprint(&self.path).is_ok_and(|h| h == self.fingerprint)
            && SourceMetadata::read(&self.path).is_ok_and(|m| m == self.metadata)
    }
}
fn invalid_json(error: serde_json::Error) -> io::Error {
    invalid(error.to_string())
}
fn fingerprint(path: &Path) -> io::Result<String> {
    fingerprint_cancellable(path, None)
}
fn fingerprint_cancellable(
    path: &Path,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> io::Result<String> {
    let mut hash = Sha256::new();
    let mut file = fs::File::open(path)?;
    let mut buffer = [0; 65_536];
    loop {
        if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Source validation cancelled.",
            ));
        }
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn atomic_save(path: &Path, bytes: &[u8]) -> io::Result<()> {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| invalid("Project requires a file name."))?;
    let staging = parent.join(format!(
        ".{}.{}-{}.tmp",
        name.to_string_lossy(),
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&staging, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(staging);
    }
    result
}
