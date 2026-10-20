//! Direct native delivery of an accepted sparse envelope, with private completion.
use crate::audio_io::AtomicOutput;
use crate::{AnalysisRecipe, EditorAnalysisResult, EffectiveEnvelopes, Project, RenderBlocker};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

#[derive(Default)]
pub struct RenderGeneration(pub(crate) Mutex<u64>);
impl RenderGeneration {
    pub fn current(&self) -> u64 {
        *self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn invalidate(&self) {
        let mut value = self.0.lock().unwrap_or_else(|e| e.into_inner());
        *value = value.wrapping_add(1);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisedRenderSnapshot {
    pub version: u32,
    pub project: PathBuf,
    pub sound_identity: String,
    pub analysis_identity: String,
    pub ownership_identity: String,
    pub effective_identity: String,
    pub recipe: AnalysisRecipe,
}
impl SupervisedRenderSnapshot {
    pub fn capture(
        project: &Project,
        analysis: &EditorAnalysisResult,
        effective: &EffectiveEnvelopes,
    ) -> Result<Self, RenderBlocker> {
        project.require_alignment()?;
        project.validate_interactive_sources()?;
        if !analysis.snapshot.matches(project)
            || !effective.analysis_current
            || project.analysis_recipe() != Some(&analysis.recipe)
            || !matches!(project.result(), crate::ResultState::Current { identity } if identity == &analysis.temporal.cache_key)
        {
            return Err(RenderBlocker(
                "Final render requires current supervised analysis and effective envelopes.".into(),
            ));
        }
        Ok(Self {
            version: 1,
            project: project.path().into(),
            sound_identity: project.sound_identity()?,
            analysis_identity: analysis.temporal.cache_key.clone(),
            ownership_identity: analysis.ownership.cache_key.clone(),
            effective_identity: effective.cache_key.clone(),
            recipe: analysis.recipe.clone(),
        })
    }
    pub fn matches(&self, project: &Project) -> bool {
        self.version == 1
            && self.project == project.path()
            && project
                .sound_identity()
                .is_ok_and(|key| key == self.sound_identity)
            && project.analysis_recipe() == Some(&self.recipe)
            && matches!(project.result(), crate::ResultState::Current { identity } if identity == &self.analysis_identity)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RenderedFile {
    pub name: String,
    pub sha256: String,
    pub metadata: crate::SourceMetadata,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RenderRecord {
    pub snapshot: SupervisedRenderSnapshot,
    pub report: PathBuf,
    pub report_sha256: String,
    pub files: Vec<RenderedFile>,
}

pub struct PreparedSupervisedRender {
    output: AtomicOutput,
    record: RenderRecord,
}
pub struct PublishedSupervisedRender {
    record: RenderRecord,
}
impl PublishedSupervisedRender {
    pub fn report_path(&self) -> &Path {
        &self.record.report
    }
    pub fn snapshot(&self) -> &SupervisedRenderSnapshot {
        &self.record.snapshot
    }
    pub fn accept(self, project: &mut Project) -> Result<(), RenderBlocker> {
        project.validate_interactive_sources()?;
        if !self.record.snapshot.matches(project) {
            return Err(RenderBlocker(
                "Final completion is obsolete; previous render retained.".into(),
            ));
        }
        project.install_render_record(self.record.clone())?;
        Ok(())
    }
}
pub fn prepare_supervised_render(
    project: &Project,
    analysis: &EditorAnalysisResult,
    effective: &EffectiveEnvelopes,
    snapshot: &SupervisedRenderSnapshot,
    destination: &Path,
    progress: &AtomicU8,
    cancelled: Option<&AtomicBool>,
) -> Result<PreparedSupervisedRender, RenderBlocker> {
    prepare_render(
        project,
        analysis,
        effective,
        snapshot,
        destination,
        progress,
        cancelled,
        crate::ExportEncoding::PreserveSource,
    )
}

/// Lossless f32 intermediate for final export encoding, before any integer dither.
pub fn prepare_supervised_float_render(
    project: &Project,
    analysis: &EditorAnalysisResult,
    effective: &EffectiveEnvelopes,
    snapshot: &SupervisedRenderSnapshot,
    destination: &Path,
    progress: &AtomicU8,
    cancelled: Option<&AtomicBool>,
) -> Result<PreparedSupervisedRender, RenderBlocker> {
    prepare_render(
        project,
        analysis,
        effective,
        snapshot,
        destination,
        progress,
        cancelled,
        crate::ExportEncoding::Float32,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_render(
    project: &Project,
    analysis: &EditorAnalysisResult,
    effective: &EffectiveEnvelopes,
    snapshot: &SupervisedRenderSnapshot,
    destination: &Path,
    progress: &AtomicU8,
    cancelled: Option<&AtomicBool>,
    encoding: crate::ExportEncoding,
) -> Result<PreparedSupervisedRender, RenderBlocker> {
    crate::prepared_analysis::check_cancelled(cancelled)?;
    if SupervisedRenderSnapshot::capture(project, analysis, effective)? != *snapshot {
        return Err(RenderBlocker("Final render snapshot changed.".into()));
    }
    project.validate_sources_cancellable(cancelled)?;
    if destination.exists() && fs::read_dir(destination)?.next().is_some() {
        return Err(RenderBlocker("output directory is not empty".into()));
    }
    let output = AtomicOutput::new(destination)?;
    // Recompose from the accepted immutable temporal result and current saved
    // constraints. No extractor, classifier, or retained fade generator runs.
    let effective = crate::EffectiveEnvelopeCache::default().update(
        project,
        &analysis.temporal,
        crate::project_fade_settings(project)?,
        cancelled,
    )?;
    if effective.cache_key != snapshot.effective_identity {
        return Err(RenderBlocker(
            "Effective envelope settings changed; rebuild before rendering.".into(),
        ));
    }
    let mut files = Vec::new();
    for (i, source) in project.sources.iter().enumerate() {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let name = format!(
            "{:02}-{}.wav",
            i + 1,
            source
                .display_name()
                .chars()
                .map(|c| if c.is_alphanumeric() || "-_ ".contains(c) {
                    c
                } else {
                    '_'
                })
                .collect::<String>()
        );
        let path = output.path(&name);
        let metadata = crate::audio_io::render_source_encoded(
            &source.audio_source(),
            &path,
            &effective.envelopes[i],
            encoding,
            cancelled,
        )?;
        if metadata != source.metadata {
            return Err(RenderBlocker(
                "Source native grid changed during render.".into(),
            ));
        }
        files.push(RenderedFile {
            name,
            metadata: crate::SourceMetadata::read_cancellable(&path, cancelled)?,
            sha256: crate::project::fingerprint_cancellable(&path, cancelled)?,
        });
        progress.store(
            ((i + 1) * 90 / project.sources.len()) as u8,
            Ordering::Relaxed,
        );
    }
    let mut regions = Vec::new();
    for (i, region) in analysis.temporal.regions().iter().enumerate() {
        if i % 1024 == 0 {
            crate::prepared_analysis::check_cancelled(cancelled)?;
        }
        let owners: Vec<_> = region
            .states
            .iter()
            .enumerate()
            .filter_map(|(i, s)| (*s == crate::TemporalTrackState::Owned).then_some(i))
            .collect();
        let decision = if region
            .states
            .contains(&crate::TemporalTrackState::UncertainUnity)
        {
            "Uncertain"
        } else if owners.is_empty() {
            "NoSpeech"
        } else {
            "Owned"
        };
        regions.push(serde_json::json!({"start":region.start,"end":region.end,"decision":decision,"owners":owners,"affected_track":null,"applied_attenuation_db":0,"evidence_reasons":[],"review_warning":null}));
    }
    let report = serde_json::to_string_pretty(&serde_json::json!({"version":1,"snapshot":snapshot,"input":{"track_grids":project.sources.iter().map(|s|s.metadata).collect::<Vec<_>>()},"output":{"stems":files.iter().map(|f|&f.name).collect::<Vec<_>>(),"files":files},"regions":regions,"supervised_warnings":crate::ReviewWorkspace::from_ownership(&analysis.temporal).warnings})) .map_err(|e|RenderBlocker(e.to_string()))?;
    output.write_report(&report)?;
    crate::prepared_analysis::check_cancelled(cancelled)?;
    progress.store(95, Ordering::Relaxed);
    Ok(PreparedSupervisedRender {
        output,
        record: RenderRecord {
            snapshot: snapshot.clone(),
            report: destination.join("report.json"),
            report_sha256: format!("{:x}", Sha256::digest(report.as_bytes())),
            files,
        },
    })
}
impl PreparedSupervisedRender {
    pub fn publish(
        self,
        project: &Project,
        generation: &RenderGeneration,
        expected_generation: u64,
        cancelled: Option<&AtomicBool>,
    ) -> Result<PublishedSupervisedRender, RenderBlocker> {
        project.validate_sources_cancellable(cancelled)?;
        for file in &self.record.files {
            let path = self.output.path(&file.name);
            if crate::project::fingerprint_cancellable(&path, cancelled)? != file.sha256
                || crate::SourceMetadata::read_cancellable(&path, cancelled)? != file.metadata
            {
                return Err(RenderBlocker(
                    "Private rendered payload changed before publication.".into(),
                ));
            }
        }
        if crate::project::fingerprint_cancellable(&self.output.path("report.json"), cancelled)?
            != self.record.report_sha256
        {
            return Err(RenderBlocker(
                "Private render provenance changed before publication.".into(),
            ));
        }
        self.output.sync_all()?;
        let _guard = generation.0.lock().unwrap_or_else(|e| e.into_inner());
        crate::prepared_analysis::check_cancelled(cancelled)?;
        if *_guard != expected_generation
            || !self.record.snapshot.matches(project)
            || Project::saved_sound_identity(project.path())? != self.record.snapshot.sound_identity
        {
            return Err(RenderBlocker(
                "Final render became obsolete before publication; previous outputs retained."
                    .into(),
            ));
        }
        project.validate_interactive_sources()?;
        crate::prepared_analysis::check_cancelled(cancelled)?;
        self.output.commit()?;
        Ok(PublishedSupervisedRender {
            record: self.record,
        })
    }
}
