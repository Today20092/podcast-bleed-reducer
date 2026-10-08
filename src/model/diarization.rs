//! Complete project-owned native probability history, independent of speaker mapping.
use super::ProbabilityFrames;
use super::{MODEL_REVISION, ModelCache, NEMOTRON, NativeRuntime, RuntimeProvenance};
use crate::analysis_audio::{AnalysisAudioIdentity, PreparedAnalysisAudio};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
};
static JOB_ID: AtomicU64 = AtomicU64::new(0);

/// Fixed evidence only: callers must check their current job/source generation
/// before installing this result into a project or replacing its preview.
pub fn prepare_diarization(
    inputs: &[DiarizationInput],
    expected_runtime: &RuntimeProvenance,
    helper: &Path,
    runtime: &Path,
    cache: &Path,
    cancel: &AtomicBool,
) -> Result<DiarizationEvidence, String> {
    check_cancel(cancel)?;
    if inputs.is_empty() || inputs.len() > 8 {
        return Err("Nemotron requires one to eight dedicated mono inputs".into());
    }
    let sources: Vec<_> = inputs.iter().map(|i| i.identity.clone()).collect();
    let key = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&(
                1,
                &sources,
                expected_runtime,
                MODEL_REVISION,
                NEMOTRON.sha256,
                "v3-offline",
                "mean-global-peak-floor-one-v1"
            ))
            .map_err(|e| e.to_string())?
        )
    );
    let destination = cache.join(key);
    if destination.is_dir() {
        let mut result = read_result(&destination)?;
        validate_result(&result, &sources, expected_runtime)?;
        result.history.path = destination.join("probabilities.f32");
        result.history.verify_cancelled(cancel)?;
        return Ok(result);
    }
    std::fs::create_dir_all(cache).map_err(|e| e.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let owned = JobDirectory(cache.join(format!(
        ".native-{}-{stamp}-{}",
        std::process::id(),
        JOB_ID.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir(&owned.0).map_err(|e| e.to_string())?;
    let output = owned.0.join("result");
    let request = DiarizationRequest {
        inputs: inputs.to_vec(),
        output_directory: output.clone(),
        mix: AnalysisMix::MeanGlobalPeak,
    };
    let request_path = owned.0.join("request.json");
    std::fs::write(
        &request_path,
        serde_json::to_vec(&request).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut command = std::process::Command::new(helper);
    command
        .arg("--diarize")
        .arg(&request_path)
        .arg("--runtime")
        .arg(runtime);
    let actual = super::verify_native_runtime(command, cancel)?;
    if &actual != expected_runtime {
        return Err("Native helper runtime provenance changed during analysis".into());
    }
    let mut result = read_result(&output)?;
    validate_result(&result, &sources, expected_runtime)?;
    result.history.path = output.join("probabilities.f32");
    result.history.verify_cancelled(cancel)?;
    // Manifest paths are rebound to this cache; helper output cannot select another file.
    result.history.path = destination.join("probabilities.f32");
    let mut manifest = File::create(output.join("result.json")).map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut manifest, &result).map_err(|e| e.to_string())?;
    manifest.sync_all().map_err(|e| e.to_string())?;
    drop(manifest);
    check_cancel(cancel)?;
    std::fs::rename(&output, &destination)
        .map_err(|e| format!("Cannot publish complete native evidence: {e}"))?;
    Ok(result)
}

fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Native analysis cancelled; accepted results retained".into())
    } else {
        Ok(())
    }
}

struct JobDirectory(PathBuf);
impl Drop for JobDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn read_result(directory: &Path) -> Result<DiarizationEvidence, String> {
    let file = File::open(directory.join("result.json"))
        .map_err(|e| format!("Cannot read native evidence manifest: {e}"))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 1024 * 1024 {
        return Err("Native result manifest exceeds one MiB".into());
    }
    serde_json::from_reader(file).map_err(|e| format!("Invalid native evidence manifest: {e}"))
}

fn validate_result(
    result: &DiarizationEvidence,
    sources: &[AnalysisAudioIdentity],
    runtime: &RuntimeProvenance,
) -> Result<(), String> {
    let samples = sources
        .iter()
        .map(|s| s.frame_count)
        .max()
        .ok_or("Missing native sources")?;
    if result.schema != 1
        || result.sources != sources
        || &result.runtime != runtime
        || result.model_revision != MODEL_REVISION
        || result.model_sha256 != NEMOTRON.sha256
        || result.preset != "v3-offline"
        || result.mix_recipe != "mean-global-peak-floor-one-v1"
        || !result.mix_peak.is_finite()
        || result.mix_peak < 1.0
        || result.mix_gains.len() != sources.len()
        || result
            .mix_gains
            .iter()
            .any(|g| !g.is_finite() || *g <= 0.0 || *g > 1.0 / sources.len() as f64)
        || result.sample_count != samples
        || !result.reported_seconds_per_frame.is_finite()
        || (result.reported_seconds_per_frame - 0.01).abs() > 1e-9
        || result.history.frames < samples.div_ceil(160)
        || result.history.frames > samples.div_ceil(160) + 1
    {
        return Err(
            "Native evidence is incompatible with the current source/model/mix request".into(),
        );
    }
    Ok(())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DiarizationInput {
    pub identity: AnalysisAudioIdentity,
    pub path: PathBuf,
}

/// Development comparison; the editor uses MeanGlobalPeak for its fixed recipe.
#[derive(Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum AnalysisMix {
    #[default]
    MeanGlobalPeak,
    MeanPerTrackPeak,
}
impl From<&PreparedAnalysisAudio> for DiarizationInput {
    fn from(audio: &PreparedAnalysisAudio) -> Self {
        Self {
            identity: audio.identity.clone(),
            path: audio.path().into(),
        }
    }
}

/// Private, parent-owned job files; callers validate source alignment before inference.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DiarizationRequest {
    pub inputs: Vec<DiarizationInput>,
    pub output_directory: PathBuf,
    #[serde(default)]
    pub mix: AnalysisMix,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DiarizationEvidence {
    pub schema: u32,
    pub sources: Vec<AnalysisAudioIdentity>,
    pub history: ProbabilityHistory,
    pub runtime: RuntimeProvenance,
    pub model_revision: String,
    pub model_sha256: String,
    pub preset: String,
    pub mix_recipe: String,
    pub mix_peak: f32,
    pub mix_gains: Vec<f64>,
    pub sample_count: u64,
    pub reported_seconds_per_frame: f64,
    pub compactions: u64,
    pub maximum_retained_frames: u64,
}

/// Invoked only by the owned model-check process, never an editor/audio thread.
pub fn diarize_in_helper(
    request: &DiarizationRequest,
    runtime: &Path,
) -> Result<DiarizationEvidence, String> {
    if request.inputs.is_empty() || request.inputs.len() > 8 {
        return Err("Nemotron requires one to eight dedicated mono inputs".into());
    }
    let frames = request
        .inputs
        .iter()
        .map(|i| i.identity.frame_count)
        .max()
        .unwrap();
    if frames == 0
        || frames > 16_000 * 7200 + 1
        || request.inputs.iter().any(|i| {
            i.identity.sample_rate != 16_000
                || i.identity.source.channels != 1
                || i.identity.frame_count == 0
                || frames - i.identity.frame_count > 1
        })
    {
        return Err("Nemotron needs nonempty synchronized 16 kHz copies, at most two hours".into());
    }
    let mut files = Vec::new();
    let mut peak = 1.0f32;
    let mut track_peaks = Vec::new();
    for input in &request.inputs {
        let mut file =
            File::open(&input.path).map_err(|e| format!("Cannot open prepared analysis: {e}"))?;
        if file.metadata().map_err(|e| e.to_string())?.len() != input.identity.frame_count * 4 {
            return Err("Prepared analysis length changed before inference".into());
        }
        let mut bytes = [0u8; 64 * 1024];
        let mut track_peak = 0.0f32;
        let mut remaining = input.identity.frame_count * 4;
        while remaining > 0 {
            let n = remaining.min(bytes.len() as u64) as usize;
            file.read_exact(&mut bytes[..n])
                .map_err(|e| e.to_string())?;
            remaining -= n as u64;
            for b in bytes[..n].as_chunks::<4>().0 {
                let sample = f32::from_le_bytes(*b);
                if !sample.is_finite() {
                    return Err("Nonfinite prepared analysis sample".into());
                }
                peak = peak.max(sample.abs());
                track_peak = track_peak.max(sample.abs());
            }
        }
        file.rewind().map_err(|e| e.to_string())?;
        files.push(file);
        track_peaks.push(track_peak.max(1e-6));
    }
    std::fs::create_dir(&request.output_directory)
        .map_err(|e| format!("Cannot create exclusive native staging directory: {e}"))?;
    let runtime = NativeRuntime::open(runtime)?;
    let model = runtime.load_model(&ModelCache::default_location()?)?;
    let mut stream = model.stream()?;
    let mut writer =
        ProbabilityHistoryWriter::create(&request.output_directory.join("probabilities.f32"))?;
    let mut previous_end = 0;
    let mut previous_start = 0;
    let mut compactions = 0;
    let mut maximum_retained_frames = 0;
    let mut position = 0;
    while position < frames {
        let count = (frames - position).min(16_000) as usize;
        let mut mix = vec![0.0f32; count];
        for ((input, file), track_peak) in request.inputs.iter().zip(&mut files).zip(&track_peaks) {
            let count =
                (input.identity.frame_count.saturating_sub(position)).min(count as u64) as usize;
            let mut bytes = vec![0u8; count * 4];
            file.read_exact(&mut bytes)
                .map_err(|e| format!("Prepared analysis changed during inference: {e}"))?;
            for (value, b) in mix.iter_mut().zip(bytes.as_chunks::<4>().0) {
                let sample = f32::from_le_bytes(*b);
                if !sample.is_finite() || sample.abs() > peak {
                    return Err("Prepared analysis changed after fixed mix measurement".into());
                }
                let normalizer = match request.mix {
                    AnalysisMix::MeanGlobalPeak => peak,
                    AnalysisMix::MeanPerTrackPeak => *track_peak,
                };
                *value += (sample / normalizer) / request.inputs.len() as f32;
            }
        }
        // Bound roundoff from summing individually nonclipping contributions.
        for sample in &mut mix {
            *sample = sample.clamp(-1.0, 1.0);
        }
        stream.push(&mix)?;
        position += count as u64;
        if stream.frame_count()? != previous_end {
            let window = stream.probabilities()?;
            if window.start > previous_start {
                compactions += 1;
            }
            previous_start = window.start;
            previous_end = window.start + window.frames;
            maximum_retained_frames = maximum_retained_frames.max(window.frames);
            writer.append(&window)?;
        }
    }
    stream.finish()?;
    let window = stream.probabilities()?;
    if window.start > previous_start {
        compactions += 1;
    }
    maximum_retained_frames = maximum_retained_frames.max(window.frames);
    writer.append(&window)?;
    let history = writer.finish()?;
    // The native final tail can include a terminal frame; never silently discard it.
    if history.frames < frames.div_ceil(160) || history.frames > frames.div_ceil(160) + 1 {
        return Err("Native history does not cover the complete source timeline".into());
    }
    let evidence = DiarizationEvidence {
        schema: 1,
        sources: request.inputs.iter().map(|i| i.identity.clone()).collect(),
        history,
        runtime: runtime.provenance().clone(),
        model_revision: MODEL_REVISION.into(),
        model_sha256: NEMOTRON.sha256.into(),
        preset: "v3-offline".into(),
        mix_recipe: match request.mix {
            AnalysisMix::MeanGlobalPeak => "mean-global-peak-floor-one-v1",
            AnalysisMix::MeanPerTrackPeak => "mean-per-track-peak-v1",
        }
        .into(),
        mix_peak: peak,
        mix_gains: track_peaks
            .iter()
            .map(|track_peak| {
                1.0 / (match request.mix {
                    AnalysisMix::MeanGlobalPeak => peak,
                    AnalysisMix::MeanPerTrackPeak => *track_peak,
                } as f64
                    * request.inputs.len() as f64)
            })
            .collect(),
        sample_count: frames,
        reported_seconds_per_frame: model.cadence(),
        compactions,
        maximum_retained_frames,
    };
    let mut file =
        File::create(request.output_directory.join("result.json")).map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut file, &evidence).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    Ok(evidence)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProbabilityHistory {
    pub frames: u64,
    pub sha256: String,
    path: PathBuf,
}

impl ProbabilityHistory {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Verify persisted evidence before reusing it as a completed result.
    pub fn verify(&self) -> Result<(), String> {
        self.verify_cancelled(&AtomicBool::new(false))
    }

    fn verify_cancelled(&self, cancel: &AtomicBool) -> Result<(), String> {
        let mut file =
            File::open(&self.path).map_err(|e| format!("Cannot open native evidence: {e}"))?;
        let expected = self
            .frames
            .checked_mul(32)
            .ok_or("Native evidence length overflow")?;
        if file.metadata().map_err(|e| e.to_string())?.len() != expected {
            return Err("Native evidence payload length disagrees with its manifest".into());
        }
        let mut hash = Sha256::new();
        let mut bytes = [0; 64 * 1024];
        loop {
            check_cancel(cancel)?;
            let count = file
                .read(&mut bytes)
                .map_err(|e| format!("Cannot verify native evidence: {e}"))?;
            if count == 0 {
                break;
            }
            hash.update(&bytes[..count]);
        }
        if format!("{:x}", hash.finalize()) != self.sha256 {
            return Err("Native evidence payload failed SHA-256 validation".into());
        }
        Ok(())
    }

    /// Native 10 ms frames, with eight anonymous speaker columns per frame.
    pub fn read_frames(&self, range: Range<u64>) -> Result<Vec<f32>, String> {
        if range.start > range.end || range.end > self.frames || range.end - range.start > 65_536 {
            return Err(
                "Native probability read is outside history or exceeds 65,536 frames".into(),
            );
        }
        let mut file =
            File::open(&self.path).map_err(|e| format!("Cannot open native evidence: {e}"))?;
        file.seek(SeekFrom::Start(
            range
                .start
                .checked_mul(32)
                .ok_or("Native evidence offset overflow")?,
        ))
        .map_err(|e| e.to_string())?;
        let mut bytes = vec![0; (range.end - range.start) as usize * 32];
        file.read_exact(&mut bytes)
            .map_err(|e| format!("Incomplete native evidence: {e}"))?;
        let values: Vec<_> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect();
        if values
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("Native evidence contains invalid probabilities".into());
        }
        Ok(values)
    }
}

