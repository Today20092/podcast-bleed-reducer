//! Explicit supervised jobs and current-result installation, independent of widgets.
use crate::{Project, RenderBlocker, SessionOwnership};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

pub const EDITOR_ANALYSIS_RECIPE: &str =
    "offline-supervised-optional-native-acoustic-model09-temporal10-v2";

pub fn verify_installed_editor_runtime(
    runtime: &crate::model::RuntimeProvenance,
    cancel: &AtomicBool,
) -> Result<(), RenderBlocker> {
    let helper = std::env::current_exe()?.with_file_name(if cfg!(windows) {
        "model-check.exe"
    } else {
        "model-check"
    });
    let native = crate::model::runtime_directory().map_err(RenderBlocker)?;
    let mut command = std::process::Command::new(helper);
    command.arg("--verify-only").arg("--runtime").arg(native);
    let actual = crate::model::verify_native_runtime(command, cancel).map_err(RenderBlocker)?;
    if &actual != runtime {
        return Err(RenderBlocker(
            "Installed model/runtime changed; verify it before final delivery.".into(),
        ));
    }
    Ok(())
}

pub fn original_envelopes(project: &Project) -> Result<Vec<crate::GainEnvelope>, RenderBlocker> {
    crate::engine::validate_source_metadata(
        &project
            .sources
            .iter()
            .map(|source| source.metadata)
            .collect::<Vec<_>>(),
    )?;
    project
        .validate_interactive_sources()
        .map_err(|e| RenderBlocker(e.to_string()))?;
    project
        .sources
        .iter()
        .map(|source| {
            crate::GainEnvelope::constant_at_rate(
                usize::try_from(source.metadata.grid.frame_count)
                    .map_err(|_| RenderBlocker("Source is too large for playback.".into()))?,
                source.metadata.grid.sample_rate,
                1.0,
            )
        })
        .collect()
}

pub fn editor_audition_envelopes(
    project: &Project,
    preview: Option<&crate::AnalysisPreview>,
    mode: crate::AuditionMode,
) -> Result<Vec<crate::GainEnvelope>, RenderBlocker> {
    if let Some(preview) = preview {
        if preview.stems != project.source_paths()
            || preview.envelopes.len() != project.sources.len()
            || preview
                .envelopes
                .iter()
                .zip(&project.sources)
                .any(|(envelope, source)| envelope.source_grid() != source.metadata.grid)
        {
            return Err(RenderBlocker(
                "Preview sources changed; Analyze again before comparison.".into(),
            ));
        }
        project
            .validate_interactive_sources()
            .map_err(|e| RenderBlocker(e.to_string()))?;
        return Ok(preview.envelopes.clone());
    }
    if mode != crate::AuditionMode::Original {
        return Err(RenderBlocker(
            "Analyze before Processed or Removed playback.".into(),
        ));
    }
    original_envelopes(project)
}

#[derive(Clone, Debug)]
pub struct GainPointDrag {
    draft: crate::GainEditDraft,
    edit: usize,
    point: usize,
    duration: u64,
}
impl GainPointDrag {
    pub fn begin(
        project: &Project,
        track: crate::TrackId,
        id: u64,
        point: usize,
    ) -> Result<Self, RenderBlocker> {
        let draft = project.begin_gain_edit();
        let edit = draft
            .constraints
            .manual_edits
            .iter()
            .position(|edit| edit.id == id && edit.source.track == track)
            .ok_or_else(|| RenderBlocker("Choose a manual envelope point.".into()))?;
        if point >= draft.constraints.manual_edits[edit].points.len() {
            return Err(RenderBlocker("Manual point is unavailable.".into()));
        }
        project
            .validate_gain_edit(&draft)
            .map_err(|e| RenderBlocker(e.to_string()))?;
        let duration = project
            .sources
            .iter()
            .find(|s| s.track_id == track)
            .ok_or_else(|| RenderBlocker("Track is unavailable.".into()))?
            .metadata
            .grid
            .duration()?;
        Ok(Self {
            draft,
            edit,
            point,
            duration,
        })
    }
    pub fn move_to(&mut self, time: u64, gain: f32) -> Result<(), RenderBlocker> {
        if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
            return Err(RenderBlocker("Gain must be finite attenuation.".into()));
        }
        let points = &mut self.draft.constraints.manual_edits[self.edit].points;
        let lower = if self.point == 0 {
            0
        } else {
            points[self.point - 1].time + 1
        };
        let upper = if self.point + 1 == points.len() {
            self.duration
        } else {
            points[self.point + 1].time - 1
        };
        points[self.point] = crate::EnvelopePoint {
            time: time.clamp(lower, upper),
            gain,
        };
        Ok(())
    }
    pub fn draft(&self) -> &crate::GainEditDraft {
        &self.draft
    }
    pub fn finish(self) -> crate::GainEditDraft {
        self.draft
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorAnalysisIdentity {
    pub key: String,
    pub project: std::path::PathBuf,
}

#[derive(Clone, Debug)]
pub struct EditorAnalysisResult {
    pub snapshot: EditorAnalysisIdentity,
    pub ownership: Arc<SessionOwnership>,
    pub temporal: Arc<crate::TemporalOwnership>,
    pub overview: crate::OwnershipOverview,
    pub recipe: crate::AnalysisRecipe,
    pub runtime: Option<crate::model::RuntimeProvenance>,
    pub timings: EditorAnalysisTimings,
    pub fixed_evidence: Option<crate::model::DiarizationEvidence>,
}

/// Wall-clock observations only; these never participate in sound or cache identity.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct EditorAnalysisTimings {
    pub source_validation_ms: f64,
    pub audio_preparation_ms: f64,
    pub reference_readiness_ms: f64,
    pub runtime_verification_ms: f64,
    pub native_history_ms: f64,
    pub ownership_ms: f64,
    pub temporal_ms: f64,
    pub overview_ms: f64,
}

