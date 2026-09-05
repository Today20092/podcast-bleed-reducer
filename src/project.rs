use crate::{AutomixParameters, OwnershipSettings, Protection, SourceStamp};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceIdentity {
    pub path: PathBuf,
    pub fingerprint: String,
    #[serde(skip, default)]
    pub available: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    #[serde(skip)]
    path: PathBuf,
    pub sources: Vec<SourceIdentity>,
    protections: Vec<Protection>,
    #[serde(default)]
    parameters: AutomixParameters,
    #[serde(default)]
    ownership: OwnershipSettings,
    #[serde(skip)]
    calibration_notice: Option<String>,
    review_states: BTreeMap<String, ReviewState>,
    render: RenderState,
    #[serde(default)]
    last_render: Option<PathBuf>,
    step: WorkflowStep,
}

impl Project {
    pub fn create(path: impl Into<PathBuf>, sources: Vec<PathBuf>) -> io::Result<Self> {
        let sources = sources
            .into_iter()
            .map(SourceIdentity::read)
            .collect::<io::Result<_>>()?;
        let project = Self {
            path: path.into(),
            sources,
            protections: vec![],
            parameters: AutomixParameters::default(),
            ownership: OwnershipSettings {
                enabled: true,
                ..Default::default()
            },
            calibration_notice: None,
            review_states: BTreeMap::new(),
            render: RenderState::None,
            last_render: None,
            step: WorkflowStep::Validate,
        };
        project.save()?;
        Ok(project)
    }

    pub fn open(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let mut project: Self =
            serde_json::from_slice(&fs::read(&path)?).map_err(io::Error::other)?;
        project.path = path;
        if project.last_render.is_none()
            && let RenderState::Current { identity } = &project.render
        {
            project.last_render = Some(identity.clone());
        }
        let mut valid = true;
        for source in &mut project.sources {
            source.available = source.matches_disk();
            valid &= source.available;
        }
        if !valid {
            project.invalidate_calibration("Source recordings changed or are missing.");
        } else if let Err(error) = project.validate_ownership(&project.ownership) {
            project.invalidate_calibration(&error.to_string());
        }
        if !valid {
            project.step = WorkflowStep::Import;
            project.render = RenderState::Stale;
        } else if matches!(&project.render, RenderState::Current { identity } if !identity.is_file())
        {
            project.step = WorkflowStep::Process;
            project.render = RenderState::Stale;
        }
        Ok(project)
    }

