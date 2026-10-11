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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvelopePoint {
    pub time: TimePosition,
    pub gain: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManualGainEdit {
    pub id: u64,
    pub source: SourceBinding,
    pub points: Vec<EnvelopePoint>,
    #[serde(default)]
    pub interpolation: ManualInterpolation,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManualInterpolation {
    #[default]
    Linear,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GainConstraints {
    pub manual_edits: Vec<ManualGainEdit>,
    pub protections: Vec<BoundProtection>,
}
#[derive(Clone, Debug)]
pub struct GainEditDraft {
    pub constraints: GainConstraints,
    base: GainConstraints,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GainRegionAction {
    Preserve,
    Attenuate(f32),
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
    #[serde(default)]
    supplied_alignment: bool,
    #[serde(default, rename = "nemotron_enabled", skip_serializing)]
    legacy_native_analysis: bool,
    references: Vec<ReferenceLabel>,
    #[serde(default)]
    next_reference_id: u64,
    #[serde(skip)]
    reference_undo: Vec<Vec<ReferenceLabel>>,
    #[serde(skip)]
    reference_redo: Vec<Vec<ReferenceLabel>>,
    #[serde(skip)]
    gain_undo: Vec<GainConstraints>,
    #[serde(skip)]
    gain_redo: Vec<GainConstraints>,
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
    #[serde(default)]
    render_record: Option<crate::supervised_render::RenderRecord>,
    step: WorkflowStep,
    #[serde(default)]
    export_encoding: crate::ExportEncoding,
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
        Self::create_from_sources(
            path,
            paths.into_iter().map(crate::AudioSource::mono).collect(),
        )
    }

    pub fn create_from_sources(
        path: impl Into<PathBuf>,
        inputs: Vec<crate::AudioSource>,
    ) -> io::Result<Self> {
        let sources = inputs
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
            supplied_alignment: false,
            export_encoding: crate::ExportEncoding::default(),
            legacy_native_analysis: false,
            references: vec![],
            next_reference_id: 1,
            reference_undo: vec![],
            reference_redo: vec![],
            gain_undo: vec![],
            gain_redo: vec![],
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
            render_record: None,
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
        if project.analysis_recipe.as_ref().is_some_and(|recipe| {
            !recipe
                .feature_recipe
                .starts_with(crate::EDITOR_ANALYSIS_RECIPE)
        }) {
            project.invalidate(
                "Analysis recipe changed; Analyze once to enable the current gap control.",
            );
        }
        if project.legacy_native_analysis
            || project
                .analysis_recipe
                .as_ref()
                .is_some_and(|recipe| recipe.model_sha256.is_some())
        {
            project.invalidate("Experimental native analysis was removed. Labels and gain edits are preserved; Analyze again for acoustic results.");
            project.legacy_native_analysis = false;
        }
        for source in &mut project.sources {
            source.loaded_stamp = SourceStamp::read_source(&source.audio_source()).ok();
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
        let mut audio_sources = HashSet::new();
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
            if source.channel.is_some() && !audio_sources.insert((&source.path, source.channel)) {
                return Err(invalid("A source channel may only be imported once."));
            }
            if source.name.as_ref().is_some_and(|name| {
                name.trim().is_empty() || name.len() > 256 || name.chars().any(char::is_control)
            }) {
                return Err(invalid(
                    "Track names require 1 to 256 bytes without control characters.",
                ));
            }
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
                    && protection
                        .sources
                        .iter()
                        .any(|binding| binding.track == edit.source.track)
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
    pub fn audio_sources(&self) -> Vec<crate::AudioSource> {
        self.sources
            .iter()
            .map(SourceIdentity::audio_source)
            .collect()
    }

    pub(crate) fn source_stamps(&self) -> Result<Vec<SourceStamp>, crate::RenderBlocker> {
        self.sources
            .iter()
            .map(|s| SourceStamp::read_source(&s.audio_source()))
            .collect()
    }

    pub fn rename_track(&mut self, index: usize, name: String) -> io::Result<()> {
        let name = name.trim().to_owned();
        if name.is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
            return Err(invalid(
                "Track names require 1 to 256 bytes without control characters.",
            ));
        }
        self.transaction(|next| {
            next.sources
                .get_mut(index)
                .ok_or_else(|| invalid("Unknown source track."))?
                .name = Some(name);
            next.invalidate("Track name changed; Analyze before final export.");
            Ok(())
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn export_encoding(&self) -> crate::ExportEncoding {
        self.export_encoding
    }

    pub fn set_export_encoding(&mut self, encoding: crate::ExportEncoding) -> io::Result<()> {
        self.transaction(|next| {
            next.export_encoding = encoding;
            Ok(())
        })
    }

    pub(crate) fn sound_identity(&self) -> io::Result<String> {
        let data = serde_json::to_vec(&(
            crate::effective_envelope::EFFECTIVE_ENVELOPE_RECIPE,
            &self.sources,
            self.supplied_alignment,
            &self.references,
            self.gain_constraints(),
            self.parameters,
            &self.analysis_recipe,
            &self.result,
        ))
        .map_err(invalid_json)?;
        Ok(format!("{:x}", Sha256::digest(data)))
    }
    pub(crate) fn saved_sound_identity(path: &Path) -> io::Result<String> {
        let project: Self = serde_json::from_slice(&fs::read(path)?).map_err(invalid_json)?;
        project.validate_contract()?;
        project.sound_identity()
    }
    pub(crate) fn delivery_identity(&self) -> io::Result<String> {
        Ok(format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(
                    self.sound_identity()?,
                    &self.render,
                    &self.render_record,
                    self.export_encoding
                ))
                .map_err(invalid_json)?
            )
        ))
    }
    pub(crate) fn saved_delivery_identity(path: &Path) -> io::Result<String> {
        let project: Self = serde_json::from_slice(&fs::read(path)?).map_err(invalid_json)?;
        project.validate_contract()?;
        project.delivery_identity()
    }
    pub(crate) fn render_record(&self) -> Option<&crate::supervised_render::RenderRecord> {
        self.render_record.as_ref()
    }
    pub(crate) fn install_render_record(
        &mut self,
        record: crate::supervised_render::RenderRecord,
    ) -> io::Result<()> {
        self.transaction(|next| {
            if !record.snapshot.matches(next) {
                return Err(invalid("Render snapshot is no longer current."));
            }
            next.render = RenderState::Current {
                identity: record.report.clone(),
            };
            next.last_render = Some(record.report.clone());
            next.render_sources = next.bindings();
            next.render_record = Some(record);
            next.step = WorkflowStep::Review;
            Ok(())
        })
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
    pub fn reference_label_is_current(&self, label: &ReferenceLabel) -> bool {
        label.sources.len() == self.sources.len()
            && label
                .sources
                .iter()
                .all(|binding| self.binding_current(binding))
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

    pub fn supplied_alignment(&self) -> bool {
        self.supplied_alignment
    }

    pub fn set_supplied_alignment(&mut self, declared: bool) -> io::Result<()> {
        self.transaction(|next| {
            if next.supplied_alignment != declared {
                next.supplied_alignment = declared;
                next.invalidate("Supplied alignment declaration changed; verify the synchronized source timeline.");
            }
            Ok(())
        })
    }

    pub fn require_alignment(&self) -> io::Result<()> {
        crate::engine::validate_source_metadata(
            &self
                .sources
                .iter()
                .map(|source| source.metadata)
                .collect::<Vec<_>>(),
        )
        .and_then(|validation| {
            validation.require_alignment(
                self.supplied_alignment
                    || self
                        .sources
                        .iter()
                        .all(|s| s.channel.is_some() && s.path == self.sources[0].path),
            )
        })
        .map_err(io_error)
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
        if self
            .render_record
            .as_ref()
            .is_some_and(|record| !record.snapshot.matches(self))
        {
            return false;
        }
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
                || source.audio_source().fingerprint(cancelled)? != source.fingerprint
                || source
                    .audio_source()
                    .metadata(cancelled)
                    .map_err(io_error)?
                    != source.metadata
            {
                return Err(invalid(
                    "Source audio changed; relink and review its bound decisions.",
                ));
            }
        }
        Ok(())
    }

    pub fn set_ownership_enabled(&mut self, enabled: bool) -> io::Result<()> {
        self.transaction(|next| {
            next.ownership_enabled = enabled;
            next.invalidate("Analysis settings changed; Analyze is required.");
            Ok(())
        })
    }
    pub fn set_ownership_settings(&mut self, settings: OwnershipSettings) -> io::Result<()> {
        if !settings.examples.is_empty() {
            let stamps = self
                .sources
                .iter()
                .map(|s| SourceStamp::read_source(&s.audio_source()).map_err(io_error))
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
            return Err(invalid(
                "Sources changed; new labels require current source bindings after relink and review.",
            ));
        }
        self.transaction(|next| {
            if next.references == labels {
                return Ok(());
            }
            next.reference_undo.push(next.references.clone());
            next.reference_redo.clear();
            next.next_reference_id = next.next_reference_id.max(
                labels
                    .iter()
                    .map(|label| label.id)
                    .max()
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or_else(|| invalid("Reference ID overflow."))?,
            );
            next.references = labels;
            next.invalidate("Reference labels changed; Analyze is required.");
            Ok(())
        })
    }
    pub fn add_reference_label(
        &mut self,
        range: TimeRange,
        mut active_speakers: Vec<SpeakerId>,
        replacing: &[u64],
    ) -> io::Result<u64> {
        if replacing.iter().collect::<HashSet<_>>().len() != replacing.len()
            || replacing
                .iter()
                .any(|id| !self.references.iter().any(|label| label.id == *id))
        {
            return Err(invalid(
                "Reference replacement must name existing unique label IDs.",
            ));
        }
        active_speakers.sort_unstable();
        let id = self.next_reference_id.max(
            self.references
                .iter()
                .map(|label| label.id)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| invalid("Reference ID overflow."))?,
        );
        let mut labels: Vec<_> = self
            .references
            .iter()
            .filter(|label| !replacing.contains(&label.id))
            .cloned()
            .collect();
        labels.push(ReferenceLabel {
            id,
            range,
            active_speakers,
            sources: self.bindings(),
            reference_track: None,
        });
        self.replace_reference_labels(labels)?;
        Ok(id)
    }
    pub fn remove_reference_label(&mut self, id: u64) -> io::Result<()> {
        if !self.references.iter().any(|label| label.id == id) {
            return Err(invalid("Unknown reference label."));
        }
        self.replace_reference_labels(
            self.references
                .iter()
                .filter(|label| label.id != id)
                .cloned()
                .collect(),
        )
    }
    pub fn undo_reference_labels(&mut self) -> io::Result<bool> {
        self.restore_reference_history(false)
    }
    pub fn redo_reference_labels(&mut self) -> io::Result<bool> {
        self.restore_reference_history(true)
    }
    fn restore_reference_history(&mut self, redo: bool) -> io::Result<bool> {
        let history = if redo {
            &self.reference_redo
        } else {
            &self.reference_undo
        };
        let Some(labels) = history.last().cloned() else {
            return Ok(false);
        };
        if labels.iter().any(|label| {
            !label
                .sources
                .iter()
                .all(|binding| self.binding_current(binding))
        }) {
            return Err(invalid(
                "Source recordings changed; review before undoing bound labels.",
            ));
        }
        self.transaction(|next| {
            if redo {
                next.reference_redo.pop();
                next.reference_undo.push(next.references.clone());
            } else {
                next.reference_undo.pop();
                next.reference_redo.push(next.references.clone());
            }
            next.references = labels;
            next.invalidate("Reference labels changed; Analyze is required.");
            Ok(())
        })?;
        Ok(true)
    }
    pub fn replace_manual_gain_edits(&mut self, edits: Vec<ManualGainEdit>) -> io::Result<()> {
        self.replace_gain_constraints(GainConstraints {
            manual_edits: edits,
            protections: self.bound_protections.clone(),
        })
    }
    pub fn gain_constraints(&self) -> GainConstraints {
        GainConstraints {
            manual_edits: self.manual_edits.clone(),
            protections: self.bound_protections.clone(),
        }
    }
    pub fn begin_gain_edit(&self) -> GainEditDraft {
        let base = self.gain_constraints();
        GainEditDraft {
            constraints: base.clone(),
            base,
        }
    }
    /// A region action is a draft so conflicting records can be explicitly replaced before one commit.
    pub fn region_gain_edit(
        &self,
        track: TrackId,
        range: TimeRange,
        action: GainRegionAction,
    ) -> io::Result<GainEditDraft> {
        let source = self.source(track)?;
        let binding = SourceBinding {
            track,
            fingerprint: source.fingerprint.clone(),
        };
        let mut draft = self.begin_gain_edit();
        match action {
            GainRegionAction::Preserve => draft.constraints.protections.push(BoundProtection {
                track: Some(track),
                range,
                sources: vec![binding],
            }),
            GainRegionAction::Attenuate(gain) => {
                if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
                    return Err(invalid("Region gain must be finite attenuation."));
                }
                let id = draft
                    .constraints
                    .manual_edits
                    .iter()
                    .map(|edit| edit.id)
                    .max()
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or_else(|| invalid("Manual gain edit IDs exhausted."))?;
                draft.constraints.manual_edits.push(ManualGainEdit {
                    id,
                    source: binding,
                    points: vec![
                        EnvelopePoint {
                            time: range.start,
                            gain,
                        },
                        EnvelopePoint {
                            time: range.end,
                            gain,
                        },
                    ],
                    interpolation: ManualInterpolation::Linear,
                });
            }
        }
        Ok(draft)
    }
    pub fn validate_gain_edit(&self, draft: &GainEditDraft) -> io::Result<()> {
        if draft.base != self.gain_constraints() {
            return Err(invalid(
                "Gain constraints changed during this edit; start a new drag.",
            ));
        }
        self.validate_gain_constraints(&draft.constraints)
    }
    pub fn validate_gain_constraints(&self, constraints: &GainConstraints) -> io::Result<()> {
        if constraints
            .manual_edits
            .iter()
            .any(|e| !self.binding_current(&e.source))
            || constraints
                .protections
                .iter()
                .any(|p| p.sources.iter().any(|b| !self.binding_current(b)))
        {
            return Err(invalid(
                "Gain constraints require current, reviewed source bindings.",
            ));
        }
        let mut next = self.clone();
        next.manual_edits = constraints.manual_edits.clone();
        next.bound_protections = constraints.protections.clone();
        next.validate_contract()
    }
    pub(crate) fn validate_interactive_sources(&self) -> io::Result<()> {
        if self.sources.iter().any(|s| {
            !self.binding_current(&SourceBinding {
                track: s.track_id,
                fingerprint: s.fingerprint.clone(),
            })
        }) {
            return Err(invalid(
                "Sources changed; relink and review before applying envelopes.",
            ));
        }
        Ok(())
    }
    pub fn commit_gain_edit(&mut self, draft: GainEditDraft) -> io::Result<()> {
        self.validate_gain_edit(&draft)?;
        self.replace_gain_constraints(draft.constraints)
    }
    pub fn replace_gain_constraints(&mut self, constraints: GainConstraints) -> io::Result<()> {
        self.validate_gain_constraints(&constraints)?;
        if self.gain_constraints() == constraints {
            return Ok(());
        }
        self.transaction(|next| {
            next.gain_undo.push(next.gain_constraints());
            next.gain_redo.clear();
            next.manual_edits = constraints.manual_edits;
            next.bound_protections = constraints.protections;
            next.render = RenderState::Stale;
            Ok(())
        })
    }
    pub fn undo_gain_edit(&mut self) -> io::Result<bool> {
        self.restore_gain_history(false)
    }
    pub fn redo_gain_edit(&mut self) -> io::Result<bool> {
        self.restore_gain_history(true)
    }
    fn restore_gain_history(&mut self, redo: bool) -> io::Result<bool> {
        let history = if redo {
            &self.gain_redo
        } else {
            &self.gain_undo
        };
        let Some(constraints) = history.last().cloned() else {
            return Ok(false);
        };
        self.validate_gain_constraints(&constraints)?;
        self.transaction(|next| {
            let current = next.gain_constraints();
            if redo {
                next.gain_redo.pop();
                next.gain_undo.push(current);
            } else {
                next.gain_undo.pop();
                next.gain_redo.push(current);
            }
            next.manual_edits = constraints.manual_edits;
            next.bound_protections = constraints.protections;
            next.render = RenderState::Stale;
            Ok(())
        })?;
        Ok(true)
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
        self.replace_gain_constraints(GainConstraints {
            manual_edits: self.manual_edits.clone(),
            protections: bound,
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
        let replacement = SourceIdentity::read(
            crate::AudioSource {
                path,
                channel: old.channel,
                name: old.name.clone(),
            },
            old.track_id,
            old.speaker_id,
        )?;
        let changed =
            replacement.fingerprint != old.fingerprint || replacement.metadata != old.metadata;
        self.transaction(|next|{
            let id=replacement.track_id;next.sources[index]=replacement;
            if changed { next.supplied_alignment = false; }
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
        self.add_audio_sources(paths.into_iter().map(crate::AudioSource::mono).collect())
    }

    pub fn add_audio_sources(&mut self, inputs: Vec<crate::AudioSource>) -> io::Result<()> {
        let first = self
            .sources
            .iter()
            .map(|s| s.track_id.0.max(s.speaker_id.0))
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| invalid("Track ID overflow."))?;
        let sources = inputs
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
            next.supplied_alignment = false;
            next.step = WorkflowStep::Process;
            next.invalidate("Session tracks changed; Analyze is required.");
            Ok(())
        })
    }
}

impl SourceIdentity {
    pub fn audio_source(&self) -> crate::AudioSource {
        crate::AudioSource {
            path: self.path.clone(),
            channel: self.channel,
            name: self.name.clone(),
        }
    }

    pub fn display_name(&self) -> String {
        self.audio_source().display_name()
    }

    fn stamp_is_current(&self) -> bool {
        self.loaded_stamp.as_ref().is_some_and(|stamp| {
            SourceStamp::read_source(&self.audio_source()).is_ok_and(|current| current == *stamp)
        })
    }
    fn read(
        source: crate::AudioSource,
        track_id: TrackId,
        speaker_id: SpeakerId,
    ) -> io::Result<Self> {
        let stamp = SourceStamp::read_source(&source).map_err(io_error)?;
        let metadata = source.metadata(None).map_err(io_error)?;
        let fingerprint = source.fingerprint(None)?;
        if SourceStamp::read_source(&source).map_err(io_error)? != stamp {
            return Err(invalid("Source changed while recording its identity."));
        }
        Ok(Self {
            track_id,
            speaker_id,
            path: source.path,
            channel: source.channel,
            name: source.name,
            fingerprint,
            metadata,
            available: true,
            loaded_stamp: Some(stamp),
        })
    }
    fn matches_disk(&self) -> bool {
        self.audio_source()
            .fingerprint(None)
            .is_ok_and(|h| h == self.fingerprint)
            && self
                .audio_source()
                .metadata(None)
                .is_ok_and(|m| m == self.metadata)
    }
}
fn invalid_json(error: serde_json::Error) -> io::Error {
    invalid(error.to_string())
}
pub(crate) fn fingerprint_cancellable(
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
pub(crate) fn atomic_save(path: &Path, bytes: &[u8]) -> io::Result<()> {
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