#[derive(Default)]
pub struct EditorAnalysisWorker {
    ownership: crate::SessionOwnershipCache,
    temporal: crate::TemporalOwnershipCache,
    accepted: Option<Arc<EditorAnalysisResult>>,
}
impl EditorAnalysisWorker {
    pub fn last_completed(&self) -> Option<&Arc<EditorAnalysisResult>> {
        self.accepted.as_ref()
    }
    pub fn stats(&self) -> crate::OwnershipCacheStats {
        self.ownership.stats()
    }
    pub fn analyze(
        &mut self,
        project: &Project,
        cache: &Path,
        runtime: Option<&crate::model::RuntimeProvenance>,
        progress: &AtomicU8,
        cancel: &AtomicBool,
    ) -> Result<Arc<EditorAnalysisResult>, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(Some(cancel))?;
        let mut timings = EditorAnalysisTimings::default();
        let stage = std::time::Instant::now();
        let snapshot = EditorAnalysisIdentity::capture(project)?;
        project
            .validate_sources_cancellable(Some(cancel))
            .map_err(|e| RenderBlocker(e.to_string()))?;
        timings.source_validation_ms = stage.elapsed().as_secs_f64() * 1000.0;
        progress.store(5, Ordering::Relaxed);
        let stage = std::time::Instant::now();
        let audio = project
            .source_paths()
            .iter()
            .map(|path| crate::prepare_analysis_audio_in(path, &cache.join("audio"), Some(cancel)))
            .collect::<Result<Vec<_>, _>>()?;
        timings.audio_preparation_ms = stage.elapsed().as_secs_f64() * 1000.0;
        let stage = std::time::Instant::now();
        let readiness = crate::reference_readiness(project, &audio, Some(cancel))?;
        timings.reference_readiness_ms = stage.elapsed().as_secs_f64() * 1000.0;
        if !readiness.ready() {
            return Err(RenderBlocker(format!(
                "Required references need correction: missing solo speakers {:?}; room tone usable: {}; {}",
                readiness.missing_speakers,
                readiness.room_tone,
                readiness.issues.join("; ")
            )));
        }
        if !project.nemotron_enabled() {
            return self.analyze_evidence(
                project,
                &audio,
                None,
                (snapshot, timings),
                progress,
                cancel,
            );
        }
        let runtime = runtime.ok_or_else(|| RenderBlocker("Nemotron is enabled: verify the installed model, or disable Nemotron for acoustic analysis.".into()))?;
        progress.store(20, Ordering::Relaxed);
        let stage = std::time::Instant::now();
        let helper = std::env::current_exe()?.with_file_name(if cfg!(windows) {
            "model-check.exe"
        } else {
            "model-check"
        });
        let native = crate::model::runtime_directory().map_err(RenderBlocker)?;
        let mut verification = std::process::Command::new(&helper);
        verification
            .arg("--verify-only")
            .arg("--runtime")
            .arg(&native);
        let actual =
            crate::model::verify_native_runtime(verification, cancel).map_err(RenderBlocker)?;
        if &actual != runtime {
            return Err(RenderBlocker(
                "Installed native model/runtime changed; verify it again before Analyze.".into(),
            ));
        }
        timings.runtime_verification_ms = stage.elapsed().as_secs_f64() * 1000.0;
        let stage = std::time::Instant::now();
        let evidence = crate::model::prepare_diarization(
            &audio
                .iter()
                .map(crate::model::DiarizationInput::from)
                .collect::<Vec<_>>(),
            runtime,
            &helper,
            &native,
            &cache.join("nemotron"),
            cancel,
        )
        .map_err(RenderBlocker)?;
        timings.native_history_ms = stage.elapsed().as_secs_f64() * 1000.0;
        self.analyze_evidence(
            project,
            &audio,
            Some(&evidence),
            (snapshot, timings),
            progress,
            cancel,
        )
    }
    /// Verified fixed evidence is also the headless evaluation boundary; never guessed labels.
    pub fn analyze_verified_evidence(
        &mut self,
        project: &Project,
        audio: &[crate::PreparedAnalysisAudio],
        evidence: &crate::model::DiarizationEvidence,
        progress: &AtomicU8,
        cancel: &AtomicBool,
    ) -> Result<Arc<EditorAnalysisResult>, RenderBlocker> {
        if !project.nemotron_enabled() {
            return Err(RenderBlocker(
                "Enable Nemotron before supplying native evidence.".into(),
            ));
        }
        let snapshot = EditorAnalysisIdentity::capture(project)?;
        self.analyze_evidence(
            project,
            audio,
            Some(evidence),
            (snapshot, EditorAnalysisTimings::default()),
            progress,
            cancel,
        )
    }
    pub fn analyze_acoustic_evidence(
        &mut self,
        project: &Project,
        audio: &[crate::PreparedAnalysisAudio],
        progress: &AtomicU8,
        cancel: &AtomicBool,
    ) -> Result<Arc<EditorAnalysisResult>, RenderBlocker> {
        if project.nemotron_enabled() {
            return Err(RenderBlocker(
                "Disable Nemotron before acoustic-only analysis.".into(),
            ));
        }
        let snapshot = EditorAnalysisIdentity::capture(project)?;
        self.analyze_evidence(
            project,
            audio,
            None,
            (snapshot, EditorAnalysisTimings::default()),
            progress,
            cancel,
        )
    }
    fn analyze_evidence(
        &mut self,
        project: &Project,
        audio: &[crate::PreparedAnalysisAudio],
        evidence: Option<&crate::model::DiarizationEvidence>,
        observed: (EditorAnalysisIdentity, EditorAnalysisTimings),
        progress: &AtomicU8,
        cancel: &AtomicBool,
    ) -> Result<Arc<EditorAnalysisResult>, RenderBlocker> {
        let (snapshot, mut timings) = observed;
        progress.store(55, Ordering::Relaxed);
        let stage = std::time::Instant::now();
        let ownership = if let Some(evidence) = evidence {
            self.ownership.analyze(
                project,
                audio,
                evidence,
                crate::OwnershipTrainingSettings::default(),
                Some(cancel),
            )?
        } else {
            self.ownership.analyze_acoustic(
                project,
                audio,
                crate::OwnershipTrainingSettings::default(),
                Some(cancel),
            )?
        };
        timings.ownership_ms = stage.elapsed().as_secs_f64() * 1000.0;
        progress.store(80, Ordering::Relaxed);
        let stage = std::time::Instant::now();
        let settings = if evidence.is_some() {
            crate::TemporalSettings::default()
        } else {
            // Development acoustic profile bridges phrase gaps up to 768 ms.
            crate::TemporalSettings {
                preserved_gap_frames: 24,
                ..crate::TemporalSettings::default()
            }
        };
        let temporal = self
            .temporal
            .optimize(project, &ownership, settings, Some(cancel))?;
        timings.temporal_ms = stage.elapsed().as_secs_f64() * 1000.0;
        let recipe = crate::AnalysisRecipe {
            decoder: crate::AUDIO_DECODER_RECIPE.into(),
            resampler: crate::ANALYSIS_RESAMPLER_RECIPE.into(),
            feature_recipe: format!(
                "{EDITOR_ANALYSIS_RECIPE}/{}",
                crate::OWNERSHIP_FEATURE_RECIPE
            ),
            model_sha256: evidence.map(|e| e.model_sha256.clone()),
            runtime_sha256: evidence.map(|e| e.runtime.recipe_sha256.clone()),
            mix_recipe: evidence.map_or_else(
                || "individual-native-tracks-acoustic-v1".into(),
                |e| e.mix_recipe.clone(),
            ),
        };
        crate::prepared_analysis::check_cancelled(Some(cancel))?;
        if !snapshot.matches(project) {
            return Err(RenderBlocker(
                "Sources or references changed before analysis completed.".into(),
            ));
        }
        let stage = std::time::Instant::now();
        let overview = crate::OwnershipOverview::new_cancellable(&temporal, Some(cancel))?;
        timings.overview_ms = stage.elapsed().as_secs_f64() * 1000.0;
        if !snapshot.matches(project) {
            return Err(RenderBlocker(
                "Sources or references changed before analysis completed.".into(),
            ));
        }
        let result = Arc::new(EditorAnalysisResult {
            snapshot,
            ownership,
            temporal,
            overview,
            recipe,
            runtime: evidence.map(|e| e.runtime.clone()),
            timings,
            fixed_evidence: evidence.cloned(),
        });
        self.accepted = Some(result.clone());
        progress.store(95, Ordering::Relaxed);
        Ok(result)
    }
}

