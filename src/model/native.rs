//! ABI from NVIDIA/NeMo-Speech.cpp at the pinned RUNTIME_REVISION.
use super::{ModelCache, NEMOTRON, RUNTIME_REVISION};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::{CStr, CString, c_char, c_void};
use std::fs::File;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeProvenance {
    pub revision: String,
    pub recipe_sha256: String,
    pub binaries: BTreeMap<String, String>,
}

#[repr(C)]
struct ModelConfig {
    size: usize,
    model_path: *const c_char,
    gpu: i32,
    preset: *const c_char,
    chunk_frames: i32,
    right_context_frames: i32,
    left_context_frames: i32,
    fifo_frames: i32,
    spkcache_frames: i32,
    update_period_frames: i32,
}

type Create = unsafe extern "C" fn(*const ModelConfig, *mut *mut c_void) -> i32;
type Destroy = unsafe extern "C" fn(*mut c_void);
type Open = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32;
type Push = unsafe extern "C" fn(*mut c_void, *const f32, usize, i32) -> i32;
type Finish = unsafe extern "C" fn(*mut c_void) -> i32;
type Count = unsafe extern "C" fn(*const c_void) -> i64;
type Speakers = unsafe extern "C" fn(*const c_void) -> i32;
type Cadence = unsafe extern "C" fn(*const c_void) -> f64;
type Probs = unsafe extern "C" fn(*const c_void, *mut f32, usize) -> i32;
type Text = unsafe extern "C" fn() -> *const c_char;

pub struct NativeRuntime {
    _library: libloading::Library,
    provenance: RuntimeProvenance,
    create: Create,
    destroy: Destroy,
    open: Open,
    close: Destroy,
    push: Push,
    finish: Finish,
    count: Count,
    start: Count,
    speakers: Speakers,
    cadence: Cadence,
    probs: Probs,
    error: Text,
}