/// Appends copied native windows once, retaining rows that the SDK later compacts.
/// Callers own a staging directory; only completed, validated history can be installed.
pub struct ProbabilityHistoryWriter {
    path: PathBuf,
    file: BufWriter<File>,
    hash: Sha256,
    next_frame: u64,
}

impl ProbabilityHistoryWriter {
    pub fn create(path: &Path) -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("Cannot create native evidence staging file: {e}"))?;
        Ok(Self {
            path: path.into(),
            file: BufWriter::new(file),
            hash: Sha256::new(),
            next_frame: 0,
        })
    }

    pub fn append(&mut self, window: &ProbabilityFrames) -> Result<(), String> {
        let end = window
            .start
            .checked_add(window.frames)
            .ok_or("Native frame range overflow")?;
        let values = usize::try_from(window.frames)
            .ok()
            .and_then(|n| n.checked_mul(8))
            .ok_or("Native frame payload overflow")?;
        if window.speakers != 8
            || !window.seconds_per_frame.is_finite()
            || (window.seconds_per_frame - 0.01).abs() > 1e-9
            || window.probabilities.len() != values
            || window
                .probabilities
                .iter()
                .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
        {
            return Err("Incompatible or malformed native probability window".into());
        }
        if window.start > self.next_frame || end < self.next_frame {
            return Err("Native probability history has a compacted gap or regressing end".into());
        }
        let skip = usize::try_from(self.next_frame - window.start)
            .ok()
            .and_then(|n| n.checked_mul(8))
            .ok_or("Native frame prefix overflow")?;
        for chunk in window.probabilities[skip..].chunks(2048) {
            let bytes: Vec<_> = chunk.iter().flat_map(|p| p.to_le_bytes()).collect();
            self.file
                .write_all(&bytes)
                .map_err(|e| format!("Cannot retain native evidence: {e}"))?;
            self.hash.update(&bytes);
        }
        self.next_frame = end;
        Ok(())
    }

    pub fn finish(mut self) -> Result<ProbabilityHistory, String> {
        self.file
            .flush()
            .map_err(|e| format!("Cannot flush native evidence: {e}"))?;
        self.file
            .get_ref()
            .sync_all()
            .map_err(|e| format!("Cannot sync native evidence: {e}"))?;
        Ok(ProbabilityHistory {
            frames: self.next_frame,
            sha256: format!("{:x}", self.hash.finalize()),
            path: self.path,
        })
    }
}
