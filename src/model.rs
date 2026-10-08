//! Explicit acquisition and verified offline Nemotron model storage.
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
mod native;
pub use native::{NativeModel, NativeRuntime, NativeStream, ProbabilityFrames, RuntimeProvenance};
mod acquisition;
pub use acquisition::ModelAcquisition;
pub use acquisition::runtime_directory;
pub use acquisition::verify_native_runtime;
mod diarization;
pub use diarization::{
    AnalysisMix, DiarizationEvidence, DiarizationInput, DiarizationRequest, ProbabilityHistory,
    ProbabilityHistoryWriter, diarize_in_helper, prepare_diarization,
};

pub struct ModelIdentity {
    pub file_name: &'static str,
    pub bytes: u64,
    pub sha256: &'static str,
}

pub const NEMOTRON: ModelIdentity = ModelIdentity {
    file_name: "Nemotron-3-Diarization.q8_0.gguf",
    bytes: 107_012_128,
    sha256: "08456d9e22cd9a323c0364d98375f3746d6e68507ebb705cd46438c534c7a3a1",
};
pub const MODEL_REVISION: &str = "f667ed73aee57d40cc39428eb768b4fd87a0a29e";
pub const RUNTIME_REVISION: &str = "8642eaa5cc51efbc17ad0f3e433944ba858a873f";
pub const MODEL_URL: &str = "https://huggingface.co/nvidia/Nemotron-3-Diarization/resolve/f667ed73aee57d40cc39428eb768b4fd87a0a29e/Nemotron-3-Diarization.q8_0.gguf";

pub struct ModelCache {
    directory: PathBuf,
}

impl ModelCache {
    /// Network access happens only when the caller explicitly invokes acquisition.
    pub fn download_from(
        &self,
        identity: &ModelIdentity,
        url: &str,
        cancel: &AtomicBool,
        progress: impl FnMut(u64),
    ) -> Result<PathBuf, String> {
        validate_identity(identity)?;
        cancelled(cancel)?;
        let (sender, receiver) = std::sync::mpsc::sync_channel(2);
        let url = url.to_owned();
        // HTTP is the external blocking boundary. The installer can cancel while
        // connection/body reads wait; the network worker never writes cache files.
        std::thread::spawn(move || {
            let transfer = (|| -> Result<(), String> {
                let client = reqwest::blocking::Client::builder()
                    .connect_timeout(std::time::Duration::from_secs(15))
                    .timeout(std::time::Duration::from_secs(600))
                    .build()
                    .map_err(|e| format!("Cannot initialize secure model download: {e}"))?;
                let mut response = client.get(url).send().and_then(|r| r.error_for_status())
                    .map_err(|e| format!("Model download unavailable: {e}. Check the connection and explicitly retry."))?;
                loop {
                    let mut chunk = vec![0; 64 * 1024];
                    let count = response
                        .read(&mut chunk)
                        .map_err(|e| format!("Model transfer failed: {e}"))?;
                    chunk.truncate(count);
                    if sender.send(Ok(chunk)).is_err() || count == 0 {
                        return Ok(());
                    }
                }
            })();
            if let Err(error) = transfer {
                let _ = sender.send(Err(error));
            }
        });
        let source = DownloadReader {
            receiver,
            cancel,
            chunk: std::io::Cursor::new(Vec::new()),
            ended: false,
        };
        self.install(identity, source, cancel, progress)
    }

    pub fn download_required(
        &self,
        cancel: &AtomicBool,
        progress: impl FnMut(u64),
    ) -> Result<PathBuf, String> {
        self.download_from(&NEMOTRON, MODEL_URL, cancel, progress)
    }

    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn default_location() -> Result<Self, String> {
        let local = std::env::var_os("LOCALAPPDATA")
            .ok_or("LOCALAPPDATA is unavailable; choose a local model cache")?;
        Ok(Self::new(
            PathBuf::from(local)
                .join("CleanSlate/models")
                .join(MODEL_REVISION),
        ))
    }

    /// Checks only local bytes; this never contacts the network.
    pub fn verified_path(&self, identity: &ModelIdentity) -> Result<PathBuf, String> {
        validate_identity(identity)?;
        let path = self.directory.join(identity.file_name);
        let file = File::open(&path).map_err(|e| {
            format!(
                "Required model missing/unreadable at {}: {e}. Download the model explicitly.",
                path.display()
            )
        })?;
        verify(file, identity)?;
        Ok(path)
    }