impl NativeRuntime {
    /// Verifies the bundled native package before loading. Never downloads code.
    pub fn open(directory: &Path) -> Result<Self, String> {
        #[cfg(target_arch = "x86_64")]
        if !std::is_x86_feature_detected!("avx2")
            || !std::is_x86_feature_detected!("fma")
            || !std::is_x86_feature_detected!("f16c")
        {
            return Err("The required Windows CPU native runtime needs AVX2, FMA and F16C; this CPU cannot run the packaged build".into());
        }
        let directory = directory.canonicalize().map_err(|e| {
            format!(
                "Required native runtime missing at {}: {e}. Reinstall the Windows CPU package.",
                directory.display()
            )
        })?;
        let manifest = File::open(directory.join("runtime-manifest.json")).map_err(|e| {
            format!("Required native runtime manifest missing: {e}. Reinstall the CPU package.")
        })?;
        let provenance: RuntimeProvenance = serde_json::from_reader(manifest)
            .map_err(|e| format!("Invalid native runtime manifest: {e}"))?;
        if provenance.revision != RUNTIME_REVISION
            || provenance.recipe_sha256.len() != 64
            || !provenance.binaries.contains_key("nemo_speech_asr_c.dll")
        {
            return Err(
                "Incompatible native runtime identity; reinstall the pinned Windows CPU package"
                    .into(),
            );
        }
        for (name, digest) in &provenance.binaries {
            if Path::new(name).components().count() != 1
                || !name.ends_with(".dll")
                || digest.len() != 64
            {
                return Err("Invalid native runtime binary manifest".into());
            }
            let mut file = File::open(directory.join(name))
                .map_err(|e| format!("Native runtime dependency {name} is missing: {e}"))?;
            let actual = {
                use sha2::{Digest, Sha256};
                let mut hash = Sha256::new();
                std::io::copy(&mut file, &mut HashWriter(&mut hash))
                    .map_err(|e| format!("Cannot verify native runtime {name}: {e}"))?;
                format!("{:x}", hash.finalize())
            };
            if &actual != digest {
                return Err(format!(
                    "Native runtime {name} failed integrity validation; reinstall the pinned package"
                ));
            }
        }
        let path = directory.join("nemo_speech_asr_c.dll");
        // The ABI and binaries are validated above; absolute loading excludes CWD DLLs.
        #[cfg(windows)]
        let library: libloading::Library = unsafe {
            libloading::os::windows::Library::load_with_flags(path,
                libloading::os::windows::LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | libloading::os::windows::LOAD_LIBRARY_SEARCH_SYSTEM32)
        }.map(Into::into).map_err(|e| format!("Cannot load required native runtime: {e}. Reinstall the complete Windows CPU package."))?;
        #[cfg(not(windows))]
        let library = unsafe { libloading::Library::new(path) }
            .map_err(|e| format!("Cannot load required native runtime: {e}"))?;
        unsafe {
            let version: Text = *library
                .get(b"nemo_speech_asr_version\0")
                .map_err(abi_error)?;
            let version = version();
            if version.is_null() || CStr::from_ptr(version).to_bytes() != b"nemo-speech-asr 0.2.0" {
                return Err(
                    "Incompatible native runtime ABI version; reinstall the pinned CPU package"
                        .into(),
                );
            }
            Ok(Self {
                create: *library
                    .get(b"nemo_speech_diar_create\0")
                    .map_err(abi_error)?,
                destroy: *library
                    .get(b"nemo_speech_diar_destroy\0")
                    .map_err(abi_error)?,
                open: *library
                    .get(b"nemo_speech_diar_stream_open\0")
                    .map_err(abi_error)?,
                close: *library
                    .get(b"nemo_speech_diar_stream_close\0")
                    .map_err(abi_error)?,
                push: *library
                    .get(b"nemo_speech_diar_stream_push_f32\0")
                    .map_err(abi_error)?,
                finish: *library
                    .get(b"nemo_speech_diar_stream_finish\0")
                    .map_err(abi_error)?,
                count: *library
                    .get(b"nemo_speech_diar_frame_count\0")
                    .map_err(abi_error)?,
                start: *library
                    .get(b"nemo_speech_diar_frame_probs_start\0")
                    .map_err(abi_error)?,
                speakers: *library
                    .get(b"nemo_speech_diar_num_speakers\0")
                    .map_err(abi_error)?,
                cadence: *library
                    .get(b"nemo_speech_diar_seconds_per_frame\0")
                    .map_err(abi_error)?,
                probs: *library
                    .get(b"nemo_speech_diar_frame_probs\0")
                    .map_err(abi_error)?,
                error: *library
                    .get(b"nemo_speech_asr_last_error\0")
                    .map_err(abi_error)?,
                _library: library,
                provenance,
            })
        }
    }

    pub fn provenance(&self) -> &RuntimeProvenance {
        &self.provenance
    }

    pub fn load_model(&self, cache: &ModelCache) -> Result<NativeModel<'_>, String> {
        let path = cache.verified_path(&NEMOTRON)?;
        // The pinned SDK accepts UTF-8 filenames and opens them through its filesystem layer.
        let path = CString::new(path.to_str().ok_or("Model path is not UTF-8")?)
            .map_err(|_| "Model path contains NUL")?;
        let config = ModelConfig {
            size: std::mem::size_of::<ModelConfig>(),
            model_path: path.as_ptr(),
            gpu: -1,
            preset: c"v3-offline".as_ptr(),
            chunk_frames: 0,
            right_context_frames: 0,
            left_context_frames: -1,
            fifo_frames: 0,
            spkcache_frames: 0,
            update_period_frames: 0,
        };
        let mut handle = std::ptr::null_mut();
        self.status(unsafe { (self.create)(&config, &mut handle) })?;
        if handle.is_null() {
            return Err("Native runtime returned no model handle".into());
        }
        let model = NativeModel {
            runtime: self,
            handle,
        };
        if model.speakers() != 8 || (model.cadence() - 0.01).abs() > 1e-9 {
            return Err(
                "Incompatible model capacity/cadence; reinstall the pinned Nemotron V3 model"
                    .into(),
            );
        }
        Ok(model)
    }

    fn status(&self, status: i32) -> Result<(), String> {
        if status == 0 {
            return Ok(());
        }
        let text = unsafe { (self.error)() };
        let detail = if text.is_null() {
            "no native error detail".into()
        } else {
            unsafe { CStr::from_ptr(text) }.to_string_lossy()
        };
        Err(format!(
            "Required native Nemotron operation failed ({status}): {detail}. No alternate detector was used."
        ))
    }
}

