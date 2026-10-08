use super::{ModelCache, NEMOTRON, RUNTIME_REVISION, RuntimeProvenance};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, Receiver},
};

struct Job {
    cancel: Arc<AtomicBool>,
    progress: Arc<AtomicU64>,
    result: Receiver<Result<RuntimeProvenance, String>>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// First-use UI state. Workers touch only model storage; never project results.
pub struct ModelAcquisition {
    job: Option<Job>,
    status: String,
    ready: Option<RuntimeProvenance>,
}

impl Default for ModelAcquisition {
    fn default() -> Self {
        Self {
            job: None,
            status: "Required Nemotron model: not checked. Download or verify explicitly.".into(),
            ready: None,
        }
    }
}

impl ModelAcquisition {
    pub fn status(&self) -> &str {
        &self.status
    }
    pub fn is_busy(&self) -> bool {
        self.job.is_some()
    }
    pub fn is_ready(&self) -> bool {
        self.ready.is_some()
    }
    pub fn provenance(&self) -> Option<&RuntimeProvenance> {
        self.ready.as_ref()
    }
    pub fn progress(&self) -> f32 {
        self.job.as_ref().map_or(0.0, |j| {
            j.progress.load(Ordering::Relaxed) as f32 / NEMOTRON.bytes as f32
        })
    }

    pub fn begin(&mut self, download: bool) {
        if self.job.is_some() {
            return;
        }
        self.ready = None;
        self.status = if download {
            "Downloading/verifying required Nemotron model…"
        } else {
            "Checking installed Nemotron model offline…"
        }
        .into();
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicU64::new(0));
        let (sender, result) = mpsc::channel();
        let job_cancel = cancel.clone();
        let job_progress = progress.clone();
        let worker = std::thread::spawn(move || {
            let result = (|| {
                let runtime = runtime_directory()?;
                let cache = ModelCache::default_location()?;
                if cache.verified_path(&NEMOTRON).is_err() && download {
                    cache.download_required(&job_cancel, |n| {
                        job_progress.store(n, Ordering::Relaxed)
                    })?;
                }
                if job_cancel.load(Ordering::Relaxed) {
                    return Err("Model acquisition cancelled; existing results retained".into());
                }
                let helper = std::env::current_exe()
                    .map_err(|e| format!("Cannot locate required native verifier: {e}"))?
                    .with_file_name(if cfg!(windows) {
                        "model-check.exe"
                    } else {
                        "model-check"
                    });
                let mut command = Command::new(helper);
                command.arg("--runtime").arg(runtime);
                verify_native_runtime(command, &job_cancel)
            })();
            let _ = sender.send(result);
        });
        self.job = Some(Job {
            cancel,
            progress,
            result,
            worker: Some(worker),
        });
    }

    pub fn cancel(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Relaxed);
            self.status = "Cancelling model acquisition…".into();
        }
    }

    pub fn poll(&mut self) {
        let Some(job) = &self.job else {
            return;
        };
        let result = match job.result.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("Model worker stopped; retry explicit verification or acquisition".into())
            }
        };
        self.job = None;
        match result {
            Ok(provenance) => {
                self.ready = Some(provenance);
                self.status = "Nemotron V3 ready for offline CPU use".into();
            }
            Err(error) => {
                self.ready = None;
                self.status = error;
            }
        }
    }
}

/// Run the required native verifier outside the editor: a native abort must not
/// terminate Iced or invalidate model bytes or accepted project results.
/// Product callers construct the pinned helper command, never user shell input.
pub fn verify_native_runtime(
    mut command: Command,
    cancel: &AtomicBool,
) -> Result<RuntimeProvenance, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err(
            "Model verification cancelled; installed model and project results retained".into(),
        );
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW for the owned helper.
    }
    let mut child = command.spawn().map_err(|e| format!(
        "Cannot start required native model verifier: {e}. Reinstall the complete CPU package; installed model and project results retained."
    ))?;
    let stdout = child
        .stdout
        .take()
        .ok_or("Native verifier stdout unavailable")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("Native verifier stderr unavailable")?;
    let output = std::thread::spawn(move || read_bounded(stdout));
    let errors = std::thread::spawn(move || read_bounded(stderr));
    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            break Err(
                "Model verification cancelled; installed model and project results retained".into(),
            );
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(20)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(format!(
                    "Cannot monitor native verifier: {error}; installed model and project results retained"
                ));
            }
        }
    };
    let output = output
        .join()
        .map_err(|_| "Native verifier output reader stopped")??;
    let errors = errors
        .join()
        .map_err(|_| "Native verifier error reader stopped")??;
    let status = status?;
    if !status.success() {
        return Err(format!(
            "Native model verification process failed ({status}); installed model and project results retained. Check available memory and the pinned CPU package, then retry. {}",
            String::from_utf8_lossy(&errors)
        ));
    }
    let output = String::from_utf8_lossy(&output);
    let provenance = output
        .lines()
        .find_map(|line| line.strip_prefix("Runtime provenance: "))
        .ok_or("Required native verifier returned no verified provenance")?;
    let provenance: RuntimeProvenance = serde_json::from_str(provenance)
        .map_err(|e| format!("Invalid native verifier provenance: {e}"))?;
    if provenance.revision != RUNTIME_REVISION
        || provenance.recipe_sha256.len() != 64
        || !provenance
            .recipe_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || !provenance.binaries.contains_key("nemo_speech_asr_c.dll")
        || provenance
            .binaries
            .values()
            .any(|h| h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err("Required native verifier returned incompatible package provenance".into());
    }
    Ok(provenance)
}

fn read_bounded(mut source: impl Read) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let count = source
            .read(&mut buffer)
            .map_err(|e| format!("Cannot read native verifier output: {e}"))?;
        if count == 0 {
            return Ok(output);
        }
        // Drain all output, retaining the tail where the completed summary lives.
        output.extend_from_slice(&buffer[..count]);
        let excess = output.len().saturating_sub(64 * 1024);
        output.drain(..excess);
    }
}

pub fn runtime_directory() -> Result<PathBuf, String> {
    let executable =
        std::env::current_exe().map_err(|e| format!("Cannot locate app runtime: {e}"))?;
    let folder = executable.parent().ok_or("Cannot locate app folder")?;
    let deployed = folder.join("native-runtime");
    if deployed.is_dir() {
        return Ok(deployed);
    }
    // Development recipe installs the same package layout beside the repository.
    #[cfg(debug_assertions)]
    {
        let development = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("native-runtime/bin");
        if development.is_dir() {
            return Ok(development);
        }
    }
    Ok(deployed)
}