    /// Installs verified bytes from the acquisition boundary. Failed or cancelled
    /// transfers retain the last installed model. Only one writer owns the cache.
    pub fn install(
        &self,
        identity: &ModelIdentity,
        mut source: impl Read,
        cancel: &AtomicBool,
        mut progress: impl FnMut(u64),
    ) -> Result<PathBuf, String> {
        validate_identity(identity)?;
        fs::create_dir_all(&self.directory)
            .map_err(|e| format!("Cannot create model cache: {e}"))?;
        let temporary = self
            .directory
            .join(format!("{}.download", identity.file_name));
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)
            .map_err(|e| format!("Cannot start model installation: {e}. Another download may be active; remove abandoned {} after closing the editor.", temporary.display()))?;
        let guard = Temporary(temporary.clone());
        let mut total = 0u64;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            cancelled(cancel)?;
            let count = source
                .read(&mut buffer)
                .map_err(|e| format!("Model download interrupted: {e}. Retry download."))?;
            if count == 0 {
                break;
            }
            total = total
                .checked_add(count as u64)
                .ok_or("Model download size overflow")?;
            if total > identity.bytes {
                return Err(
                    "Model download exceeds pinned size; retry from the official source".into(),
                );
            }
            file.write_all(&buffer[..count])
                .map_err(|e| format!("Cannot write model cache: {e}"))?;
            hash.update(&buffer[..count]);
            progress(total);
        }
        cancelled(cancel)?;
        check_digest(total, format!("{:x}", hash.finalize()), identity)?;
        fs::write(
            self.directory.join("OpenMDW-1.1.txt"),
            include_str!("../docs/licenses/OpenMDW-1.1.txt"),
        )
        .map_err(|e| format!("Cannot retain model agreement: {e}"))?;
        fs::write(
            self.directory.join("model-origin.md"),
            include_str!("../docs/runtime-notices.md"),
        )
        .map_err(|e| format!("Cannot retain model origin notice: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("Cannot flush verified model: {e}"))?;
        drop(file);
        cancelled(cancel)?;
        let destination = self.directory.join(identity.file_name);
        atomic_replace(&temporary, &destination)?;
        drop(guard);
        Ok(destination)
    }
}

fn validate_identity(identity: &ModelIdentity) -> Result<(), String> {
    if Path::new(identity.file_name).components().count() != 1
        || identity.file_name == "."
        || identity.file_name == ".."
        || identity.file_name.contains(['/', '\\'])
        || identity.bytes == 0
        || identity.sha256.len() != 64
        || !identity.sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("Invalid pinned model identity".into());
    }
    Ok(())
}

fn check_digest(total: u64, hash: String, identity: &ModelIdentity) -> Result<(), String> {
    if total != identity.bytes || hash != identity.sha256 {
        return Err(format!(
            "Model integrity check failed (received {total} of {} bytes); expected SHA-256 {}. Retry download.",
            identity.bytes, identity.sha256
        ));
    }
    Ok(())
}

fn verify(mut file: File, identity: &ModelIdentity) -> Result<(), String> {
    let length = file
        .metadata()
        .map_err(|e| format!("Cannot inspect model: {e}"))?
        .len();
    if length != identity.bytes {
        return Err(format!(
            "Model integrity check failed: expected {} bytes, found {length}. Retry download.",
            identity.bytes
        ));
    }
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut total = 0;
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|e| format!("Cannot read model: {e}"))?;
        if n == 0 {
            break;
        }
        total += n as u64;
        hash.update(&buffer[..n]);
    }
    check_digest(total, format!("{:x}", hash.finalize()), identity)
}

fn cancelled(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Model download cancelled; installed model retained".into())
    } else {
        Ok(())
    }
}

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

struct DownloadReader<'a> {
    receiver: std::sync::mpsc::Receiver<Result<Vec<u8>, String>>,
    cancel: &'a AtomicBool,
    chunk: std::io::Cursor<Vec<u8>>,
    ended: bool,
}
impl Read for DownloadReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.cancel.load(Ordering::Relaxed) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Model download cancelled",
                ));
            }
            let count = self.chunk.read(buffer)?;
            if count > 0 || self.ended {
                return Ok(count);
            }
            match self
                .receiver
                .recv_timeout(std::time::Duration::from_millis(100))
            {
                Ok(Ok(chunk)) => {
                    self.ended = chunk.is_empty();
                    self.chunk = std::io::Cursor::new(chunk);
                }
                Ok(Err(error)) => return Err(std::io::Error::other(error)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(std::io::Error::other(
                        "Model network worker stopped before completion",
                    ));
                }
            }
        }
    }
}

#[cfg(windows)]
fn atomic_replace(source: &Path, destination: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }
    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    // Same-directory replacement with write-through retains the previous file on failure.
    if unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 1 | 8) } == 0 {
        return Err(format!(
            "Cannot atomically install model: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}
#[cfg(not(windows))]
fn atomic_replace(source: &Path, destination: &Path) -> Result<(), String> {
    fs::rename(source, destination).map_err(|e| format!("Cannot atomically install model: {e}"))
}
