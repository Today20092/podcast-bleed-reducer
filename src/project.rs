use crate::Protection;
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

    pub fn protect_track(&mut self, track: usize) -> io::Result<()> {
        self.protections.push(Protection {
            track: Some(track),
            start: 0,
            end: usize::MAX,
        });
        self.render = RenderState::Stale;
        self.save()
    }

    pub fn protect_range(&mut self, start: usize, end: usize) -> io::Result<()> {
        self.protections.push(Protection {
            track: None,
            start,
            end,
        });
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
        self.sources[index] = SourceIdentity::read(path)?;
        self.step = WorkflowStep::Validate;
        self.render = RenderState::Stale;
        self.save()
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