pub fn project_fade_settings(project: &Project) -> Result<crate::GainFadeSettings, RenderBlocker> {
    let parameters = project.parameters();
    let duration = |ms: f32| -> Result<u64, RenderBlocker> {
        if !ms.is_finite() || ms <= 0.0 {
            return Err(RenderBlocker(
                "Fade durations must be finite and positive.".into(),
            ));
        }
        crate::time_from_seconds(f64::from(ms) / 1000.0, crate::Rounding::Ceil)
    };
    Ok(crate::GainFadeSettings {
        attenuation: 10_f32.powf(parameters.attenuation_db / 20.0),
        fade_in: duration(parameters.open_ms)?,
        fade_out: duration(parameters.close_ms)?,
    })
}

impl EditorAnalysisResult {
    /// Gain changes made during the job are reapplied from the current project.
    pub fn install(
        &self,
        project: &mut Project,
        envelopes: &mut crate::EffectiveEnvelopeCache,
        runtime: Option<&crate::model::RuntimeProvenance>,
        cancel: Option<&AtomicBool>,
    ) -> Result<Arc<crate::EffectiveEnvelopes>, RenderBlocker> {
        if runtime != self.runtime.as_ref()
            || project.nemotron_enabled() != self.runtime.is_some()
            || (self.runtime.is_some()
                && self.recipe.model_sha256.as_deref() != Some(crate::model::NEMOTRON.sha256))
        {
            return Err(RenderBlocker(
                "Required model or runtime changed; verify and Analyze again.".into(),
            ));
        }
        if !self.snapshot.matches(project) {
            return Err(RenderBlocker(
                "Analysis is obsolete; current sources and references require Analyze.".into(),
            ));
        }
        let mut staged = envelopes.clone();
        let result = staged.update(
            project,
            &self.temporal,
            project_fade_settings(project)?,
            cancel,
        )?;
        if !result.analysis_current {
            return Err(RenderBlocker("Supervised analysis is not current.".into()));
        }
        crate::prepared_analysis::check_cancelled(cancel)?;
        project
            .set_result(
                crate::ResultState::Current {
                    identity: self.temporal.cache_key.clone(),
                },
                Some(self.recipe.clone()),
            )
            .map_err(|e| RenderBlocker(e.to_string()))?;
        *envelopes = staged;
        Ok(result)
    }
    pub fn preview(
        &self,
        project: &Project,
        effective: &crate::EffectiveEnvelopes,
    ) -> crate::AnalysisPreview {
        crate::AnalysisPreview {
            stems: project.source_paths(),
            regions: self.temporal.engine_regions(),
            envelopes: effective.envelopes.clone(),
            protections: project.protections().to_vec(),
            parameters: project.parameters(),
        }
    }
}
impl EditorAnalysisIdentity {
    pub fn capture(project: &Project) -> Result<Self, RenderBlocker> {
        project
            .require_alignment()
            .map_err(|e| RenderBlocker(e.to_string()))?;
        project
            .validate_interactive_sources()
            .map_err(|e| RenderBlocker(e.to_string()))?;
        Ok(Self {
            project: project.path().into(),
            key: format!(
                "{:x}",
                Sha256::digest(
                    serde_json::to_vec(&(
                        EDITOR_ANALYSIS_RECIPE,
                        SessionOwnership::reference_identity(project),
                        project.supplied_alignment(),
                        project.nemotron_enabled(),
                    ))
                    .map_err(|e| RenderBlocker(e.to_string()))?
                )
            ),
        })
    }
    pub fn matches(&self, project: &Project) -> bool {
        Self::capture(project).as_ref().ok() == Some(self)
    }
}