    pub fn save(&self) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        fs::write(&self.path, bytes)
    }

    pub fn source_paths(&self) -> Vec<PathBuf> {
        self.sources
            .iter()
            .map(|source| source.path.clone())
            .collect()
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn step(&self) -> WorkflowStep {
        self.step
    }
    pub fn protections(&self) -> &[Protection] {
        &self.protections
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

    pub fn set_ownership_settings(&mut self, settings: OwnershipSettings) -> io::Result<()> {
        self.validate_ownership(&settings)?;
        let mut updated = self.clone();
        updated.ownership = settings;
        updated.calibration_notice = None;
        if matches!(updated.render, RenderState::Current { .. }) {
            updated.render = RenderState::Stale;
        }
        updated.save()?;
        *self = updated;
        Ok(())
    }

    fn validate_ownership(&self, settings: &OwnershipSettings) -> io::Result<()> {
        if settings.examples.is_empty() {
            return Ok(());
        }
        if self.sources.iter().any(|source| !source.available) {
            return Err(io::Error::other(
                "Relink unavailable sources before adding calibration examples.",
            ));
        }
        let stamps = self
            .sources
            .iter()
            .map(|source| SourceStamp::read(&source.path).map_err(io::Error::other))
            .collect::<io::Result<Vec<_>>>()?;
        // Header-only reads keep calibration edits independent of PCM size and inference.
        let counts = self
            .sources
            .iter()
            .map(|source| {
                let mut reader =
                    bwavfile::WaveReader::open(&source.path).map_err(io::Error::other)?;
                let format = reader.format().map_err(io::Error::other)?;
                if format.sample_rate != 48_000 || format.channel_count != 1 {
                    return Err(io::Error::other(
                        "Calibration requires 48 kHz mono sources.",
                    ));
                }
                usize::try_from(reader.frame_length().map_err(io::Error::other)?)
                    .map_err(io::Error::other)
            })
            .collect::<io::Result<Vec<_>>>()?;
        settings
            .validate(&stamps, &counts)
            .map_err(io::Error::other)
    }

    fn invalidate_calibration(&mut self, reason: &str) {
        if !self.ownership.examples.is_empty() {
            self.ownership.examples.clear();
            self.ownership.sources.clear();
            self.calibration_notice = Some(format!(
                "Calibration examples cleared. {reason} Select new examples for these sources."
            ));
            self.render = RenderState::Stale;
        }
    }
    pub fn review_states(&self) -> &BTreeMap<String, ReviewState> {
        &self.review_states
    }
    pub fn render(&self) -> &RenderState {
        &self.render
    }
    pub fn review_identity(&self) -> Option<&Path> {
        self.last_render.as_deref()
    }

    pub fn mark_validated(&mut self) -> io::Result<()> {
        self.step = WorkflowStep::Process;
        self.save()
    }

    pub fn replace_protections(&mut self, protections: Vec<Protection>) -> io::Result<()> {
        self.protections = protections;
        self.render = RenderState::Stale;
        self.save()
    }

    pub fn set_parameters(&mut self, parameters: AutomixParameters) -> io::Result<()> {
        self.parameters = parameters;
        if matches!(self.render, RenderState::Current { .. }) {
            self.render = RenderState::Stale;
        }
        self.save()
    }

    pub fn mark_render_stale(&mut self) -> io::Result<()> {
        self.render = RenderState::Stale;
        self.save()
    }

    pub fn protect_track(&mut self, track: usize) -> io::Result<()> {
        self.add_protections(vec![Protection {
            track: Some(track),
            start: 0,
            end: usize::MAX,
        }])
    }

    pub fn protect_range(&mut self, start: usize, end: usize) -> io::Result<()> {
        self.add_protections(vec![Protection {
            track: None,
            start,
            end,
        }])
    }

    pub fn protect_region(&mut self, tracks: &[usize], start: usize, end: usize) -> io::Result<()> {
        self.add_protections(
            tracks
                .iter()
                .map(|track| Protection {
                    track: Some(*track),
                    start,
                    end,
                })
                .collect(),
        )
    }

    fn add_protections(&mut self, protections: Vec<Protection>) -> io::Result<()> {
        self.protections.extend(protections);
        self.render = RenderState::Stale;
        self.save()
    }

    pub fn set_review_state(
        &mut self,
        warning: impl Into<String>,
        state: ReviewState,
    ) -> io::Result<()> {
        self.review_states.insert(warning.into(), state);
        self.save()
    }

    pub fn mark_render_current(&mut self, identity: impl Into<PathBuf>) -> io::Result<()> {
        let identity = identity.into();
        self.last_render = Some(identity.clone());
        self.render = RenderState::Current { identity };
        self.step = WorkflowStep::Review;
        self.save()
    }

    pub fn relink(&mut self, index: usize, path: PathBuf) -> io::Result<()> {
        let mut updated = self.clone();
        let source = updated
            .sources
            .get_mut(index)
            .ok_or_else(|| io::Error::other("Source track is not in this session."))?;
        *source = SourceIdentity::read(path)?;
        updated.invalidate_calibration("A source was relinked.");
        updated.step = WorkflowStep::Validate;
        updated.render = RenderState::Stale;
        updated.save()?;
        *self = updated;
        Ok(())
    }

    pub fn add_sources(&mut self, paths: Vec<PathBuf>) -> io::Result<()> {
        let mut updated = self.clone();
        updated.sources.extend(
            paths
                .into_iter()
                .map(SourceIdentity::read)
                .collect::<io::Result<Vec<_>>>()?,
        );
        updated.invalidate_calibration("The session's source tracks changed.");
        updated.step = WorkflowStep::Process;
        updated.render = RenderState::Stale;
        updated.save()?;
        *self = updated;
        Ok(())
    }
}

impl SourceIdentity {
    fn read(path: PathBuf) -> io::Result<Self> {
        let fingerprint = fingerprint(&path)?;
        Ok(Self {
            path,
            fingerprint,
            available: true,
        })
    }

    fn matches_disk(&self) -> bool {
        fingerprint(&self.path).is_ok_and(|value| value == self.fingerprint)
    }
}

fn fingerprint(path: &Path) -> io::Result<String> {
    let mut hash = Sha256::new();
    hash.update(fs::read(path)?);
    Ok(format!("{:x}", hash.finalize()))
}
