//! Project time is independent of any audio grid. Required rates have exact ticks;
//! other rates round preserved edges outward by less than one tick/sample.
use crate::RenderBlocker;
use serde::{Deserialize, Serialize};
use std::ops::Range;

pub type TimePosition = u64;
pub const TICKS_PER_SECOND: u64 = 28_224_000;
pub(crate) const DETECTOR_SAMPLE_RATE: u32 = 48_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    Floor,
    Ceil,
}

pub fn time_from_seconds(seconds: f64, rounding: Rounding) -> Result<TimePosition, RenderBlocker> {
    let ticks = seconds * TICKS_PER_SECOND as f64;
    if !ticks.is_finite() || ticks < 0.0 || ticks >= u64::MAX as f64 {
        return Err(RenderBlocker(
            "time must be finite, nonnegative and representable".into(),
        ));
    }
    Ok(match rounding {
        Rounding::Floor => ticks.floor(),
        Rounding::Ceil => ticks.ceil(),
    } as u64)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceTimebase {
    pub sample_rate: u32,
    pub frame_count: u64,
}

fn scale(
    value: u64,
    numerator: u64,
    denominator: u64,
    rounding: Rounding,
) -> Result<u64, RenderBlocker> {
    if denominator == 0 {
        return Err(RenderBlocker("sample rate must be nonzero".into()));
    }
    let product = u128::from(value) * u128::from(numerator);
    let divisor = u128::from(denominator);
    let result =
        product / divisor + u128::from(rounding == Rounding::Ceil && product % divisor != 0);
    u64::try_from(result).map_err(|_| RenderBlocker("time conversion overflow".into()))
}

impl SourceTimebase {
    pub fn new(sample_rate: u32, frame_count: u64) -> Result<Self, RenderBlocker> {
        let grid = Self {
            sample_rate,
            frame_count,
        };
        grid.duration()?;
        Ok(grid)
    }
    pub fn duration(self) -> Result<TimePosition, RenderBlocker> {
        scale(
            self.frame_count,
            TICKS_PER_SECOND,
            u64::from(self.sample_rate),
            Rounding::Ceil,
        )
    }
    pub fn time_at_frame(
        self,
        frame: u64,
        rounding: Rounding,
    ) -> Result<TimePosition, RenderBlocker> {
        if frame > self.frame_count {
            return Err(RenderBlocker("frame is outside source".into()));
        }
        scale(
            frame,
            TICKS_PER_SECOND,
            u64::from(self.sample_rate),
            rounding,
        )
    }
    pub fn frame_at_time(
        self,
        time: TimePosition,
        rounding: Rounding,
    ) -> Result<u64, RenderBlocker> {
        if time > self.duration()? {
            return Err(RenderBlocker("time is outside source".into()));
        }
        Ok(scale(
            time,
            u64::from(self.sample_rate),
            TICKS_PER_SECOND,
            rounding,
        )?
        .min(self.frame_count))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: TimePosition,
    pub end: TimePosition,
}
impl TimeRange {
    pub fn new(start: TimePosition, end: TimePosition) -> Result<Self, RenderBlocker> {
        if start >= end {
            return Err(RenderBlocker(
                "time range must be nonempty and half-open".into(),
            ));
        }
        Ok(Self { start, end })
    }
    pub fn native_frames(self, grid: SourceTimebase) -> Result<Range<usize>, RenderBlocker> {
        Self::new(self.start, self.end)?;
        let start = grid.frame_at_time(self.start, Rounding::Floor)?;
        let end = grid.frame_at_time(self.end, Rounding::Ceil)?;
        Ok(
            usize::try_from(start).map_err(|_| RenderBlocker("source too large".into()))?
                ..usize::try_from(end).map_err(|_| RenderBlocker("source too large".into()))?,
        )
    }
}

pub(crate) fn detector_time(frame: usize) -> TimePosition {
    // PCM allocation already bounds frame indices to this platform's address space.
    SourceTimebase::new(DETECTOR_SAMPLE_RATE, frame as u64)
        .and_then(|grid| grid.duration())
        .expect("validated detector frame count")
}
pub(crate) fn detector_frame(
    time: TimePosition,
    rounding: Rounding,
) -> Result<usize, RenderBlocker> {
    usize::try_from(scale(
        time,
        u64::from(DETECTOR_SAMPLE_RATE),
        TICKS_PER_SECOND,
        rounding,
    )?)
    .map_err(|_| RenderBlocker("detector coordinate overflow".into()))
}
