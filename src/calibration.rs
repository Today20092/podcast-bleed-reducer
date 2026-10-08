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
    pub start: crate::TimePosition,
    pub end: crate::TimePosition,
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
        grids: &[crate::SourceTimebase],
    ) -> Result<(), RenderBlocker> {
        let durations = grids
            .iter()
            .map(|grid| grid.duration())
            .collect::<Result<Vec<_>, _>>()?;
        self.validate_durations(sources, &durations)
    }

    pub(crate) fn validate_durations(
        &self,
        sources: &[SourceStamp],
        durations: &[crate::TimePosition],
    ) -> Result<(), RenderBlocker> {
        if self.examples.is_empty() {
            return Ok(());
        }
        if self.sources != sources || sources.len() != durations.len() {
            return Err(RenderBlocker(
                "Calibration sources changed. Remove the old examples and select new ones.".into(),
            ));
        }
        for example in &self.examples {
            let Some(&duration) = durations.get(example.track) else {
                return Err(RenderBlocker(
                    "Calibration track is not in this session.".into(),
                ));
            };
            if example.end > duration
                || (example.kind == CalibrationKind::SoloSpeech
                    && durations.iter().any(|&duration| example.end > duration))
                || example
                    .end
                    .checked_sub(example.start)
                    .is_none_or(|length| length < crate::TICKS_PER_SECOND / 4)
            {
                return Err(RenderBlocker("Calibration examples must span at least 0.25 seconds within the source recording.".into()));
            }
        }
        Ok(())
    }
}
