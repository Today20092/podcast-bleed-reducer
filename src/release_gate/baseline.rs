//! Frozen build receipts are pinned before comparison, never inferred from scores.
use crate::RenderBlocker;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub const FROZEN_BASELINE_COMMIT: &str = "6a60cc2bfe187ea889fd2d8bcc3344311cfd706b";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenExample {
    kind: crate::CalibrationKind,
    track: usize,
    start: u64,
    end: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenSettings {
    enabled: bool,
    examples: Vec<FrozenExample>,
    sources: Vec<crate::SourceStamp>,
}

/// The historical executable uses native 48 kHz sample units, unlike today's common-time examples.
pub fn validate_frozen_development_settings(
    project: &crate::Project,
    json: &str,
    cancel: Option<&AtomicBool>,
) -> Result<(), RenderBlocker> {
    crate::prepared_analysis::check_cancelled(cancel)?;
    project.require_alignment()?;
    project.validate_sources_cancellable(cancel)?;
    let settings: FrozenSettings =
        serde_json::from_str(json).map_err(|e| RenderBlocker(e.to_string()))?;
    let stamps = project
        .sources
        .iter()
        .map(|s| crate::SourceStamp::read(&s.path))
        .collect::<Result<Vec<_>, _>>()?;
    if !settings.enabled
        || settings.examples.is_empty()
        || settings.sources != stamps
        || project
            .sources
            .iter()
            .any(|s| s.metadata.grid.sample_rate != 48000)
    {
        return Err(RenderBlocker(
            "Frozen comparison requires enabled source-bound 48 kHz development settings.".into(),
        ));
    }
    for example in settings.examples {
        crate::prepared_analysis::check_cancelled(cancel)?;
        let source = project
            .sources
            .get(example.track)
            .ok_or_else(|| RenderBlocker("Frozen calibration names an unknown source.".into()))?;
        if example.end <= example.start
            || example.end > source.metadata.grid.frame_count
            || example.end - example.start < 12000
        {
            return Err(RenderBlocker("Frozen calibration range is invalid.".into()));
        }
        let usable = project.reference_labels().iter().any(|label| {
            let kind_matches = match example.kind {
                crate::CalibrationKind::RoomTone => {
                    label.active_speakers.is_empty()
                        && label
                            .reference_track
                            .is_none_or(|track| track == source.track_id)
                }
                crate::CalibrationKind::SoloSpeech => label.active_speakers == [source.speaker_id],
            };
            kind_matches
                && label.sources.iter().any(|binding| {
                    binding.track == source.track_id && binding.fingerprint == source.fingerprint
                })
                && label
                    .range
                    .native_frames(source.metadata.grid)
                    .is_ok_and(|range| {
                        example.start >= range.start as u64 && example.end <= range.end as u64
                    })
        });
        if !usable {
            return Err(RenderBlocker("Frozen calibration is outside trusted development references; held-out truth cannot guide the baseline.".into()));
        }
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    name: String,
    path: PathBuf,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenBuildReceipt {
    version: u32,
    commit: String,
    checkout: PathBuf,
    checkout_clean: bool,
    lock_sha256: String,
    silero_model_sha256: String,
    rustc: String,
    build_command: Vec<String>,
    artifacts: Vec<Artifact>,
    allowed_variants: Vec<String>,
    comparison_requires_identical_48khz_sources: bool,
    quality_claim: String,
}
fn supported(variant: &str) -> bool {
    matches!(
        variant,
        "legacy_default" | "automatic_joint" | "guided_joint"
    )
}

/// Verify pinned receipt and actual executable bytes. Checkout provenance is a separate required check.
pub fn verify_frozen_artifacts(
    bytes: &[u8],
    expected_sha256: &str,
    cancel: Option<&AtomicBool>,
) -> Result<FrozenBuildReceipt, RenderBlocker> {
    crate::prepared_analysis::check_cancelled(cancel)?;
    if format!("{:x}", Sha256::digest(bytes)) != expected_sha256 {
        return Err(RenderBlocker(
            "Frozen baseline receipt checksum changed.".into(),
        ));
    }
    let receipt: FrozenBuildReceipt =
        serde_json::from_slice(bytes).map_err(|e| RenderBlocker(e.to_string()))?;
    if receipt.version != 1
        || receipt.commit != FROZEN_BASELINE_COMMIT
        || !receipt.checkout_clean
        || !receipt.comparison_requires_identical_48khz_sources
        || receipt.artifacts.is_empty()
        || receipt.allowed_variants.is_empty()
        || receipt.allowed_variants.iter().any(|v| !supported(v))
        || receipt.rustc.trim().is_empty()
        || !receipt.build_command.iter().any(|a| a == "--locked")
        || receipt.quality_claim.trim().is_empty()
    {
        return Err(RenderBlocker(
            "Unsupported frozen baseline build receipt.".into(),
        ));
    }
    let mut names = std::collections::BTreeSet::new();
    for artifact in &receipt.artifacts {
        if !names.insert(artifact.name.to_lowercase())
            || artifact.path.file_name().and_then(|s| s.to_str()) != Some(&artifact.name)
            || crate::project::fingerprint_cancellable(&artifact.path, cancel)? != artifact.sha256
        {
            return Err(RenderBlocker(
                "Frozen baseline artifact identity or bytes changed.".into(),
            ));
        }
    }
    if !names.contains("ownership_evaluate.exe") && !names.contains("ownership_evaluate") {
        return Err(RenderBlocker(
            "Frozen ownership evaluator is missing.".into(),
        ));
    }
    Ok(receipt)
}
impl FrozenBuildReceipt {
    pub fn verify_checkout(&self, cancel: Option<&AtomicBool>) -> Result<(), RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancel)?;
        let git = |args: &[&str]| -> Result<String, RenderBlocker> {
            let output = std::process::Command::new("git")
                .current_dir(&self.checkout)
                .args(args)
                .output()?;
            if !output.status.success() {
                return Err(RenderBlocker(
                    "Frozen checkout Git verification failed.".into(),
                ));
            }
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        };
        if git(&["rev-parse", "HEAD"])? != FROZEN_BASELINE_COMMIT
            || !git(&["status", "--porcelain"])?.is_empty()
            || crate::project::fingerprint_cancellable(&self.checkout.join("Cargo.lock"), cancel)?
                != self.lock_sha256
            || crate::project::fingerprint_cancellable(
                &self.checkout.join("assets/silero_vad.onnx"),
                cancel,
            )? != self.silero_model_sha256
        {
            return Err(RenderBlocker(
                "Frozen baseline checkout, lockfile or embedded model changed.".into(),
            ));
        }
        crate::prepared_analysis::check_cancelled(cancel)
    }
    pub fn executable(&self, variant: &str) -> Result<&Path, RenderBlocker> {
        if !supported(variant) || !self.allowed_variants.iter().any(|v| v == variant) {
            return Err(RenderBlocker(
                "Baseline variant was not frozen in the build receipt.".into(),
            ));
        }
        self.artifacts
            .iter()
            .find(|a| a.name == "ownership_evaluate.exe" || a.name == "ownership_evaluate")
            .map(|a| a.path.as_path())
            .ok_or_else(|| RenderBlocker("Frozen evaluator is missing.".into()))
    }
}
