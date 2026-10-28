//! Project time is independent of any audio grid. Required rates have exact ticks;
//! other rates round preserved edges outward by less than one tick/sample.
use crate::RenderBlocker;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Physical project time in ticks, independent of source or analysis sample rates.
/// One second is [`TICKS_PER_SECOND`]; this value is not a sample/frame index.
pub type TimePosition = u64;
/// Project ticks per second; 44.1, 48 and 96 kHz frame boundaries are exact on this grid.
pub const TICKS_PER_SECOND: u64 = 28_224_000;
pub(crate) const DETECTOR_SAMPLE_RATE: u32 = 48_000;

/// Direction used when a physical time is not exactly representable on the target grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    /// Choose the preceding grid boundary, including an exactly matching boundary.
    Floor,
    /// Choose the following grid boundary, including an exactly matching boundary.
    Ceil,
}

/// Convert finite, nonnegative seconds to project ticks with the requested rounding.
/// Returns an error if the scaled time is nonfinite, negative or unrepresentable.
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

/// A source's frame grid, where one frame is one sample time, not interleaved channels.
/// Use [`Self::new`] to validate the nonzero rate and full duration; public fields or
/// deserialization alone do not establish validity. Methods check their requested conversions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceTimebase {
    /// Native frames per second; conversions reject zero.
    pub sample_rate: u32,
    /// Number of frames; `frame_count` itself denotes the exclusive end boundary.
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
    /// Validate a nonzero frame rate and a duration representable in project ticks.
    /// An empty source (`frame_count == 0`) is permitted.
    pub fn new(sample_rate: u32, frame_count: u64) -> Result<Self, RenderBlocker> {
        let grid = Self {
            sample_rate,
            frame_count,
        };
        grid.duration()?;
        Ok(grid)
    }
    /// Exclusive source end in project ticks, rounded upward if necessary.
    pub fn duration(self) -> Result<TimePosition, RenderBlocker> {
        scale(
            self.frame_count,
            TICKS_PER_SECOND,
            u64::from(self.sample_rate),
            Rounding::Ceil,
        )
    }
    /// Convert a source frame boundary to project ticks using the requested rounding.
    /// `frame == frame_count` is permitted as the exclusive end; larger indices fail.
    /// Invalid rates or unrepresentable conversions also return an error.
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
    /// Convert project ticks to a source frame boundary using the requested rounding.
    /// Times through [`Self::duration`] are accepted; the rounded result is capped at
    /// `frame_count`, including the exclusive end. Later times or invalid grids fail.
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

/// A nonempty half-open project-time interval `[start, end)` in ticks.
/// [`Self::new`] checks ordering; source bounds are checked by [`Self::native_frames`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: TimePosition,
    pub end: TimePosition,
}
impl TimeRange {
    /// Require `start < end`; neither endpoint is tied to a particular source yet.
    pub fn new(start: TimePosition, end: TimePosition) -> Result<Self, RenderBlocker> {
        if start >= end {
            return Err(RenderBlocker(
                "time range must be nonempty and half-open".into(),
            ));
        }
        Ok(Self { start, end })
    }
    /// Cover this interval with a half-open native-frame range, rounding start down
    /// and end up so preserved edges are not shortened. Reject invalid ordering,
    /// endpoints beyond the source, invalid grids or indices unrepresentable as `usize`.
    ///
    /// For example, at 96 kHz a frame spans 294 ticks: `[295, 587)` becomes `1..2`.
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
