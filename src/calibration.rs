//! Source-bound human examples used by joint microphone ownership.
use crate::{RenderBlocker, waveform::SourceStamp};
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

impl OwnershipSettings {
    /// Validate half-open ranges against the same ordered sources used for analysis.
    /// A quarter second supplies several analysis frames for either example kind.
    pub fn validate(
        &self,
        sources: &[SourceStamp],
        sample_counts: &[usize],
    ) -> Result<(), RenderBlocker> {
        if self.examples.is_empty() {
            return Ok(());
        }
        if self.sources != sources || sources.len() != sample_counts.len() {
            return Err(RenderBlocker(
                "Calibration sources changed. Remove the old examples and select new ones.".into(),
            ));
        }
        for example in &self.examples {
            let Some(&samples) = sample_counts.get(example.track) else {
                return Err(RenderBlocker(
                    "Calibration track is not in this session.".into(),
                ));
            };
            if example.end > samples
                || (example.kind == CalibrationKind::SoloSpeech
                    && sample_counts.iter().any(|&count| example.end > count))
                || example
                    .end
                    .checked_sub(example.start)
                    .is_none_or(|length| length < 12_000)
            {
                return Err(RenderBlocker("Calibration examples must span at least 0.25 seconds within the source recording.".into()));
            }
        }
        Ok(())
    }
}
