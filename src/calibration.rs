//! Source-bound human examples used by joint microphone ownership.
use crate::waveform::SourceStamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationKind {
    RoomTone,
    SoloSpeech,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalibrationExample {
    pub kind: CalibrationKind,
    pub track: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OwnershipSettings {
    pub enabled: bool,
    pub examples: Vec<CalibrationExample>,
    pub sources: Vec<SourceStamp>,
}