fn abi_error(e: libloading::Error) -> String {
    format!("Incompatible required native runtime symbol: {e}. Reinstall the pinned CPU package.")
}

struct HashWriter<'a>(&'a mut sha2::Sha256);
impl std::io::Write for HashWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        use sha2::Digest;
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct NativeModel<'a> {
    runtime: &'a NativeRuntime,
    handle: *mut c_void,
}
impl NativeModel<'_> {
    pub fn speakers(&self) -> usize {
        unsafe { (self.runtime.speakers)(self.handle) as usize }
    }
    pub fn cadence(&self) -> f64 {
        unsafe { (self.runtime.cadence)(self.handle) }
    }
    pub fn stream(&self) -> Result<NativeStream<'_>, String> {
        let mut handle = std::ptr::null_mut();
        self.runtime
            .status(unsafe { (self.runtime.open)(self.handle, &mut handle) })?;
        if handle.is_null() {
            return Err("Native runtime returned no stream handle".into());
        }
        Ok(NativeStream {
            model: self,
            handle,
            finished: false,
        })
    }
}
impl Drop for NativeModel<'_> {
    fn drop(&mut self) {
        unsafe { (self.runtime.destroy)(self.handle) };
    }
}

pub struct NativeStream<'a> {
    model: &'a NativeModel<'a>,
    handle: *mut c_void,
    finished: bool,
}
#[derive(Debug)]
pub struct ProbabilityFrames {
    pub start: u64,
    pub frames: u64,
    pub speakers: usize,
    pub seconds_per_frame: f64,
    pub probabilities: Vec<f32>,
}
impl NativeStream<'_> {
    pub fn frame_count(&self) -> Result<u64, String> {
        u64::try_from(unsafe { (self.model.runtime.count)(self.handle) })
            .map_err(|_| "Invalid native probability frame count".into())
    }
    pub fn push(&mut self, mono_16khz: &[f32]) -> Result<(), String> {
        if self.finished || mono_16khz.iter().any(|v| !v.is_finite()) {
            return Err("Invalid Nemotron input or finished stream".into());
        }
        self.model.runtime.status(unsafe {
            (self.model.runtime.push)(self.handle, mono_16khz.as_ptr(), mono_16khz.len(), 16_000)
        })
    }
    pub fn finish(&mut self) -> Result<(), String> {
        if !self.finished {
            self.model
                .runtime
                .status(unsafe { (self.model.runtime.finish)(self.handle) })?;
            self.finished = true;
        }
        Ok(())
    }
    pub fn probabilities(&self) -> Result<ProbabilityFrames, String> {
        let start = unsafe { (self.model.runtime.start)(self.handle) };
        let count = unsafe { (self.model.runtime.count)(self.handle) };
        if start < 0 || count < start {
            return Err("Invalid native probability frame range".into());
        }
        let speakers = self.model.speakers();
        let length = usize::try_from(count - start)
            .ok()
            .and_then(|n| n.checked_mul(speakers))
            .ok_or("Native probability size overflow")?;
        let mut probabilities = Vec::new();
        probabilities
            .try_reserve_exact(length)
            .map_err(|e| format!("Cannot allocate native probabilities: {e}"))?;
        probabilities.resize(length, 0.0);
        self.model.runtime.status(unsafe {
            (self.model.runtime.probs)(self.handle, probabilities.as_mut_ptr(), length)
        })?;
        if probabilities
            .iter()
            .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
        {
            return Err("Native runtime returned invalid probabilities".into());
        }
        Ok(ProbabilityFrames {
            start: start as u64,
            frames: (count - start) as u64,
            speakers,
            seconds_per_frame: self.model.cadence(),
            probabilities,
        })
    }
}
impl Drop for NativeStream<'_> {
    fn drop(&mut self) {
        unsafe { (self.model.runtime.close)(self.handle) };
    }
}
