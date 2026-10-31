//! Source-time Session capture. Callbacks only enqueue PCM and validate in-memory fingerprints.
use crate::{
    ListenMode, ProbeParams,
    session_analysis::{self, Analysis, LabelRange, Settings},
};
use crossbeam::queue::ArrayQueue;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, OnceLock, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering::SeqCst},
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MEMBERS: usize = 8;
const BLOCK: usize = 2048;
const QUEUE: usize = 128;
const MAX_SECONDS: i64 = 7200;
const MAX_RANGES: usize = 8192;
const MAX_LABELS: usize = 256;
static NEXT_KEY: AtomicU64 = AtomicU64::new(1);
static GROUPS: OnceLock<Mutex<[Weak<SessionCapture>; 8]>> = OnceLock::new();

/// Debug-only acceptance commands for group one. Normal artifacts omit this symbol.
/// All PCM, parameters, transport, state and output still cross the real format ABI.
#[cfg(cleanslate_acceptance_driver)]
#[unsafe(no_mangle)]
pub extern "C" fn cleanslate_podcast_acceptance(op: u32, start: i64, end: i64, mask: u32) -> i32 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let capture = GROUPS
            .get()
            .and_then(|groups| groups.lock()[0].upgrade())
            .ok_or(())?;
        match op {
            0 => capture.flush().map(|_| 0).map_err(|_| ()),
            1 if mask <= u8::MAX as u32 => capture
                .label(start, end, Some(mask as u8))
                .map(|_| 0)
                .map_err(|_| ()),
            2 => capture
                .analyze(Settings::default())
                .map(|_| 0)
                .map_err(|_| ()),
            3 => capture
                .view(None, None, 32)
                .map(|view| {
                    if view.ready {
                        2
                    } else if view.busy {
                        1
                    } else {
                        0
                    }
                })
                .ok_or(()),
            4 => {
                capture.cancel_analysis();
                Ok(0)
            }
            5 => capture
                .view(None, None, 32)
                .map(|view| view.labels.len() as i32)
                .ok_or(()),
            _ => Err(()),
        }
    }))
    .ok()
    .and_then(Result::ok)
    .unwrap_or(-1)
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
struct Peak {
    min: f32,
    max: f32,
    valid: bool,
}
impl Peak {
    fn merge(self, other: Self) -> Self {
        if !self.valid {
            return other;
        }
        if !other.valid {
            return self;
        }
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
            valid: true,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelEvidence {
    row: LabelRange,
    digest: [u8; 32],
}

#[derive(Clone, Copy)]
struct Packet {
    member: usize,
    owner: u64,
    start: i64,
    count: usize,
    channels: usize,
    samples: [[f32; 2]; BLOCK],
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct Ranges(BTreeMap<i64, i64>);
impl Ranges {
    fn covers(&self, start: i64, end: i64) -> bool {
        start < end
            && self
                .0
                .range(..=start)
                .next_back()
                .is_some_and(|(_, &right)| right >= end)
    }
    fn insert(&mut self, mut start: i64, mut end: i64) -> Result<(), String> {
        if start >= end {
            return Err("Invalid capture range".into());
        }
        if let Some((&a, &b)) = self.0.range(..=start).next_back()
            && b >= start
        {
            start = a;
            end = end.max(b);
            self.0.remove(&a);
        }
        loop {
            let next = self.0.range(start..).next().map(|(&a, &b)| (a, b));
            let Some((a, b)) = next else { break };
            if a > end {
                break;
            }
            end = end.max(b);
            self.0.remove(&a);
        }
        if self.0.len() >= MAX_RANGES {
            return Err(
                "Capture has too many gaps. Clear the cache and capture a continuous section."
                    .into(),
            );
        }
        self.0.insert(start, end);
        Ok(())
    }
    fn intersections(&self, other: &Self) -> Self {
        let mut out = Self::default();
        for (&a, &b) in &self.0 {
            for (&c, &d) in other.0.range(..b) {
                if a.max(c) < b.min(d) {
                    out.0.insert(a.max(c), b.min(d));
                }
            }
        }
        out
    }
    fn remove(&mut self, start: i64, end: i64) {
        let old = std::mem::take(&mut self.0);
        for (a, b) in old {
            if b <= start || a >= end {
                self.0.insert(a, b);
            } else {
                if a < start {
                    self.0.insert(a, start);
                }
                if b > end {
                    self.0.insert(end, b);
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StoredTrack {
    channels: usize,
    coverage: Ranges,
}
#[derive(Serialize, Deserialize)]
struct Manifest {
    version: u32,
    rate: u32,
    tracks: Vec<StoredTrack>,
}
struct Track {
    file: Option<File>,
    peak_file: Option<File>,
    channels: usize,
    coverage: Ranges,
    verified: Ranges,
    peaks: Vec<Peak>,
    // Eight-way summaries bound overview queries without duplicating PCM or changing disk format.
    peak_levels: Vec<Vec<Peak>>,
}
impl Track {
    fn empty() -> Self {
        Self {
            file: None,
            peak_file: None,
            channels: 0,
            coverage: Ranges::default(),
            verified: Ranges::default(),
            peaks: Vec::new(),
            peak_levels: Vec::new(),
        }
    }

    fn rebuild_peak_levels(&mut self) {
        self.peak_levels.clear();
        let mut source = &self.peaks;
        while source.len() > 1 {
            let next = source
                .chunks(8)
                .map(|chunk| chunk.iter().copied().fold(Peak::default(), Peak::merge))
                .collect();
            self.peak_levels.push(next);
            source = self.peak_levels.last().unwrap();
        }
    }

    fn set_peak(&mut self, index: usize, peak: Peak) {
        if self.peaks.len() <= index {
            self.peaks.resize(index + 1, Peak::default());
        }
        self.peaks[index] = peak;
        let mut index = index;
        let mut level = 0;
        loop {
            let source = if level == 0 {
                &self.peaks
            } else {
                &self.peak_levels[level - 1]
            };
            if source.len() <= 1 {
                break;
            }
            index /= 8;
            let aggregate = source[index * 8..(index * 8 + 8).min(source.len())]
                .iter()
                .copied()
                .fold(Peak::default(), Peak::merge);
            let count = source.len().div_ceil(8);
            if self.peak_levels.len() <= level {
                self.peak_levels.push(Vec::new());
            }
            self.peak_levels[level].resize(count, Peak::default());
            self.peak_levels[level][index] = aggregate;
            level += 1;
        }
    }

    fn peak_range(&self, mut start: usize, end: usize) -> Peak {
        let end = end.min(self.peaks.len());
        let mut peak = Peak::default();
        while start < end {
            let mut width = 1;
            let mut value = self.peaks[start];
            for level in &self.peak_levels {
                let next = width * 8;
                if !start.is_multiple_of(next) || next > end - start {
                    break;
                }
                width = next;
                value = level[start / width];
            }
            peak = peak.merge(value);
            start += width;
        }
        peak
    }
}
struct State {
    tracks: [Track; MEMBERS],
    labels: Vec<LabelRange>,
    settings: Settings,
    message: String,
    busy: bool,
    dirty: bool,
    review: Ranges,
    evidence: Vec<LabelEvidence>,
    pending_validation: Vec<LabelEvidence>,
}
enum Command {
    Analyze(Settings),
    Clear,
    Flush(std::sync::mpsc::Sender<Result<(), String>>),
}
struct Published {
    revision: u64,
    analysis: Analysis,
    hashes: Vec<Vec<[u8; 32]>>,
    member_map: [Option<usize>; MEMBERS],
}
pub struct SessionCapture {
    _lock: File,
    pub key: String,
    rate: u32,
    path: PathBuf,
    owners: [AtomicU64; MEMBERS],
    members: Mutex<[Weak<ProbeParams>; MEMBERS]>,
    binding: Mutex<()>,
    queue: ArrayQueue<Packet>,
    commands: Mutex<Vec<Command>>,
    state: Mutex<State>,
    analysis: RwLock<Option<Published>>,
    revision: AtomicU64,
    overrun: AtomicBool,
    cancel: Arc<AtomicBool>,
    user_cancel: AtomicBool,
    stopping: AtomicBool,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
    jobs: Mutex<Vec<thread::JoinHandle<()>>>,
}
pub struct WavePeak {
    pub start: i64,
    pub end: i64,
    pub min: f32,
    pub max: f32,
}
#[derive(Debug, PartialEq)]
pub enum GainDetail {
    Bounds(f32, f32),
    TooDense,
}
#[derive(Debug, PartialEq)]
pub struct GainColumn {
    pub start: i64,
    pub end: i64,
    pub detail: GainDetail,
}
#[derive(Debug)]
pub struct TrackGain {
    pub member: usize,
    pub columns: Vec<GainColumn>,
}
#[derive(Debug)]
pub struct AnalysisGainView {
    pub revision: u64,
    pub tracks: Vec<TrackGain>,
}
pub struct TrackView {
    pub member: usize,
    pub present: bool,
    pub verified: bool,
    pub peaks: Vec<WavePeak>,
}
pub struct SessionView {
    pub rate: u32,
    pub start: i64,
    pub end: i64,
    pub captured_end: i64,
    pub tracks: Vec<TrackView>,
    pub labels: Vec<LabelRange>,
    pub message: String,
    pub busy: bool,
    pub ready: bool,
    pub settings: Settings,
}

fn valid_key(key: &str) -> bool {
    key.len() == 32 && key.bytes().all(|b| b.is_ascii_hexdigit())
}
fn cache_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from))
        .unwrap_or_else(std::env::temp_dir)
        .join("CleanSlate")
        .join(if cfg!(cleanslate_podcast) {
            "Podcast/Sessions/v1"
        } else {
            "PluginSessions"
        })
}
fn new_key() -> String {
    let mut digest = Sha256::new();
    digest.update(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .to_le_bytes(),
    );
    digest.update(std::process::id().to_le_bytes());
    digest.update(NEXT_KEY.fetch_add(1, SeqCst).to_le_bytes());
    format!("{:x}", digest.finalize())[..32].to_owned()
}

impl SessionCapture {
    pub fn sample_rate(&self) -> u32 {
        self.rate
    }
    pub fn bind(
        group: i32,
        member: usize,
        owner: u64,
        params: &Arc<ProbeParams>,
        rate: u32,
    ) -> Result<Option<CaptureHandle>, String> {
        if group == 0 {
            return Ok(None);
        }
        if !(1..=8).contains(&group)
            || member >= MEMBERS
            || owner == 0
            || !(8000..=192000).contains(&rate)
        {
            return Err("Choose a valid Session group and microphone.".into());
        }
        let groups = GROUPS.get_or_init(|| Mutex::new(std::array::from_fn(|_| Weak::new())));
        let mut groups = groups.lock();
        let saved = params.session_key.lock().clone();
        let incoming_labels = params.session_labels.lock().clone();
        let incoming_settings = *params.session_settings.lock();
        let incoming_evidence = params.session_label_evidence.lock().clone();
        if !saved.is_empty() && !valid_key(&saved) {
            return Err("Saved Session cache identity is invalid.".into());
        }
        let relocated = groups.iter().enumerate().find_map(|(index, entry)| {
            let existing = entry.upgrade()?;
            (index != group as usize - 1 && existing.key == saved && existing.rate == rate)
                .then_some((index, existing))
        });
        if let Some((index, existing)) = relocated {
            if !existing.active_members().is_empty() {
                return Err(format!(
                    "This saved Session is already open in group {}. Use that group.",
                    index + 1
                ));
            }
            if groups[group as usize - 1]
                .upgrade()
                .is_some_and(|other| !other.active_members().is_empty())
            {
                return Err("The destination group already contains another Session.".into());
            }
            groups[index] = Weak::new();
            groups[group as usize - 1] = Arc::downgrade(&existing);
        }
        let engine = if let Some(existing) = groups[group as usize - 1]
            .upgrade()
            .filter(|e| e.rate == rate)
        {
            if !saved.is_empty() && saved != existing.key {
                return Err("This numeric group belongs to a different saved Session. Choose another group.".into());
            }
            existing
        } else {
            let key = if saved.is_empty() {
                new_key()
            } else {
                saved.clone()
            };
            let engine = Self::create(
                key,
                rate,
                cache_root(),
                incoming_labels.clone(),
                incoming_settings,
                incoming_evidence.clone(),
            )?;
            groups[group as usize - 1] = Arc::downgrade(&engine);
            engine
        };
        let lifecycle = engine.binding.lock();
        let mismatch = {
            let state = engine.state.lock();
            incoming_labels != state.labels
                || incoming_settings != state.settings
                || incoming_evidence != state.evidence
        };
        if !saved.is_empty() && mismatch {
            engine.invalidate();
            return Err("Saved Session state differs between instances. Reopen matching project states before processing.".into());
        }
        let prior = engine.owners[member].load(SeqCst);
        if engine.members.lock()[member].upgrade().is_none() {
            engine.owners[member].store(0, SeqCst);
        }
        let prior = if engine.owners[member].load(SeqCst) == 0 {
            0
        } else {
            prior
        };
        if prior != 0 && prior != owner {
            return Err(
                "That microphone is already present. Choose a unique microphone number.".into(),
            );
        }
        if prior == 0 {
            engine.invalidate();
            let mut state = engine.state.lock();
            for row in state.labels.clone() {
                state.review.insert(row.start, row.end)?;
            }
            state.pending_validation = state.evidence.clone();
        }
        engine.owners[member].store(owner, SeqCst);
        engine.members.lock()[member] = Arc::downgrade(params);
        engine.start_worker()?;
        *params.session_key.lock() = engine.key.clone();
        {
            let state = engine.state.lock();
            *params.session_labels.lock() = state.labels.clone();
            *params.session_settings.lock() = state.settings;
            *params.session_label_evidence.lock() = state.evidence.clone();
        }
        drop(lifecycle);
        Ok(Some(CaptureHandle::new(engine, member, owner)))
    }
    fn create(
        key: String,
        rate: u32,
        root: PathBuf,
        labels: Vec<LabelRange>,
        settings: Settings,
        evidence: Vec<LabelEvidence>,
    ) -> Result<Arc<Self>, String> {
        if !valid_key(&key)
            || labels.len() > MAX_LABELS
            || evidence.len() > MAX_LABELS
            || evidence.iter().any(|e| !labels.contains(&e.row))
        {
            return Err("Invalid saved Session state".into());
        }
        let path = root.join(&key).join(rate.to_string());
        fs::create_dir_all(&path).map_err(|e| format!("Cannot create Session cache: {e}"))?;
        let cache_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.join("owner.lock"))
            .map_err(|e| e.to_string())?;
        cache_lock.try_lock().map_err(|_|"This Session cache is open in another process. Close the other project before using it here.".to_owned())?;
        let mut tracks = std::array::from_fn(|_| Track::empty());
        let manifest = path.join("capture.json");
        if manifest.exists() {
            let length = fs::metadata(&manifest).map_err(|e| e.to_string())?.len();
            if length > 4 * 1024 * 1024 {
                return Err("Session cache manifest is oversized. Clear this cache.".into());
            }
            let saved: Manifest =
                serde_json::from_reader(File::open(&manifest).map_err(|e| e.to_string())?)
                    .map_err(|e| format!("Invalid Session cache: {e}"))?;
            if saved.version != 1 || saved.rate != rate || saved.tracks.len() != MEMBERS {
                return Err("Incompatible Session cache".into());
            }
            for (i, stored) in saved.tracks.into_iter().enumerate() {
                if stored.channels > 2
                    || stored.coverage.0.len() > MAX_RANGES
                    || stored
                        .coverage
                        .0
                        .iter()
                        .any(|(&a, &b)| a < 0 || a >= b || b > i64::from(rate) * MAX_SECONDS)
                {
                    return Err("Invalid cached source coverage".into());
                }
                tracks[i].channels = stored.channels;
                // Saved coverage is a display hint. This host run must recapture before analysis.
                tracks[i].coverage = stored.coverage;
                let peaks = path.join(format!("mic-{i}.peaks"));
                if peaks.exists()
                    && fs::metadata(&peaks).map_err(|e| e.to_string())?.len() < 32 * 1024 * 1024
                {
                    let bytes = fs::read(peaks).map_err(|e| e.to_string())?;
                    if !bytes.len().is_multiple_of(12) {
                        return Err("Invalid waveform cache".into());
                    }
                    let loaded = bytes
                        .as_chunks::<12>()
                        .0
                        .iter()
                        .map(|b| Peak {
                            min: f32::from_le_bytes(b[0..4].try_into().unwrap()),
                            max: f32::from_le_bytes(b[4..8].try_into().unwrap()),
                            valid: u32::from_le_bytes(b[8..12].try_into().unwrap()) == 1,
                        })
                        .collect::<Vec<_>>();
                    if loaded.len() <= MAX_SECONDS as usize * 32
                        && loaded
                            .iter()
                            .all(|p| p.min.is_finite() && p.max.is_finite() && p.min <= p.max)
                    {
                        tracks[i].peaks = loaded;
                        tracks[i].rebuild_peak_levels();
                    }
                }
            }
        }
        let mut review = Ranges::default();
        for row in &labels {
            review.insert(row.start, row.end)?;
        }
        let engine = Arc::new(Self {
            _lock: cache_lock,
            key,
            rate,
            path,
            owners: std::array::from_fn(|_| AtomicU64::new(0)),
            members: Mutex::new(std::array::from_fn(|_| Weak::new())),
            binding: Mutex::new(()),
            queue: ArrayQueue::new(QUEUE),
            commands: Mutex::new(Vec::new()),
            state: Mutex::new(State {
                tracks,
                labels,
                settings,
                message:
                    "Play the DAW to capture. Cached audio must be verified again after reopening."
                        .into(),
                busy: false,
                dirty: false,
                review,
                pending_validation: evidence.clone(),
                evidence,
            }),
            analysis: RwLock::new(None),
            revision: AtomicU64::new(1),
            overrun: AtomicBool::new(false),
            cancel: Arc::new(AtomicBool::new(false)),
            user_cancel: AtomicBool::new(false),
            stopping: AtomicBool::new(true),
            worker: Mutex::new(None),
            jobs: Mutex::new(Vec::new()),
        });
        engine.start_worker()?;
        Ok(engine)
    }
    fn start_worker(self: &Arc<Self>) -> Result<(), String> {
        let mut worker = self.worker.lock();
        if worker.is_some() {
            return Ok(());
        }
        self.stopping.store(false, SeqCst);
        let weak = Arc::downgrade(self);
        *worker = Some(
            thread::Builder::new()
                .name("CleanSlate Session capture".into())
                .spawn(move || Self::worker(weak))
                .map_err(|e| e.to_string())?,
        );
        Ok(())
    }
    fn shutdown(&self) {
        self.cancel.store(true, SeqCst);
        self.stopping.store(true, SeqCst);
        if let Some(worker) = self.worker.lock().take() {
            let _ = worker.join();
        }
        for job in self.jobs.lock().drain(..) {
            let _ = job.join();
        }
    }
    fn invalidate(&self) {
        self.revision.fetch_add(1, SeqCst);
        self.cancel.store(true, SeqCst);
    }
    fn persist_members(&self, state: &State) {
        for params in self.members.lock().iter().filter_map(Weak::upgrade) {
            *params.session_labels.lock() = state.labels.clone();
            *params.session_settings.lock() = state.settings;
            *params.session_label_evidence.lock() = state.evidence.clone();
        }
    }
    pub fn label(&self, start: i64, end: i64, mask: Option<u8>) -> Result<(), String> {
        self.edit_label(start, end, mask, false)
    }
    pub fn replace_label(&self, start: i64, end: i64, mask: u8) -> Result<(), String> {
        self.edit_label(start, end, Some(mask), true)
    }
    fn edit_label(
        &self,
        start: i64,
        end: i64,
        mask: Option<u8>,
        replace: bool,
    ) -> Result<(), String> {
        let mut state = self.state.lock();
        if state.busy {
            return Err("Wait for analysis to finish.".into());
        }
        let active = self.active_members();
        if start < 0
            || start >= end
            || active.is_empty()
            || active
                .iter()
                .any(|&i| !state.tracks[i].verified.covers(start, end))
        {
            return Err(
                "Capture every microphone across this selection first; gaps cannot be labeled."
                    .into(),
            );
        }
        let allowed = active.iter().fold(0_u8, |bits, &i| bits | (1 << i));
        if mask.is_some_and(|bits| bits & !allowed != 0) {
            return Err("The label includes an absent microphone.".into());
        }
        if !replace
            && mask.is_some_and(|mask| {
                state
                    .labels
                    .iter()
                    .any(|row| row.start < end && row.end > start && row.mask != mask)
            })
        {
            return Err("This selection conflicts with saved examples. Confirm replacement or choose another range.".into());
        }
        let mut labels = Vec::new();
        for row in &state.labels {
            if row.end <= start || row.start >= end {
                labels.push(*row)
            } else {
                if row.start < start {
                    labels.push(LabelRange { end: start, ..*row });
                }
                if row.end > end {
                    labels.push(LabelRange { start: end, ..*row });
                }
            }
        }
        if let Some(mask) = mask {
            labels.push(LabelRange { start, end, mask });
        }
        labels.sort_by_key(|row| row.start);
        if labels.len() > MAX_LABELS {
            return Err("At most 256 example ranges are supported.".into());
        }
        let mut evidence = Vec::new();
        for &row in &labels {
            if let Some(old) = state
                .evidence
                .iter()
                .find(|e| e.row == row && (row.end <= start || row.start >= end))
            {
                evidence.push(old.clone());
            } else {
                let digest = label_digest(&mut state, &active, row, self.rate)?;
                evidence.push(LabelEvidence { row, digest });
            }
        }
        state.evidence = evidence;
        state
            .pending_validation
            .retain(|e| e.row.end <= start || e.row.start >= end);
        state.labels = labels;
        state.review.remove(start, end);
        state.dirty = true;
        state.message = "Examples changed. Analyze to regenerate processing.".into();
        self.invalidate();
        self.persist_members(&state);
        Ok(())
    }
    fn active_members(&self) -> Vec<usize> {
        let members = self.members.lock();
        (0..MEMBERS)
            .filter(|&i| self.owners[i].load(SeqCst) != 0 && members[i].strong_count() > 0)
            .collect()
    }
    pub fn analyze(&self, settings: Settings) -> Result<(), String> {
        let mut state = self.state.lock();
        if state.settings != settings {
            return Err("Session sound changed in another instance. Use its current settings and Analyze again.".into());
        }
        if state.busy {
            return Err("Analysis is already running.".into());
        }
        if state.labels.is_empty() {
            return Err("Label room tone and solo speech for each microphone first.".into());
        }
        if !state.review.0.is_empty() {
            return Err("Audio changed inside saved examples. Relabel or erase the affected example ranges before Analyze.".into());
        }
        state.busy = true;
        state.settings = settings;
        state.message = "Preparing captured audio for analysis…".into();
        self.invalidate();
        self.user_cancel.store(false, SeqCst);
        self.cancel.store(false, SeqCst);
        self.persist_members(&state);
        self.commands.lock().push(Command::Analyze(settings));
        Ok(())
    }
    pub fn cancel_analysis(&self) {
        self.user_cancel.store(true, SeqCst);
        self.invalidate();
        if let Some(mut state) = self.state.try_lock()
            && state.busy
        {
            state.message = "Cancelling analysis; original audio is preserved…".into();
        }
    }
    pub fn set_settings(&self, settings: Settings) -> Result<(), String> {
        if !settings.reduction_db.is_finite()
            || !(-48.0..=0.0).contains(&settings.reduction_db)
            || !settings.attack_ms.is_finite()
            || !(5.0..=80.0).contains(&settings.attack_ms)
            || !settings.release_ms.is_finite()
            || !(10.0..=150.0).contains(&settings.release_ms)
        {
            return Err("Invalid Session sound settings".into());
        }
        let mut state = self.state.lock();
        if state.settings != settings {
            state.settings = settings;
            self.invalidate();
            self.persist_members(&state);
            state.message = "Sound settings changed. Analyze again to update the envelopes.".into();
        }
        Ok(())
    }
    fn flush(&self) -> Result<(), String> {
        let (send, receive) = std::sync::mpsc::channel();
        self.commands.lock().push(Command::Flush(send));
        receive.recv_timeout(Duration::from_secs(5)).map_err(|_| {
            "Capture is still writing; replay the tail before relying on a reopened cache."
                .to_owned()
        })?
    }
    pub fn clear(&self) -> Result<(), String> {
        self.invalidate();
        self.commands.lock().push(Command::Clear);
        Ok(())
    }
    pub fn analysis_view(
        &self,
        _start: i64,
        _end: i64,
        _columns: usize,
        _member_mask: u8,
    ) -> Option<AnalysisGainView> {
        None
    }
    pub fn analysis_view_is_current(&self, _view: &AnalysisGainView) -> bool {
        false
    }
    pub fn view(
        &self,
        start: Option<i64>,
        end: Option<i64>,
        columns: usize,
    ) -> Option<SessionView> {
        let state = self.state.try_lock()?;
        let width = (i64::from(self.rate) * 32 / 1000).max(1);
        let extent = state
            .tracks
            .iter()
            .filter_map(|t| t.coverage.0.last_key_value().map(|(_, b)| *b))
            .max()
            .unwrap_or(i64::from(self.rate) * 10);
        let total = i64::from(self.rate) * MAX_SECONDS;
        let requested_left = start.unwrap_or(0);
        let span = end
            .unwrap_or(extent)
            .saturating_sub(requested_left)
            .clamp(1, total);
        let left = requested_left.clamp(0, total - span);
        let right = left + span;
        let columns = columns.clamp(1, 2048);
        let mut tracks = Vec::new();
        for (member, track) in state.tracks.iter().enumerate() {
            let present = self.owners[member].load(SeqCst) != 0;
            if !present && track.coverage.0.is_empty() {
                continue;
            }
            let mut peaks = Vec::new();
            for column in 0..columns {
                let a = left + (right - left) * column as i64 / columns as i64;
                let b = left + (right - left) * (column + 1) as i64 / columns as i64;
                let peak =
                    track.peak_range((a / width) as usize, ((b + width - 1) / width) as usize);
                if peak.valid {
                    peaks.push(WavePeak {
                        start: a,
                        end: b.max(a + 1),
                        min: peak.min,
                        max: peak.max,
                    });
                }
            }
            tracks.push(TrackView {
                member,
                present,
                verified: track.verified.covers(left, right),
                peaks,
            });
        }
        let analyzed = self.analysis.try_read().and_then(|guard| {
            guard
                .as_ref()
                .filter(|p| p.revision == self.revision.load(SeqCst))
                .map(|p| (p.analysis.origin, p.analysis.end))
        });
        let message = if let Some((start, end)) = analyzed {
            format!(
                "{} Analyzed {:.2}–{:.2} s; outside this interval remains original.",
                state.message,
                start as f64 / f64::from(self.rate),
                end as f64 / f64::from(self.rate)
            )
        } else {
            state.message.clone()
        };
        Some(SessionView {
            rate: self.rate,
            start: left,
            end: right,
            captured_end: extent,
            tracks,
            labels: state.labels.clone(),
            message,
            busy: state.busy,
            ready: self.analysis.try_read().is_some_and(|a| {
                a.as_ref()
                    .is_some_and(|a| a.revision == self.revision.load(SeqCst))
            }),
            settings: state.settings,
        })
    }
    fn worker(weak: Weak<Self>) {
        let mut ticks = 0_u32;
        while let Some(engine) = weak.upgrade() {
            if engine.stopping.load(SeqCst) {
                break;
            }
            if engine.overrun.swap(false, SeqCst) {
                // None of the queued pre-fault packets can establish freshness after input loss.
                for _ in 0..QUEUE {
                    if engine.queue.pop().is_none() {
                        break;
                    }
                }
                let mut state = engine.state.lock();
                for track in &mut state.tracks {
                    track.verified = Ranges::default();
                    // Retain old display/cache coverage as unverified; never invent replacement audio.
                }
                state.message="Capture could not keep up or received invalid audio. Stop, then recapture the section.".into();
                state.dirty = true;
            }
            for _ in 0..QUEUE {
                let Some(packet) = engine.queue.pop() else {
                    break;
                };
                if let Err(error) = engine.write_packet(packet) {
                    engine.invalidate();
                    engine.state.lock().message = format!("Capture stopped: {error}");
                    engine.overrun.store(true, SeqCst);
                    break;
                }
            }
            let commands = std::mem::take(&mut *engine.commands.lock());
            for command in commands {
                match command {
                    Command::Flush(reply) => {
                        let _ = reply.send(engine.save());
                    }
                    Command::Clear => {
                        let mut state = engine.state.lock();
                        for (i, track) in state.tracks.iter_mut().enumerate() {
                            *track = Track::empty();
                            let _ = fs::remove_file(engine.path.join(format!("mic-{i}.pcm")));
                            let _ = fs::remove_file(engine.path.join(format!("mic-{i}.peaks")));
                        }
                        state.labels.clear();
                        state.evidence.clear();
                        state.pending_validation.clear();
                        state.review = Ranges::default();
                        state.dirty = true;
                        state.message = "Cache cleared. Play to capture a new Session.".into();
                        engine.persist_members(&state);
                        *engine.analysis.write() = None;
                    }
                    Command::Analyze(settings) => {
                        if let Err(e) = engine.start_analysis(settings) {
                            let mut state = engine.state.lock();
                            state.busy = false;
                            state.message = if engine.user_cancel.load(SeqCst) {
                                "Analysis cancelled. Examples and captured audio are retained; Analyze when ready.".into()
                            } else if engine.cancel.load(SeqCst) {
                                "Captured audio, examples or sound changed. Analyze again.".into()
                            } else {
                                e
                            };
                        }
                    }
                }
            }
            ticks += 1;
            if ticks.is_multiple_of(200)
                && let Err(error) = engine.save()
            {
                engine.invalidate();
                engine.state.lock().message = format!("Cache save failed: {error}");
            }
            drop(engine);
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn write_packet(&self, packet: Packet) -> Result<(), String> {
        if self.owners[packet.member].load(SeqCst) != packet.owner {
            return Ok(());
        }
        let mut state = self.state.lock();
        let track = &mut state.tracks[packet.member];
        if track.channels != 0 && track.channels != packet.channels {
            track.file = None;
            track.peak_file = None;
            track.coverage = Ranges::default();
            track.verified = Ranges::default();
            track.peaks.clear();
            track.peak_levels.clear();
            self.invalidate();
            File::create(self.path.join(format!("mic-{}.pcm", packet.member)))
                .map_err(|e| e.to_string())?;
            track.peak_file = Some(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(self.path.join(format!("mic-{}.peaks", packet.member)))
                    .map_err(|e| e.to_string())?,
            );
        }
        track.channels = packet.channels;
        if track.file.is_none() {
            track.file = Some(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(self.path.join(format!("mic-{}.pcm", packet.member)))
                    .map_err(|e| e.to_string())?,
            );
        }
        let file = track.file.as_mut().unwrap();
        let end = packet.start + packet.count as i64;
        let already_captured = track.coverage.covers(packet.start, end);
        let stride = packet.channels * 4;
        let mut bytes = [0_u8; BLOCK * 8];
        for (n, sample) in packet.samples[..packet.count].iter().enumerate() {
            for (c, value) in sample[..packet.channels].iter().enumerate() {
                bytes[n * stride + c * 4..n * stride + c * 4 + 4]
                    .copy_from_slice(&value.to_le_bytes());
            }
        }
        let mut changed = false;
        for (&a, &b) in track.coverage.0.range(..end) {
            let a = a.max(packet.start);
            let b = b.min(end);
            if a >= b {
                continue;
            }
            let offset = ((a - packet.start) as usize) * stride;
            let count = ((b - a) as usize) * stride;
            let mut old = [0_u8; BLOCK * 8];
            file.seek(SeekFrom::Start(a as u64 * stride as u64))
                .map_err(|e| e.to_string())?;
            if file.read_exact(&mut old[..count]).is_err()
                || old[..count] != bytes[offset..offset + count]
            {
                changed = true;
                break;
            }
        }
        if changed {
            self.invalidate();
        }
        let width = (i64::from(self.rate) * 32 / 1000).max(1);
        let have_peaks = (packet.start / width..=(end - 1) / width).all(|i| {
            !track.coverage.covers(i * width, (i + 1) * width)
                || track.peaks.get(i as usize).is_some_and(|p| p.valid)
        });
        if already_captured && !changed && have_peaks {
            track.verified.insert(packet.start, end)?;
            self.validate_saved_examples(&mut state)?;
            return Ok(());
        }
        if !already_captured || changed {
            file.seek(SeekFrom::Start(packet.start as u64 * stride as u64))
                .map_err(|e| e.to_string())?;
            file.write_all(&bytes[..packet.count * stride])
                .map_err(|e| e.to_string())?;
        }
        track.coverage.insert(packet.start, end)?;
        track.verified.insert(packet.start, end)?;
        let width = (i64::from(self.rate) * 32 / 1000).max(1);
        for index in packet.start / width..=(end - 1) / width {
            let left = index * width;
            let right = left + width;
            if !track.coverage.covers(left, right) {
                continue;
            }
            let mut data = vec![0_u8; (width as usize) * stride];
            let file = track.file.as_mut().unwrap();
            file.seek(SeekFrom::Start(left as u64 * stride as u64))
                .map_err(|e| e.to_string())?;
            file.read_exact(&mut data).map_err(|e| e.to_string())?;
            let mut peak = Peak {
                min: f32::INFINITY,
                max: f32::NEG_INFINITY,
                valid: true,
            };
            for value in data.as_chunks::<4>().0 {
                let value = f32::from_le_bytes(*value);
                peak.min = peak.min.min(value);
                peak.max = peak.max.max(value);
            }
            track.set_peak(index as usize, peak);
            if track.peak_file.is_none() {
                track.peak_file = Some(
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(self.path.join(format!("mic-{}.peaks", packet.member)))
                        .map_err(|e| e.to_string())?,
                );
            }
            let peak_file = track.peak_file.as_mut().unwrap();
            peak_file
                .seek(SeekFrom::Start(index as u64 * 12))
                .map_err(|e| e.to_string())?;
            peak_file
                .write_all(&peak.min.to_le_bytes())
                .and_then(|_| peak_file.write_all(&peak.max.to_le_bytes()))
                .and_then(|_| peak_file.write_all(&1_u32.to_le_bytes()))
                .map_err(|e| e.to_string())?;
        }
        state.dirty = true;
        if changed {
            let affected = state
                .labels
                .iter()
                .filter(|l| l.start < end && l.end > packet.start)
                .copied()
                .collect::<Vec<_>>();
            for label in affected {
                state.review.insert(label.start, label.end)?;
            }
            state.message="Audio changed. Relabel affected examples, then Analyze again; original audio is preserved.".into();
        } else if !state.busy && self.analysis.read().is_none() {
            state.message="Capturing at DAW source time. Label room tone and each microphone’s solo speech, then Analyze.".into();
        }
        self.validate_saved_examples(&mut state)?;
        Ok(())
    }
    fn validate_saved_examples(&self, state: &mut State) -> Result<(), String> {
        let active = self.active_members();
        let pending = std::mem::take(&mut state.pending_validation);
        for evidence in pending {
            if active.is_empty()
                || active.iter().any(|&i| {
                    !state.tracks[i]
                        .verified
                        .covers(evidence.row.start, evidence.row.end)
                })
            {
                state.pending_validation.push(evidence);
                continue;
            }
            if label_digest(state, &active, evidence.row, self.rate)? == evidence.digest {
                state.review.remove(evidence.row.start, evidence.row.end);
            } else {
                state.message="Source audio differs from saved examples. Relabel affected examples before Analyze.".into();
            }
        }
        Ok(())
    }
    fn save(&self) -> Result<(), String> {
        let mut state = self.state.lock();
        if !state.dirty {
            return Ok(());
        }
        for track in &mut state.tracks {
            if let Some(file) = &mut track.file {
                file.sync_data().map_err(|e| e.to_string())?;
            }
            if let Some(file) = &mut track.peak_file {
                file.sync_data().map_err(|e| e.to_string())?;
            }
        }
        let manifest = Manifest {
            version: 1,
            rate: self.rate,
            tracks: state
                .tracks
                .iter()
                .map(|t| StoredTrack {
                    channels: t.channels,
                    coverage: t.coverage.clone(),
                })
                .collect(),
        };
        let temp = self.path.join("capture.tmp");
        serde_json::to_writer(File::create(&temp).map_err(|e| e.to_string())?, &manifest)
            .map_err(|e| e.to_string())?;
        fs::rename(temp, self.path.join("capture.json")).map_err(|e| e.to_string())?;
        state.dirty = false;
        Ok(())
    }
    fn start_analysis(self: &Arc<Self>, settings: Settings) -> Result<(), String> {
        let revision = self.revision.load(SeqCst);
        let active = self.active_members();
        if active.is_empty() {
            return Err("No microphones are present.".into());
        }
        let mut state = self.state.lock();
        if self.cancel.load(SeqCst) {
            return Err(if self.user_cancel.load(SeqCst) {
                "Analysis cancelled. Examples and captured audio are retained; Analyze when ready."
            } else {
                "Captured audio, examples or sound changed. Analyze again."
            }
            .into());
        }
        let mut common = state.tracks[active[0]].verified.clone();
        let allowed = active.iter().fold(0_u8, |mask, &i| mask | (1 << i));
        if state.labels.iter().any(|row| row.mask & !allowed != 0) {
            return Err("Saved examples refer to an absent microphone. Restore that member or review those examples.".into());
        }
        for &i in &active[1..] {
            common = common.intersections(&state.tracks[i].verified);
        }
        let (a,b)=common.0.iter().filter_map(|(&a,&b)|{
            (a<b && state.labels.iter().all(|l|l.start>=a&&l.end<=b)).then_some((a,b))
        }).max_by_key(|(a,b)|b-a).ok_or("Capture a continuous section on every microphone containing all examples. Fill the gaps before Analyze.")?;
        let job = self.path.join(format!("analysis-{}", new_key()));
        fs::create_dir(&job).map_err(|e| e.to_string())?;
        let prepared = (|| {
            let mut paths = Vec::new();
            let mut hashes = Vec::new();
            for &i in &active {
                let track = &mut state.tracks[i];
                let file = track
                    .file
                    .as_mut()
                    .ok_or("Play every microphone to verify the cached audio first.")?;
                let path = job.join(format!("mic-{i}.wav"));
                let (output, hash) = snapshot_wav(
                    file,
                    &path,
                    a,
                    b,
                    self.rate,
                    track.channels,
                    Some(&self.cancel),
                )?;
                paths.push(output);
                hashes.push(hash);
            }
            // Desktop speaker IDs are dense; retain the mapping back to sparse plugin slots.
            let labels = state
                .labels
                .iter()
                .map(|row| LabelRange {
                    start: row.start,
                    end: row.end,
                    mask: active.iter().enumerate().fold(0_u8, |mask, (n, &i)| {
                        mask | if row.mask & (1 << i) != 0 { 1 << n } else { 0 }
                    }),
                })
                .collect::<Vec<_>>();
            Ok::<_, String>((paths, hashes, labels))
        })();
        drop(state);
        let (paths, hashes, labels) = match prepared {
            Ok(p) => p,
            Err(e) => {
                let _ = fs::remove_dir_all(&job);
                return Err(e);
            }
        };
        let engine = Arc::downgrade(self);
        let cancel = self.cancel.clone();
        let rate = self.rate;
        let job_thread=thread::Builder::new().name("CleanSlate Session analysis".into()).spawn(move||{
            let result=session_analysis::analyze(&paths,&labels,a,rate,&job,&cancel,settings);
            if let Some(engine)=engine.upgrade() {
                let mut state=engine.state.lock();state.busy=false;
                if engine.user_cancel.load(SeqCst) {state.message="Analysis cancelled. Examples and captured audio are retained; Analyze when ready.".into();}
                else if engine.revision.load(SeqCst)!=revision {state.message="Captured audio, examples or sound changed during analysis. Analyze again.".into();}
                else {match result {
                    Ok(analysis)=>{
                        let mut full_hashes=vec![Vec::new();MEMBERS];
                        let mut member_map=[None;MEMBERS];
                        for ((n,&slot),hash) in active.iter().enumerate().zip(hashes) {member_map[slot]=Some(n);full_hashes[slot]=hash;}
                        *engine.analysis.write()=Some(Published{revision,analysis,hashes:full_hashes,member_map});
                        state.message="Analysis ready. Enable processing and replay to validate the source audio. Unverified sections stay original.".into();
                    }
                    Err(e)=>state.message=format!("Analysis stopped: {e}"),
                }}
            }
            // This fresh UUID-named directory is owned by this job, never a caller source path.
            let _=fs::remove_dir_all(job);
        }).map_err(|e|e.to_string())?;
        let mut jobs = self.jobs.lock();
        let mut retained = Vec::new();
        for job in jobs.drain(..) {
            if job.is_finished() {
                let _ = job.join();
            } else {
                retained.push(job);
            }
        }
        retained.push(job_thread);
        *jobs = retained;
        Ok(())
    }
}

fn label_digest(
    state: &mut State,
    active: &[usize],
    row: LabelRange,
    rate: u32,
) -> Result<[u8; 32], String> {
    let mut digest = Sha256::new();
    digest.update(rate.to_le_bytes());
    digest.update(row.start.to_le_bytes());
    digest.update(row.end.to_le_bytes());
    let mut bytes = [0_u8; 8192];
    for &member in active {
        let track = &mut state.tracks[member];
        if !track.verified.covers(row.start, row.end) {
            return Err("Replay all microphones across the saved example first.".into());
        }
        let file = track.file.as_mut().ok_or("Missing captured source")?;
        digest.update((member as u64).to_le_bytes());
        digest.update((track.channels as u64).to_le_bytes());
        file.seek(SeekFrom::Start(
            row.start as u64 * track.channels as u64 * 4,
        ))
        .map_err(|e| e.to_string())?;
        let mut remaining = (row.end - row.start) as u64 * track.channels as u64 * 4;
        while remaining > 0 {
            let count = remaining.min(bytes.len() as u64) as usize;
            file.read_exact(&mut bytes[..count])
                .map_err(|e| e.to_string())?;
            digest.update(&bytes[..count]);
            remaining -= count as u64;
        }
    }
    Ok(digest.finalize().into())
}

fn snapshot_wav(
    file: &mut File,
    path: &Path,
    start: i64,
    end: i64,
    rate: u32,
    channels: usize,
    cancel: Option<&AtomicBool>,
) -> Result<(PathBuf, Vec<[u8; 32]>), String> {
    if cancel.is_some_and(|cancel| cancel.load(SeqCst)) {
        return Err("Session snapshot cancelled.".into());
    }
    let length = (end - start)
        .checked_mul(channels as i64 * 4)
        .and_then(|v| u32::try_from(v).ok())
        .filter(|&v| v < u32::MAX - 36)
        .ok_or("Capture section exceeds WAV size limit. Analyze a shorter section.")?;
    let mut output = File::create(path).map_err(|e| e.to_string())?;
    output
        .write_all(b"RIFF")
        .and_then(|_| output.write_all(&(length + 36).to_le_bytes()))
        .and_then(|_| output.write_all(b"WAVEfmt "))
        .and_then(|_| output.write_all(&16_u32.to_le_bytes()))
        .and_then(|_| output.write_all(&3_u16.to_le_bytes()))
        .and_then(|_| output.write_all(&(channels as u16).to_le_bytes()))
        .and_then(|_| output.write_all(&rate.to_le_bytes()))
        .and_then(|_| output.write_all(&(rate * channels as u32 * 4).to_le_bytes()))
        .and_then(|_| output.write_all(&(channels as u16 * 4).to_le_bytes()))
        .and_then(|_| output.write_all(&32_u16.to_le_bytes()))
        .and_then(|_| output.write_all(b"data"))
        .and_then(|_| output.write_all(&length.to_le_bytes()))
        .map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(start as u64 * channels as u64 * 4))
        .map_err(|e| e.to_string())?;
    let mut hashes = Vec::new();
    let mut bytes = [0_u8; BLOCK * 8];
    let mut cursor = start;
    while cursor < end {
        if cancel.is_some_and(|cancel| cancel.load(SeqCst)) {
            return Err("Session snapshot cancelled.".into());
        }
        let count = (BLOCK as i64 - cursor.rem_euclid(BLOCK as i64)).min(end - cursor) as usize;
        let bytes = &mut bytes[..count * channels * 4];
        file.read_exact(bytes).map_err(|e| e.to_string())?;
        output.write_all(bytes).map_err(|e| e.to_string())?;
        if count == BLOCK {
            hashes.push(Sha256::digest(bytes).into());
        }
        cursor += count as i64;
    }
    output.sync_all().map_err(|e| e.to_string())?;
    Ok((path.to_path_buf(), hashes))
}

#[derive(Clone, Copy, Default)]
struct Verified {
    block: i64,
    revision: u64,
}
pub struct CaptureHandle {
    pub engine: Arc<SessionCapture>,
    member: usize,
    owner: u64,
    expected: Option<i64>,
    hash: Sha256,
    hash_start: i64,
    hash_count: usize,
    verified: [Verified; 2048],
}
impl CaptureHandle {
    pub fn reset_stream(&mut self) {
        self.expected = None;
        self.hash = Sha256::new();
        self.hash_count = 0;
        self.verified.fill(Verified::default());
    }
    pub fn unbind(&self) {
        let _lifecycle = self.engine.binding.lock();
        if let Err(error) = self.engine.flush() {
            self.engine.state.lock().message = error;
        }
        if self.engine.owners[self.member]
            .compare_exchange(self.owner, 0, SeqCst, SeqCst)
            .is_ok()
        {
            self.engine.members.lock()[self.member] = Weak::new();
            self.engine.invalidate();
            // Join plugin-owned workers before the last native instance can unload its DLL.
            if self.engine.active_members().is_empty() {
                self.engine.shutdown();
            }
        }
    }
    fn new(engine: Arc<SessionCapture>, member: usize, owner: u64) -> Self {
        Self {
            engine,
            member,
            owner,
            expected: None,
            hash: Sha256::new(),
            hash_start: 0,
            hash_count: 0,
            verified: [Verified::default(); 2048],
        }
    }
    pub fn capture(&mut self, start: Option<i64>, channels: &[&mut [f32]], active: bool) {
        if !active {
            self.reset_stream();
            return;
        }
        let Some(start) = start else {
            self.reset_stream();
            return;
        };
        let count = channels.first().map_or(0, |c| c.len());
        if start < 0
            || start
                .checked_add(count as i64)
                .is_none_or(|end| end > i64::from(self.engine.rate) * MAX_SECONDS)
            || !(1..=2).contains(&channels.len())
        {
            self.reset_stream();
            return;
        }
        if self.expected != Some(start) {
            self.reset_stream();
        }
        let analysis = self.engine.analysis.try_read();
        for offset in (0..count).step_by(BLOCK) {
            let mut packet = Packet {
                member: self.member,
                owner: self.owner,
                start: start + offset as i64,
                count: (count - offset).min(BLOCK),
                channels: channels.len(),
                samples: [[0.0; 2]; BLOCK],
            };
            for n in 0..packet.count {
                let position = packet.start + n as i64;
                if position.rem_euclid(BLOCK as i64) == 0 {
                    self.hash = Sha256::new();
                    self.hash_start = position;
                    self.hash_count = 0;
                }
                for (c, channel) in channels.iter().enumerate() {
                    let value = channel[offset + n];
                    if !value.is_finite() {
                        self.engine.invalidate();
                        self.engine.overrun.store(true, SeqCst);
                        return;
                    }
                    packet.samples[n][c] = value;
                    if position == self.hash_start + self.hash_count as i64 {
                        self.hash.update(value.to_le_bytes());
                    }
                }
                if position == self.hash_start + self.hash_count as i64 {
                    self.hash_count += 1;
                }
                if self.hash_count == BLOCK {
                    if let Some(Some(published)) = analysis.as_deref().map(|p| p.as_ref()) {
                        let revision = self.engine.revision.load(SeqCst);
                        let first = ((published.analysis.origin + BLOCK as i64 - 1) / BLOCK as i64)
                            * BLOCK as i64;
                        let index = (self.hash_start - first).div_euclid(BLOCK as i64);
                        if published.revision == revision
                            && index >= 0
                            && let Some(expected) =
                                published.hashes[self.member].get(index as usize)
                        {
                            let actual: [u8; 32] = self.hash.clone().finalize().into();
                            if actual == *expected {
                                let block = self.hash_start / BLOCK as i64;
                                self.verified[block.rem_euclid(2048) as usize] =
                                    Verified { block, revision };
                            } else {
                                self.engine.invalidate();
                            }
                        }
                    }
                    self.hash_count = 0;
                }
            }
            if self.engine.queue.push(packet).is_err() {
                self.engine.invalidate();
                self.engine.overrun.store(true, SeqCst);
            }
        }
        self.expected = start.checked_add(count as i64);
    }
    pub fn render(
        &self,
        channels: &mut [&mut [f32]],
        start: Option<i64>,
        delay: usize,
        enabled: bool,
        listen: ListenMode,
    ) {
        let analysis = self.engine.analysis.try_read();
        let revision = self.engine.revision.load(SeqCst);
        for n in 0..channels.first().map_or(0, |c| c.len()) {
            let time = start
                .and_then(|a| a.checked_add(n as i64))
                .and_then(|a| a.checked_sub(delay as i64));
            let gain = if enabled {
                time.and_then(|time| {
                    let block = time.div_euclid(BLOCK as i64);
                    let stamp = self.verified[block.rem_euclid(2048) as usize];
                    if stamp.block != block
                        || stamp.revision != revision
                        || self.engine.revision.load(SeqCst) != revision
                    {
                        return None;
                    }
                    let published = analysis.as_deref()?.as_ref()?;
                    (published.revision == revision).then(|| {
                        published.member_map[self.member]
                            .map_or(1.0, |member| published.analysis.gain_at(member, time))
                    })
                })
                .unwrap_or(1.0)
            } else {
                1.0
            };
            for channel in channels.iter_mut() {
                let original = channel[n];
                let processed = if gain == 1.0 {
                    original
                } else {
                    original * gain
                };
                channel[n] = match listen {
                    ListenMode::Original => original,
                    ListenMode::Processed => processed,
                    ListenMode::Removed => original - processed,
                };
            }
        }
    }
}
impl Drop for CaptureHandle {
    fn drop(&mut self) {
        // Activation restarts keep the same owner; weak parameter ownership detects actual removal.
        if !self.engine.stopping.load(SeqCst) {
            let _ = self.engine.flush();
        }
    }
}

#[cfg(test)]
#[path = "session_mode_tests.rs"]
pub(crate) mod integration_tests;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coverage_merges_replays_preserves_holes_and_intersects_tracks() {
        let mut a = Ranges::default();
        a.insert(10, 30).unwrap();
        a.insert(0, 20).unwrap();
        assert!(a.covers(0, 30));
        a.insert(40, 50).unwrap();
        assert!(!a.covers(20, 45));
        let mut b = Ranges::default();
        b.insert(5, 45).unwrap();
        let common = a.intersections(&b);
        assert!(common.covers(5, 30));
        assert!(common.covers(40, 45));
        assert!(!common.covers(30, 40));
    }
    #[test]
    fn cache_keys_cannot_escape_owned_storage() {
        assert!(valid_key(&new_key()));
        assert!(!valid_key("../recordings"));
        assert!(!valid_key(""));
        assert_ne!(new_key(), new_key());
    }
    #[test]
    fn snapshot_preserves_native_stereo_and_exact_fingerprints() {
        let path = std::env::temp_dir().join(format!("cleanslate-capture-test-{}", new_key()));
        fs::create_dir(&path).unwrap();
        let raw = path.join("source.pcm");
        let mut file = File::create(&raw).unwrap();
        let mut bytes = Vec::new();
        for _ in 0..BLOCK * 2 {
            bytes.extend_from_slice(&0.5_f32.to_le_bytes());
            bytes.extend_from_slice(&(-0.5_f32).to_le_bytes());
        }
        file.write_all(&bytes).unwrap();
        drop(file);
        let (_, hashes) = snapshot_wav(
            &mut File::open(raw).unwrap(),
            &path.join("source.wav"),
            0,
            (BLOCK * 2) as i64,
            48000,
            2,
            None,
        )
        .unwrap();
        assert_eq!(hashes.len(), 2);
        assert_eq!(
            hashes[0],
            <[u8; 32]>::from(Sha256::digest(&bytes[..BLOCK * 8]))
        );
        let wav = fs::read(path.join("source.wav")).unwrap();
        assert_eq!(&wav[44..], bytes);
        assert_eq!(
            u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize,
            bytes.len()
        );
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn snapshot_retains_partial_edges_without_trusting_partial_fingerprints() {
        let path = std::env::temp_dir().join(format!("cleanslate-edge-test-{}", new_key()));
        fs::create_dir(&path).unwrap();
        let bytes = (0..BLOCK * 3)
            .flat_map(|i| (i as f32).to_le_bytes())
            .collect::<Vec<_>>();
        fs::write(path.join("source.pcm"), &bytes).unwrap();
        let end = (BLOCK * 2 + 73) as i64;
        let (_, hashes) = snapshot_wav(
            &mut File::open(path.join("source.pcm")).unwrap(),
            &path.join("source.wav"),
            17,
            end,
            48000,
            1,
            None,
        )
        .unwrap();
        assert_eq!(
            hashes,
            vec![<[u8; 32]>::from(Sha256::digest(
                &bytes[BLOCK * 4..BLOCK * 8]
            ))]
        );
        assert_eq!(
            &fs::read(path.join("source.wav")).unwrap()[44..],
            &bytes[17 * 4..end as usize * 4]
        );
        fs::remove_dir_all(path).unwrap();
    }
}
