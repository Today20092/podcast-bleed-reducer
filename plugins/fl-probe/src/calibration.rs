use crate::history::{History, HistorySnapshot};
use cleanslate_ownership_model::{
    OwnershipModel, OwnershipPredictor, OwnershipTrainingExample, OwnershipTrainingSettings,
};
use crossbeam::queue::ArrayQueue;
use nice_plug::params::persist::PersistentField;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, OnceLock, Weak,
        atomic::{AtomicBool, AtomicI64, AtomicU8, AtomicU64, AtomicUsize, Ordering::SeqCst},
    },
};

pub const RECIPE: &str = "plugin-native-ac32-energy-coherence-logistic-v1";
const MAX_ROWS: usize = 2048;
const MAX_RANGES: usize = 128;
const MIN_CLASS: usize = 10;
static SESSIONS: OnceLock<Mutex<[Weak<Session>; 8]>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Label {
    #[default]
    None = 0,
    Room = 1,
    Member1 = 2,
    Member2 = 3,
    Both = 4,
}
impl Label {
    fn from_code(value: u8) -> Self {
        match value {
            1 => Self::Room,
            2 => Self::Member1,
            3 => Self::Member2,
            4 => Self::Both,
            _ => Self::None,
        }
    }
    pub fn mask(self) -> u8 {
        match self {
            Self::Member1 => 1,
            Self::Member2 => 2,
            Self::Both => 3,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceFrame {
    pub start: i64,
    pub end: i64,
    pub label: Label,
    pub rate: u32,
    pub channels: usize,
    pub features: [[f64; 4]; 2],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationDocument {
    pub schema: u32,
    pub recipe: String,
    pub rows: Vec<ReferenceFrame>,
    pub models: [OwnershipModel; 2],
}

#[derive(Default)]
pub struct SavedCalibration {
    document: Mutex<Option<CalibrationDocument>>,
    pub revision: AtomicU64,
}

impl SavedCalibration {
    pub fn lock(&self) -> parking_lot::MutexGuard<'_, Option<CalibrationDocument>> {
        self.document.lock()
    }
}

impl<'a> PersistentField<'a, Option<CalibrationDocument>> for SavedCalibration {
    fn set(&self, document: Option<CalibrationDocument>) {
        // Host state can load while activated; the callback sees only this atomic revision.
        self.revision.fetch_add(1, SeqCst);
        *self.document.lock() = document;
    }
    fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&Option<CalibrationDocument>) -> R,
    {
        f(&self.document.lock())
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct TruthRange {
    start: i64,
    end: i64,
    mask: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct ModelBank {
    pub generation: u64,
    pub predictors: [OwnershipPredictor; 2],
    pub rate: u32,
    pub channels: usize,
    ranges: [TruthRange; MAX_RANGES],
    range_count: usize,
}

impl ModelBank {
    pub fn truth_at(&self, sample: i64) -> Option<u8> {
        let ranges = &self.ranges[..self.range_count];
        let index = ranges.partition_point(|range| range.end <= sample);
        ranges
            .get(index)
            .filter(|range| range.start <= sample)
            .map(|range| range.mask)
    }
}

fn names() -> Vec<String> {
    [
        "local_ac_power_db/60",
        "peer_ac_power_db/60",
        "relative_power_db/60",
        "squared_zero_lag_coherence",
    ]
    .map(String::from)
    .to_vec()
}

fn examples(rows: &[ReferenceFrame], member: usize) -> Vec<OwnershipTrainingExample> {
    rows.iter()
        .map(|row| OwnershipTrainingExample {
            features: row.features[member].to_vec(),
            owns_speech: row.label.mask() & (1 << member) != 0,
        })
        .collect()
}

fn normalize_rows(rows: &[ReferenceFrame]) -> Result<Vec<ReferenceFrame>, String> {
    validate_examples(rows, true)
}

fn validate_examples(
    rows: &[ReferenceFrame],
    require_classes: bool,
) -> Result<Vec<ReferenceFrame>, String> {
    if rows.is_empty() && !require_classes {
        return Ok(Vec::new());
    }
    if rows.is_empty() || rows.len() > MAX_ROWS {
        return Err("Capture bounded examples for room tone and each microphone first.".into());
    }
    let first = rows[0];
    let hz = f32::from_bits(first.rate);
    if !hz.is_finite()
        || hz.fract() != 0.0
        || !(8000.0..=192000.0).contains(&hz)
        || !(1..=2).contains(&first.channels)
    {
        return Err("Unsupported calibration sample rate or channel layout.".into());
    }
    let mut unique = BTreeMap::<i64, ReferenceFrame>::new();
    for &row in rows {
        if row.label == Label::None
            || row.end <= row.start
            || row
                .end
                .checked_sub(row.start)
                .is_none_or(|length| length > 8192)
            || row.rate != first.rate
            || row.channels != first.channels
            || row
                .features
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 1e6)
        {
            return Err(
                "Calibration contains invalid frames or incompatible inputs; clear and recapture."
                    .into(),
            );
        }
        if let Some(old) = unique.insert(row.start, row)
            && (old.label != row.label || old.end != row.end || old.features != row.features)
        {
            return Err("Conflicting examples cover the same source time. Clear and recapture the correct labels.".into());
        }
    }
    let rows: Vec<_> = unique.into_values().collect();
    if rows.windows(2).any(|pair| pair[0].end > pair[1].start) {
        return Err("Reference intervals overlap inconsistently.".into());
    }
    for label in [Label::Room, Label::Member1, Label::Member2]
        .into_iter()
        .filter(|_| require_classes)
    {
        if rows.iter().filter(|row| row.label == label).count() < MIN_CLASS {
            return Err("Capture at least 0.32 seconds each of room tone, microphone 1 solo and microphone 2 solo.".into());
        }
    }
    Ok(rows)
}

impl CalibrationDocument {
    pub fn fit(rows: &[ReferenceFrame], cancel: &AtomicBool) -> Result<Self, String> {
        let rows = normalize_rows(rows)?;
        let mut models = Vec::with_capacity(2);
        for member in 0..2 {
            models.push(
                OwnershipModel::train(
                    names(),
                    &examples(&rows, member),
                    OwnershipTrainingSettings::default(),
                    Some(cancel),
                )
                .map_err(|e| e.to_string())?,
            );
        }
        let document = Self {
            schema: 1,
            recipe: RECIPE.into(),
            rows,
            models: models.try_into().unwrap(),
        };
        document.compile(1)?;
        Ok(document)
    }

    pub fn compile(&self, generation: u64) -> Result<ModelBank, String> {
        if self.schema != 1 || self.recipe != RECIPE {
            return Err("Saved calibration uses an unsupported recipe; recapture examples.".into());
        }
        let rows = normalize_rows(&self.rows)?;
        for (member, model) in self.models.iter().enumerate() {
            if !model.matches_recipe(&names(), OwnershipTrainingSettings::default()) {
                return Err("Saved model feature recipe differs from this plugin.".into());
            }
            model
                .validate_truth_rows(&examples(&rows, member))
                .map_err(|e| e.to_string())?;
        }
        let mut bank = ModelBank {
            generation,
            predictors: [
                self.models[0].predictor().map_err(|e| e.to_string())?,
                self.models[1].predictor().map_err(|e| e.to_string())?,
            ],
            rate: rows[0].rate,
            channels: rows[0].channels,
            ranges: [TruthRange::default(); MAX_RANGES],
            range_count: 0,
        };
        for row in &rows {
            if bank.range_count > 0 {
                let previous = &mut bank.ranges[bank.range_count - 1];
                if previous.end == row.start && previous.mask == row.label.mask() {
                    previous.end = row.end;
                    continue;
                }
            }
            if bank.range_count == MAX_RANGES {
                return Err(
                    "Too many separate reference ranges; clear and capture longer examples.".into(),
                );
            }
            bank.ranges[bank.range_count] = TruthRange {
                start: row.start,
                end: row.end,
                mask: row.label.mask(),
            };
            bank.range_count += 1;
        }
        Ok(bank)
    }

    fn identity(&self) -> Result<String, String> {
        let bytes = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }
}

#[derive(Clone, Copy)]
struct Captured {
    data_generation: u64,
    frame: ReferenceFrame,
}

pub struct SessionView {
    pub capture: Label,
    pub counts: [usize; 4],
    pub trained: bool,
    pub confirmed: bool,
    pub busy: bool,
    pub message: String,
}

pub struct Session {
    operation: Mutex<()>,
    capture: AtomicU8,
    capture_start: AtomicI64,
    capture_id: AtomicU64,
    data_generation: AtomicU64,
    pub generation: AtomicU64,
    pub confirmed: AtomicBool,
    pub current_time: AtomicI64,
    pub running: AtomicBool,
    counts: [AtomicUsize; 4],
    rt_error: AtomicU8,
    dirty: AtomicBool,
    busy: AtomicBool,
    cancel: AtomicBool,
    captured: ArrayQueue<Captured>,
    collected: Mutex<(u64, Vec<ReferenceFrame>)>,
    document: Mutex<Option<CalibrationDocument>>,
    message: Mutex<String>,
    members: Mutex<[Weak<crate::ProbeParams>; 2]>,
    bound_revisions: Mutex<[Option<u64>; 2]>,
    pub models: [ArrayQueue<ModelBank>; 2],
    pub history: History,
    examples_loaded: AtomicBool,
}

pub fn session_for(group: i32) -> Option<Arc<Session>> {
    if !(1..=8).contains(&group) {
        return None;
    }
    let registry = SESSIONS.get_or_init(|| Mutex::new(std::array::from_fn(|_| Weak::new())));
    let mut groups = registry.lock();
    let index = group as usize - 1;
    if let Some(session) = groups[index].upgrade() {
        return Some(session);
    }
    let session = Arc::new(Session::new());
    groups[index] = Arc::downgrade(&session);
    Some(session)
}

impl Session {
    pub(crate) fn new() -> Self {
        Self {
            operation: Mutex::new(()),
            capture: AtomicU8::new(0),
            capture_start: AtomicI64::new(0),
            capture_id: AtomicU64::new(0),
            data_generation: AtomicU64::new(1),
            generation: AtomicU64::new(1),
            confirmed: AtomicBool::new(false),
            current_time: AtomicI64::new(0),
            running: AtomicBool::new(false),
            counts: std::array::from_fn(|_| AtomicUsize::new(0)),
            rt_error: AtomicU8::new(0),
            dirty: AtomicBool::new(false),
            busy: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            captured: ArrayQueue::new(MAX_ROWS),
            collected: Mutex::new((1, Vec::new())),
            document: Mutex::new(None),
            message: Mutex::new(
                "Capture room tone and clear solo examples for both microphones.".into(),
            ),
            members: Mutex::new(std::array::from_fn(|_| Weak::new())),
            bound_revisions: Mutex::new([None; 2]),
            models: std::array::from_fn(|_| ArrayQueue::new(2)),
            history: History::new(),
            examples_loaded: AtomicBool::new(false),
        }
    }

    pub fn bind(
        &self,
        member: usize,
        params: &Arc<crate::ProbeParams>,
        rate: u32,
        channels: usize,
    ) -> Result<(), String> {
        let _operation = self.operation.lock();
        if member > 1 {
            return Err("Choose member 1 or 2.".into());
        }
        let mut members = self.members.lock();
        if let Some(old) = members[member].upgrade()
            && !Arc::ptr_eq(&old, params)
        {
            return Err(
                "This group already has that member; choose a unique member number.".into(),
            );
        }
        let new_member = members[member].upgrade().is_none();
        members[member] = Arc::downgrade(params);
        drop(members);
        let saved_guard = params.saved.lock();
        let revision = params.saved.revision.load(SeqCst);
        let saved = saved_guard.clone();
        let pending_guard = params.saved_examples.lock();
        let pending = pending_guard.clone();
        let incoming = pending
            .as_ref()
            .or_else(|| saved.as_ref().map(|saved| &saved.rows))
            .map(|rows| validate_examples(rows, false))
            .transpose()?;
        if incoming
            .as_ref()
            .and_then(|rows| rows.first())
            .is_some_and(|row| row.rate != rate || row.channels != channels)
        {
            self.confirmed.store(false, SeqCst);
            return Err(
                "Saved examples use a different rate/layout. Clear and relabel the current audio."
                    .into(),
            );
        }
        if self.examples_loaded.load(SeqCst) {
            let current = self.collected.lock();
            if incoming.as_ref().is_some_and(|rows| *rows != current.1)
                || (incoming.is_none() && !current.1.is_empty())
            {
                self.confirmed.store(false, SeqCst);
                return Err("The two instances contain different saved examples. Clear this group and relabel the current audio.".into());
            }
        }
        let mut document = self.document.lock();
        if saved.is_none() && document.is_some() {
            self.confirmed.store(false, SeqCst);
            return Err("This instance has no saved calibration but its group does. Clear the other member's examples, then reload this instance and recapture.".into());
        }
        if let Some(saved) = saved {
            let bank = saved.compile(self.generation.load(SeqCst))?;
            if bank.rate != rate || bank.channels != channels {
                self.confirmed.store(false, SeqCst);
                return Err("Input rate/layout changed. Clear and recapture calibration.".into());
            }
            if let Some(current) = &*document {
                if current.identity()? != saved.identity()? {
                    self.confirmed.store(false, SeqCst);
                    return Err("The two instances contain different saved calibrations. Clear this group and recapture.".into());
                }
            } else {
                *self.collected.lock() = (self.data_generation.load(SeqCst), saved.rows.clone());
                self.update_counts(&saved.rows);
                *document = Some(saved);
                self.models[0].force_push(bank);
                self.models[1].force_push(bank);
                *self.message.lock() = "Saved calibration found. Confirm the same microphones/sources before using it.".into();
            }
        }
        if new_member && document.is_some() {
            self.confirmed.store(false, SeqCst);
        }
        if let Some(rows) = incoming {
            let changed = document
                .as_ref()
                .is_none_or(|document| document.rows != rows);
            *self.collected.lock() = (self.data_generation.load(SeqCst), rows.clone());
            self.update_counts(&rows);
            self.examples_loaded.store(true, SeqCst);
            self.dirty
                .store(changed && (!rows.is_empty() || document.is_some()), SeqCst);
            if self.dirty.load(SeqCst) {
                self.confirmed.store(false, SeqCst);
                *self.message.lock() =
                    "Saved examples restored. Learn to apply the current labels.".into();
            }
        }
        if params.saved.revision.load(SeqCst) != revision {
            self.confirmed.store(false, SeqCst);
            return Err("Host state changed while reconnecting. Reload this instance.".into());
        }
        self.bound_revisions.lock()[member] = Some(revision);
        Ok(())
    }

    pub fn unbind(&self, member: usize, params: &Arc<crate::ProbeParams>) {
        let _operation = self.operation.lock();
        let mut members = self.members.lock();
        if members[member]
            .upgrade()
            .is_some_and(|old| Arc::ptr_eq(&old, params))
        {
            members[member] = Weak::new();
            self.bound_revisions.lock()[member] = None;
            self.confirmed.store(false, SeqCst);
            self.generation.fetch_add(1, SeqCst);
            self.cancel.store(true, SeqCst);
            self.busy.store(false, SeqCst);
            if self.capture.swap(0, SeqCst) != 0 {
                self.data_generation.fetch_add(1, SeqCst);
                self.rt_error.store(1, SeqCst);
            }
        }
    }

    pub fn snapshot(&self) -> SessionView {
        let error = match self.rt_error.load(SeqCst) {
            1 => Some("Capture cancelled by a seek. Clear and recapture the examples."),
            2 => Some("Example storage is full. Learn or clear before more capture."),
            _ => None,
        };
        SessionView {
            capture: Label::from_code(self.capture.load(SeqCst)),
            counts: std::array::from_fn(|i| self.counts[i].load(SeqCst)),
            trained: self.document.lock().is_some(),
            confirmed: self.confirmed.load(SeqCst),
            busy: self.busy.load(SeqCst),
            message: error
                .map(str::to_string)
                .unwrap_or_else(|| self.message.lock().clone()),
        }
    }

    pub fn begin_capture(&self, label: Label) -> Result<(), String> {
        let _operation = self.operation.lock();
        if label == Label::None || self.busy.load(SeqCst) {
            return Err("Wait for learning to finish.".into());
        }
        if !self.running.load(SeqCst) {
            return Err("Start host playback, then capture the example while it plays.".into());
        }
        if self.rt_error.load(SeqCst) == 1 {
            return Err("Clear the cancelled calibration before capturing again.".into());
        }
        self.confirmed.store(false, SeqCst);
        self.dirty.store(true, SeqCst);
        self.generation.fetch_add(1, SeqCst);
        self.capture_start
            .store(self.current_time.load(SeqCst), SeqCst);
        self.capture_id.fetch_add(1, SeqCst);
        self.capture.store(label as u8, SeqCst);
        Ok(())
    }

    pub fn stop_capture(&self) {
        self.capture.store(0, SeqCst);
    }

    #[cfg(test)]
    pub fn clear(&self) {
        let _ = self.try_clear();
    }

    pub fn try_clear(&self) -> Result<(), String> {
        let _operation = self.operation.lock();
        let members = self.bound_members();
        let mut saved = members
            .each_ref()
            .map(|params| params.as_ref().map(|params| params.saved.lock()));
        let mut pending = members
            .each_ref()
            .map(|params| params.as_ref().map(|params| params.saved_examples.lock()));
        self.check_bound_revisions(&members)?;
        self.capture.store(0, SeqCst);
        self.cancel.store(true, SeqCst);
        self.busy.store(false, SeqCst);
        self.confirmed.store(false, SeqCst);
        self.dirty.store(false, SeqCst);
        self.generation.fetch_add(1, SeqCst);
        let generation = self.data_generation.fetch_add(1, SeqCst) + 1;
        *self.collected.lock() = (generation, Vec::new());
        *self.document.lock() = None;
        for slot in saved.iter_mut().flatten() {
            **slot = None;
        }
        for slot in pending.iter_mut().flatten() {
            **slot = Some(Vec::new());
        }
        self.examples_loaded.store(true, SeqCst);
        for count in &self.counts {
            count.store(0, SeqCst);
        }
        while self.captured.pop().is_some() {}
        self.rt_error.store(0, SeqCst);
        *self.message.lock() =
            "Cleared. Capture room tone and each microphone's solo speech.".into();
        Ok(())
    }

    pub fn confirm_saved(&self) -> Result<(), String> {
        let _operation = self.operation.lock();
        let members = self.bound_members();
        let _saved = members
            .each_ref()
            .map(|params| params.as_ref().map(|params| params.saved.lock()));
        self.check_bound_revisions(&members)?;
        if self.dirty.load(SeqCst) || self.rt_error.load(SeqCst) != 0 {
            return Err("Learn the changed examples or clear and recapture first.".into());
        }
        let document = self.document.lock();
        let Some(document) = &*document else {
            return Err("No valid saved calibration is available.".into());
        };
        let bank = document.compile(self.generation.load(SeqCst))?;
        self.check_bound_revisions(&members)?;
        for queue in &self.models {
            queue.force_push(bank);
        }
        self.confirmed.store(true, SeqCst);
        *self.message.lock() = "Calibration confirmed. Enable processing to audition.".into();
        Ok(())
    }

    pub fn transport(&self, start: i64, playing: bool, discontinuity: bool) {
        self.current_time.store(start, SeqCst);
        self.running.store(playing, SeqCst);
        if self.capture.load(SeqCst) != 0 {
            if !playing {
                self.capture.store(0, SeqCst);
            } else if discontinuity {
                self.capture.store(0, SeqCst);
                self.cancel.store(true, SeqCst);
                self.confirmed.store(false, SeqCst);
                self.data_generation.fetch_add(1, SeqCst);
                self.generation.fetch_add(1, SeqCst);
                for count in &self.counts {
                    count.store(0, SeqCst);
                }
                self.rt_error.store(1, SeqCst);
            }
        }
    }

    pub fn capture_key(&self) -> (u64, u64) {
        (
            self.data_generation.load(SeqCst),
            self.capture_id.load(SeqCst),
        )
    }

    pub fn capture_frame(&self, mut frame: ReferenceFrame) {
        let generation = self.data_generation.load(SeqCst);
        let label = Label::from_code(self.capture.load(SeqCst));
        if label == Label::None || frame.start < self.capture_start.load(SeqCst) {
            return;
        }
        if self.counts.iter().map(|c| c.load(SeqCst)).sum::<usize>() >= MAX_ROWS {
            self.capture.store(0, SeqCst);
            self.rt_error.store(2, SeqCst);
            return;
        }
        frame.label = label;
        if self.data_generation.load(SeqCst) != generation {
            return;
        }
        if self
            .captured
            .push(Captured {
                data_generation: generation,
                frame,
            })
            .is_err()
        {
            self.capture.store(0, SeqCst);
            self.rt_error.store(2, SeqCst);
            return;
        }
        self.counts[label as usize - 1].fetch_add(1, SeqCst);
    }

    pub fn labels(&self) -> Vec<ReferenceFrame> {
        let collected = self.collected.lock();
        if collected.0 == self.data_generation.load(SeqCst) {
            collected.1.clone()
        } else {
            Vec::new()
        }
    }

    pub fn label_history(
        &self,
        snapshot: &HistorySnapshot,
        start: i64,
        end: i64,
        label: Label,
    ) -> Result<usize, String> {
        let _operation = self.operation.lock();
        let members = self.bound_members();
        let _saved = members
            .each_ref()
            .map(|params| params.as_ref().map(|params| params.saved.lock()));
        let mut pending = members
            .each_ref()
            .map(|params| params.as_ref().map(|params| params.saved_examples.lock()));
        self.check_bound_revisions(&members)?;
        if self.busy.load(SeqCst) {
            return Err("Wait for learning to finish before changing labels.".into());
        }
        if start >= end || snapshot.epoch == 0 || snapshot.epoch != self.history.latest_epoch() {
            return Err("History changed after a seek or reconnect. Return to Live and freeze a new section.".into());
        }
        let mut selected: Vec<_> = snapshot
            .frames
            .iter()
            .filter(|frame| frame.reference.start >= start && frame.reference.end <= end)
            .map(|frame| {
                let mut row = frame.reference;
                row.label = if label == Label::None {
                    Label::Room
                } else {
                    label
                };
                (frame.epoch, row)
            })
            .collect();
        if selected.is_empty() {
            return Err("Select at least one complete 32 ms frame.".into());
        }
        if selected.iter().any(|(epoch, _)| *epoch != snapshot.epoch) {
            return Err("The selection crosses incompatible history. Freeze a new section.".into());
        }
        selected.sort_by_key(|(_, row)| row.start);
        let rows: Vec<_> = selected.into_iter().map(|(_, row)| row).collect();
        let rows = validate_examples(&rows, false)?;
        if rows.windows(2).any(|pair| pair[0].end != pair[1].start) {
            return Err(
                "The selection contains missing paired audio. Select a continuous section.".into(),
            );
        }
        let left = rows[0].start;
        let right = rows.last().unwrap().end;
        let generation = self.data_generation.load(SeqCst);
        let mut collected = self.collected.lock();
        let mut updated = if collected.0 == generation {
            collected.1.clone()
        } else {
            Vec::new()
        };
        // Drain the old live-capture ingress before editing the same source-time examples.
        for _ in 0..MAX_ROWS {
            let Some(captured) = self.captured.pop() else {
                break;
            };
            if captured.data_generation == generation {
                updated.push(captured.frame);
            }
        }
        updated.retain(|row| row.end <= left || row.start >= right);
        if label != Label::None {
            updated.extend_from_slice(&rows);
        }
        let updated = validate_examples(&updated, false)?;
        if snapshot.epoch != self.history.latest_epoch()
            || generation != self.data_generation.load(SeqCst)
        {
            return Err(
                "History changed while editing. Return to Live and freeze a new section.".into(),
            );
        }
        self.check_bound_revisions(&members)?;
        self.stop_capture();
        self.cancel.store(true, SeqCst);
        self.generation.fetch_add(1, SeqCst);
        self.confirmed.store(false, SeqCst);
        self.dirty.store(true, SeqCst);
        self.examples_loaded.store(true, SeqCst);
        self.rt_error.store(0, SeqCst);
        self.update_counts(&updated);
        *collected = (generation, updated.clone());
        drop(collected);
        for slot in pending.iter_mut().flatten() {
            **slot = Some(updated.clone());
        }
        *self.message.lock() = "Examples changed. Learn to apply these labels.".into();
        Ok(rows.len())
    }

    fn update_counts(&self, rows: &[ReferenceFrame]) {
        for (i, count) in self.counts.iter().enumerate() {
            count.store(
                rows.iter().filter(|r| r.label as usize == i + 1).count(),
                SeqCst,
            );
        }
    }

    fn bound_members(&self) -> [Option<Arc<crate::ProbeParams>>; 2] {
        let registered = self.members.lock();
        std::array::from_fn(|i| registered[i].upgrade())
    }

    // Call with each member's saved-model guard held, in member order, before editing state.
    fn check_bound_revisions(
        &self,
        members: &[Option<Arc<crate::ProbeParams>>; 2],
    ) -> Result<(), String> {
        let expected = self.bound_revisions.lock();
        if members.iter().enumerate().any(|(i, params)| {
            params
                .as_ref()
                .map(|params| params.saved.revision.load(SeqCst))
                != expected[i]
        }) {
            return Err(
                "Host state changed. Reload the instances before changing examples.".into(),
            );
        }
        Ok(())
    }
}

pub struct TrainingTask {
    session: Arc<Session>,
    generation: u64,
    saved_revisions: [Option<u64>; 2],
}
impl TrainingTask {
    pub fn new(session: Arc<Session>) -> Self {
        let operation = session.operation.lock();
        session.stop_capture();
        session.cancel.store(true, SeqCst);
        session.confirmed.store(false, SeqCst);
        let generation = session.generation.fetch_add(1, SeqCst) + 1;
        session.busy.store(true, SeqCst);
        let saved_revisions = *session.bound_revisions.lock();
        drop(operation);
        Self {
            session,
            generation,
            saved_revisions,
        }
    }

    pub fn run(self) {
        if self.session.generation.load(SeqCst) != self.generation {
            return;
        }
        self.session.cancel.store(false, SeqCst);
        let result = self.fit();
        let _operation = self.session.operation.lock();
        if self.session.generation.load(SeqCst) != self.generation {
            return;
        }
        match result {
            Ok(document) => {
                let bank = match document.compile(self.generation) {
                    Ok(bank) => bank,
                    Err(error) => {
                        *self.session.message.lock() = error;
                        self.session.busy.store(false, SeqCst);
                        return;
                    }
                };
                let members: [Option<Arc<crate::ProbeParams>>; 2] = {
                    let registered = self.session.members.lock();
                    std::array::from_fn(|i| registered[i].upgrade())
                };
                let mut saved = members
                    .each_ref()
                    .map(|params| params.as_ref().map(|params| params.saved.lock()));
                let mut pending = members
                    .each_ref()
                    .map(|params| params.as_ref().map(|params| params.saved_examples.lock()));
                if members.iter().enumerate().any(|(i, params)| {
                    params
                        .as_ref()
                        .map(|params| params.saved.revision.load(SeqCst))
                        != self.saved_revisions[i]
                }) {
                    self.session.confirmed.store(false, SeqCst);
                    self.session.busy.store(false, SeqCst);
                    *self.session.message.lock() = "Host state changed during learning. Reload the instances before using calibration.".into();
                    return;
                }
                for document_slot in saved.iter_mut().flatten() {
                    **document_slot = Some(document.clone());
                }
                for slot in pending.iter_mut().flatten() {
                    **slot = Some(document.rows.clone());
                }
                self.session.examples_loaded.store(true, SeqCst);
                self.session.update_counts(&document.rows);
                *self.session.document.lock() = Some(document);
                for queue in &self.session.models {
                    queue.force_push(bank);
                }
                self.session.dirty.store(false, SeqCst);
                self.session.confirmed.store(true, SeqCst);
                *self.session.message.lock() =
                    "Learned. Enable processing, then compare Original, Processed and Removed."
                        .into();
            }
            Err(error) => {
                *self.session.message.lock() = error;
            }
        }
        self.session.busy.store(false, SeqCst);
    }

    fn fit(&self) -> Result<CalibrationDocument, String> {
        if self.session.rt_error.load(SeqCst) == 1 {
            return Err("Capture was invalidated by a seek; clear and recapture.".into());
        }
        let generation = self.session.data_generation.load(SeqCst);
        let mut collected = self.session.collected.lock();
        if collected.0 != generation {
            *collected = (generation, Vec::new());
        }
        for _ in 0..MAX_ROWS {
            let Some(captured) = self.session.captured.pop() else {
                break;
            };
            if captured.data_generation == generation {
                if collected.1.len() == MAX_ROWS {
                    return Err(
                        "Too many stored examples. Clear and capture shorter sections.".into(),
                    );
                }
                collected.1.push(captured.frame);
            }
        }
        let rows = collected.1.clone();
        drop(collected);
        CalibrationDocument::fit(&rows, &self.session.cancel)
    }
}

// shortcut: this streaming recipe omits whole-recording VAD/transfer context; replace only after
// equivalent streaming features pass measured wanted-speech, overlap and bleed qualification.
pub fn acoustic_features(a: &[[f32; 2]], b: &[[f32; 2]], channels: usize) -> Option<[[f64; 4]; 2]> {
    if a.len() != b.len() || a.len() < 2 || !(1..=2).contains(&channels) {
        return None;
    }
    let mut sum_a = [0.0; 2];
    let mut sum_b = [0.0; 2];
    let (mut aa, mut bb, mut ab) = (0.0, 0.0, 0.0);
    for (a, b) in a.iter().zip(b) {
        for c in 0..channels {
            let (x, y) = (f64::from(a[c]), f64::from(b[c]));
            if !x.is_finite() || !y.is_finite() {
                return None;
            }
            sum_a[c] += x;
            sum_b[c] += y;
            aa += x * x;
            bb += y * y;
            ab += x * y;
        }
    }
    let n = a.len() as f64;
    for c in 0..channels {
        aa -= sum_a[c] * sum_a[c] / n;
        bb -= sum_b[c] * sum_b[c] / n;
        ab -= sum_a[c] * sum_b[c] / n;
    }
    let energy = |v: f64| {
        (10.0 * (v.max(0.0) / (n * channels as f64)).max(1e-12).log10()).clamp(-120.0, 12.0)
    };
    let (a_db, b_db) = (energy(aa), energy(bb));
    let coherence = if aa > 1e-12 && bb > 1e-12 {
        (ab * ab / (aa * bb)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Some([
        [a_db / 60.0, b_db / 60.0, (a_db - b_db) / 60.0, coherence],
        [b_db / 60.0, a_db / 60.0, (b_db - a_db) / 60.0, coherence],
    ])
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn signal(
        label: Label,
        variation: usize,
        channels: usize,
    ) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
        let mut a = Vec::new();
        let mut b = Vec::new();
        let level = 0.1 + (variation % 11) as f32 * 0.012;
        for n in 0..1536 {
            let x = (n as f32 * (0.073 + variation as f32 * 0.0003)).sin() * level;
            let y = (n as f32 * (0.119 + variation as f32 * 0.0002)).sin() * level;
            let (x, y) = match label {
                Label::Member1 => (x, x * 0.04),
                Label::Member2 => (y * 0.04, y),
                Label::Both => (x + y * 0.04, y + x * 0.04),
                _ => (x * 0.0002, y * 0.0002),
            };
            a.push([x, if channels == 2 { -x } else { 0.0 }]);
            b.push([y, if channels == 2 { -y } else { 0.0 }]);
        }
        (a, b)
    }

    pub fn rows() -> Vec<ReferenceFrame> {
        let mut rows = Vec::new();
        for label in [Label::Room, Label::Member1, Label::Member2, Label::Both] {
            for i in 0..60 {
                let (a, b) = signal(label, i, 2);
                let start = rows.len() as i64 * 1536;
                rows.push(ReferenceFrame {
                    start,
                    end: start + 1536,
                    label,
                    rate: 48000_f32.to_bits(),
                    channels: 2,
                    features: acoustic_features(&a, &b, 2).unwrap(),
                });
            }
        }
        rows
    }

    #[test]
    fn acoustic_training_generalizes_to_held_out_room_solos_and_overlap() {
        let document = CalibrationDocument::fit(&rows(), &AtomicBool::new(false)).unwrap();
        let restored: CalibrationDocument =
            serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
        let bank = restored.compile(9).unwrap();
        assert_eq!(bank.generation, 9);
        for label in [Label::Room, Label::Member1, Label::Member2, Label::Both] {
            for variation in 61..70 {
                let (a, b) = signal(label, variation, 2);
                let features = acoustic_features(&a, &b, 2).unwrap();
                for (member, features) in features.iter().enumerate() {
                    let probability = bank.predictors[member].estimate(features).unwrap();
                    assert!(
                        if label.mask() & (1 << member) != 0 {
                            probability > 0.8
                        } else {
                            probability < 0.2
                        },
                        "{label:?}, member {member}, probability {probability}"
                    );
                }
            }
        }
        assert_eq!(bank.truth_at(60 * 1536), Some(1));
        assert_eq!(bank.truth_at(240 * 1536), None);
        assert!(acoustic_features(&[[f32::NAN; 2]; 4], &[[0.0; 2]; 4], 2).is_none());
    }

    #[test]
    fn invalid_capture_and_obsolete_learning_cannot_enable_processing() {
        let mut examples = rows();
        examples[0].features[0][0] = f64::NAN;
        assert!(CalibrationDocument::fit(&examples, &AtomicBool::new(false)).is_err());
        assert!(CalibrationDocument::fit(&rows()[..60], &AtomicBool::new(false)).is_err());
        assert!(CalibrationDocument::fit(&rows(), &AtomicBool::new(true)).is_err());
        let session = Arc::new(Session::new());
        assert!(session.begin_capture(Label::Room).is_err());
        session.transport(0, true, false);
        session.begin_capture(Label::Room).unwrap();
        session.capture_frame(rows()[0]);
        session.transport(900000, true, true);
        assert!(session.begin_capture(Label::Member1).is_err());
        assert!(session.confirm_saved().is_err());
        session.clear();
        for row in rows() {
            session.transport(row.start, true, false);
            session.begin_capture(row.label).unwrap();
            session.capture_frame(row);
            session.stop_capture();
        }
        let task = TrainingTask::new(session.clone());
        session.clear();
        task.run();
        assert!(!session.confirmed.load(SeqCst));
        assert!(!session.snapshot().busy);
        assert!(!session.snapshot().trained);
    }

    #[test]
    fn restored_state_has_a_callback_visible_revision_and_legacy_state_clears_processing() {
        use nice_plug::plugin::ParamValue;
        use nice_plug::prelude::*;
        let saved = SavedCalibration::default();
        assert_eq!(saved.revision.load(SeqCst), 0);
        PersistentField::set(&saved, None);
        assert_eq!(saved.revision.load(SeqCst), 1);
        let mut state: PluginState = serde_json::from_str(
            r#"{"version":"0.0.1","params":{"probe_group":{"i32":1}},"fields":{}}"#,
        )
        .unwrap();
        crate::FlProbe::filter_state(&mut state);
        assert_eq!(state.fields.get("calibration").unwrap(), "null");
        assert!(matches!(
            state.params.get("enable_processing"),
            Some(ParamValue::Bool(false))
        ));
    }

    #[test]
    fn frozen_history_labels_can_be_replaced_removed_and_survive_live_overwrite() {
        use crate::history::{BINS, HistoryFrame, Peak};
        let session = Session::new();
        for mut row in rows().into_iter().take(30) {
            row.label = Label::None;
            session.history.push(HistoryFrame {
                epoch: 3,
                reference: row,
                peaks: [[Peak::default(); BINS]; 2],
            });
        }
        let frozen = session.history.snapshot();
        assert_eq!(
            session
                .label_history(&frozen, 0, 15360, Label::Room)
                .unwrap(),
            10
        );
        assert_eq!(session.snapshot().counts, [10, 0, 0, 0]);
        assert_eq!(
            session
                .label_history(&frozen, 0, 15360, Label::Member1)
                .unwrap(),
            10
        );
        assert_eq!(session.snapshot().counts, [0, 10, 0, 0]);
        assert!(!session.confirmed.load(SeqCst));
        assert!(session.confirm_saved().is_err());
        let mut next = frozen.frames[0];
        for i in 30..3000 {
            next.reference.start = i * 1536;
            next.reference.end = (i + 1) * 1536;
            session.history.push(next);
        }
        let live = session.history.snapshot();
        assert!(live.frames[0].reference.start > frozen.frames.last().unwrap().reference.end);
        assert_eq!(session.labels().len(), 10);
        assert_eq!(
            session
                .label_history(&frozen, 0, 15360, Label::None)
                .unwrap(),
            10
        );
        assert!(session.labels().is_empty());
        next.epoch = 4;
        session.history.push(next);
        assert!(
            session
                .label_history(&frozen, 0, 15360, Label::Room)
                .is_err()
        );
        assert!(session.labels().is_empty());
    }

    #[test]
    fn partial_examples_restore_without_enabling_an_old_model_and_conflicts_reject() {
        use crate::history::{BINS, HistoryFrame, Peak};
        let session = Session::new();
        let a = crate::FlProbe::default().params.clone();
        let b = crate::FlProbe::default().params.clone();
        session.bind(0, &a, 48000_f32.to_bits(), 2).unwrap();
        session.bind(1, &b, 48000_f32.to_bits(), 2).unwrap();
        for mut row in rows().into_iter().take(10) {
            row.label = Label::None;
            session.history.push(HistoryFrame {
                epoch: 1,
                reference: row,
                peaks: [[Peak::default(); BINS]; 2],
            });
        }
        session
            .label_history(&session.history.snapshot(), 0, 15360, Label::Member1)
            .unwrap();
        assert_eq!(a.saved_examples.lock().as_ref().unwrap().len(), 10);
        assert_eq!(*a.saved_examples.lock(), *b.saved_examples.lock());
        let restored = Session::new();
        restored.bind(0, &a, 48000_f32.to_bits(), 2).unwrap();
        restored.bind(1, &b, 48000_f32.to_bits(), 2).unwrap();
        assert_eq!(restored.snapshot().counts, [0, 10, 0, 0]);
        assert!(restored.confirm_saved().is_err());
        assert_eq!(restored.labels().len(), 10);
        b.saved_examples.lock().as_mut().unwrap()[0].label = Label::Room;
        assert!(restored.bind(1, &b, 48000_f32.to_bits(), 2).is_err());
        restored.clear();
        assert!(a.saved_examples.lock().as_ref().unwrap().is_empty());
        assert!(b.saved_examples.lock().as_ref().unwrap().is_empty());
    }

    #[test]
    fn history_labels_train_and_unlearned_edits_remain_stale_after_reload() {
        use crate::history::{BINS, HistoryFrame, Peak};
        let session = Arc::new(Session::new());
        let a = crate::FlProbe::default().params.clone();
        let b = crate::FlProbe::default().params.clone();
        session.bind(0, &a, 48000_f32.to_bits(), 2).unwrap();
        session.bind(1, &b, 48000_f32.to_bits(), 2).unwrap();
        for mut row in rows() {
            row.label = Label::None;
            session.history.push(HistoryFrame {
                epoch: 9,
                reference: row,
                peaks: [[Peak::default(); BINS]; 2],
            });
        }
        let frozen = session.history.snapshot();
        for (i, label) in [Label::Room, Label::Member1, Label::Member2, Label::Both]
            .into_iter()
            .enumerate()
        {
            session
                .label_history(
                    &frozen,
                    i as i64 * 60 * 1536,
                    (i + 1) as i64 * 60 * 1536,
                    label,
                )
                .unwrap();
        }
        TrainingTask::new(session.clone()).run();
        assert!(
            session.confirmed.load(SeqCst),
            "{}",
            session.snapshot().message
        );
        assert_eq!(session.snapshot().counts, [60, 60, 60, 60]);
        session
            .label_history(&frozen, 60 * 1536, 61 * 1536, Label::Room)
            .unwrap();
        assert!(!session.confirmed.load(SeqCst));
        assert!(session.confirm_saved().is_err());
        let restored = Session::new();
        restored.bind(0, &a, 48000_f32.to_bits(), 2).unwrap();
        restored.bind(1, &b, 48000_f32.to_bits(), 2).unwrap();
        assert_eq!(restored.snapshot().counts, [61, 59, 60, 60]);
        assert!(
            restored.confirm_saved().is_err(),
            "A saved old model cannot bypass unlearned edits"
        );
    }

    #[test]
    fn loaded_host_state_cannot_be_overwritten_by_frozen_labels_clear_or_learning() {
        use crate::history::{BINS, HistoryFrame, Peak};
        let session = Arc::new(Session::new());
        let a = crate::FlProbe::default().params.clone();
        let b = crate::FlProbe::default().params.clone();
        session.bind(0, &a, 48000_f32.to_bits(), 2).unwrap();
        session.bind(1, &b, 48000_f32.to_bits(), 2).unwrap();
        for mut row in rows() {
            row.label = Label::None;
            session.history.push(HistoryFrame {
                epoch: 11,
                reference: row,
                peaks: [[Peak::default(); BINS]; 2],
            });
        }
        let frozen = session.history.snapshot();
        for (i, label) in [Label::Room, Label::Member1, Label::Member2, Label::Both]
            .into_iter()
            .enumerate()
        {
            session
                .label_history(
                    &frozen,
                    i as i64 * 60 * 1536,
                    (i + 1) as i64 * 60 * 1536,
                    label,
                )
                .unwrap();
        }
        TrainingTask::new(session.clone()).run();
        assert!(session.confirmed.load(SeqCst));
        let incoming_model = a.saved.lock().clone();
        let mut incoming_examples = a.saved_examples.lock().clone().unwrap();
        incoming_examples[60].label = Label::Room;
        // Host state loads before its reactivation/next paired history callback.
        PersistentField::set(&a.saved, incoming_model.clone());
        *a.saved_examples.lock() = Some(incoming_examples.clone());
        let labels_before = session.labels();
        assert!(
            session
                .label_history(&frozen, 0, 1536, Label::Member2)
                .is_err()
        );
        assert!(session.try_clear().is_err());
        assert!(session.confirm_saved().is_err());
        assert_eq!(session.labels(), labels_before);
        assert_eq!(
            a.saved_examples.lock().as_ref().unwrap(),
            &incoming_examples
        );
        assert_eq!(
            a.saved.lock().as_ref().unwrap().identity().unwrap(),
            incoming_model.as_ref().unwrap().identity().unwrap()
        );
        // Even a newly queued old-session Learn task must not adopt the incoming revision.
        TrainingTask::new(session.clone()).run();
        assert!(!session.confirmed.load(SeqCst));
        assert_eq!(
            a.saved_examples.lock().as_ref().unwrap(),
            &incoming_examples
        );
        assert_eq!(
            a.saved.lock().as_ref().unwrap().identity().unwrap(),
            incoming_model.as_ref().unwrap().identity().unwrap()
        );
    }

    #[test]
    fn history_selection_rejects_missing_paired_intervals_and_invalid_rows() {
        use crate::history::{BINS, HistoryFrame, Peak};
        let session = Session::new();
        for mut row in rows()
            .into_iter()
            .take(3)
            .enumerate()
            .filter_map(|(i, row)| (i != 1).then_some(row))
        {
            row.label = Label::None;
            session.history.push(HistoryFrame {
                epoch: 7,
                reference: row,
                peaks: [[Peak::default(); BINS]; 2],
            });
        }
        let snapshot = session.history.snapshot();
        assert!(
            session
                .label_history(&snapshot, 0, 3 * 1536, Label::Room)
                .is_err()
        );
        let mut invalid = snapshot.clone();
        invalid.frames[0].reference.features[0][0] = f64::NAN;
        assert!(
            session
                .label_history(&invalid, 0, 1536, Label::Room)
                .is_err()
        );
        assert!(session.labels().is_empty());
    }

    #[test]
    fn learn_persists_both_members_and_reload_requires_confirmation() {
        let session = Arc::new(Session::new());
        let a = crate::FlProbe::default().params.clone();
        let b = crate::FlProbe::default().params.clone();
        session.bind(0, &a, 48000_f32.to_bits(), 2).unwrap();
        session.bind(1, &b, 48000_f32.to_bits(), 2).unwrap();
        for row in rows() {
            session.transport(row.start, true, false);
            session.begin_capture(row.label).unwrap();
            session.capture_frame(row);
            session.stop_capture();
        }
        TrainingTask::new(session.clone()).run();
        assert!(
            session.confirmed.load(SeqCst),
            "{}",
            session.snapshot().message
        );
        assert_eq!(
            a.saved.lock().as_ref().unwrap().identity().unwrap(),
            b.saved.lock().as_ref().unwrap().identity().unwrap()
        );
        let restored = Session::new();
        restored.bind(0, &a, 48000_f32.to_bits(), 2).unwrap();
        restored.bind(1, &b, 48000_f32.to_bits(), 2).unwrap();
        assert!(!restored.confirmed.load(SeqCst));
        restored.confirm_saved().unwrap();
        assert!(restored.confirmed.load(SeqCst));
        *a.saved.lock() = None;
        assert!(restored.bind(0, &a, 48000_f32.to_bits(), 2).is_err());
        assert!(
            !restored.confirmed.load(SeqCst),
            "Loading empty state cannot retain calibrated processing"
        );
        assert!(restored.bind(1, &a, 48000_f32.to_bits(), 2).is_err());
        restored.clear();
        assert!(a.saved.lock().is_none() && b.saved.lock().is_none());
        assert!(!restored.confirmed.load(SeqCst));
        let obsolete = TrainingTask::new(session.clone());
        PersistentField::set(&a.saved, None);
        obsolete.run();
        assert!(
            a.saved.lock().is_none(),
            "An old training result cannot overwrite loaded host state"
        );
        assert!(!session.confirmed.load(SeqCst));
    }
}
