//! Cached ownership range summaries and bounded visible gain samples.
use crate::{
    EnvelopePoint, GainEnvelope, RenderBlocker, TemporalOwnership, TemporalTrackState,
    timeline::TimelineViewport,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverviewState {
    Owned,
    Absent,
    MixedOrUncertain,
}
#[derive(Clone, Copy, Debug)]
pub struct OverviewBand {
    pub start: u64,
    pub end: u64,
    pub state: OverviewState,
}
#[derive(Clone, Copy, Debug)]
pub struct GainDrawingBucket {
    pub start: u64,
    pub end: u64,
    pub lower: f32,
    pub upper: f32,
}
#[derive(Clone, Copy, Debug, Default)]
struct Masks {
    owned: u8,
    absent: u8,
    uncertain: u8,
}
impl Masks {
    fn combine(self, other: Self) -> Self {
        Self {
            owned: self.owned | other.owned,
            absent: self.absent | other.absent,
            uncertain: self.uncertain | other.uncertain,
        }
    }
}
#[derive(Clone, Debug)]
pub struct OwnershipOverview {
    ranges: Vec<(u64, u64)>,
    tree: Vec<Masks>,
    leaves: usize,
    tracks: usize,
}
impl OwnershipOverview {
    pub fn new(ownership: &TemporalOwnership) -> Result<Self, RenderBlocker> {
        Self::new_cancellable(ownership, None)
    }
    pub fn new_cancellable(
        ownership: &TemporalOwnership,
        cancel: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<Self, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancel)?;
        let regions = ownership.regions();
        if regions.is_empty() || ownership.tracks.is_empty() || ownership.tracks.len() > 8 {
            return Err(RenderBlocker(
                "Ownership drawing requires one to eight tracks and complete regions.".into(),
            ));
        }
        let leaves = regions
            .len()
            .checked_next_power_of_two()
            .ok_or_else(|| RenderBlocker("Ownership overview is too large.".into()))?;
        let size = leaves
            .checked_mul(2)
            .ok_or_else(|| RenderBlocker("Ownership overview is too large.".into()))?;
        let mut tree = vec![Masks::default(); size];
        let mut ranges = Vec::with_capacity(regions.len());
        let mut previous = 0;
        for (index, region) in regions.iter().enumerate() {
            if index % 1024 == 0 {
                crate::prepared_analysis::check_cancelled(cancel)?;
            }
            if region.start != previous
                || region.end <= region.start
                || region.states.len() != ownership.tracks.len()
            {
                return Err(RenderBlocker("Invalid ownership drawing intervals.".into()));
            }
            previous = region.end;
            ranges.push((region.start, region.end));
            let masks = &mut tree[leaves + index];
            for (track, state) in region.states.iter().enumerate() {
                let bit = 1_u8 << track;
                match state {
                    TemporalTrackState::Owned => masks.owned |= bit,
                    TemporalTrackState::Absent => masks.absent |= bit,
                    TemporalTrackState::UncertainUnity => masks.uncertain |= bit,
                }
            }
        }
        if previous != ownership.duration {
            return Err(RenderBlocker(
                "Ownership drawing does not cover the recording.".into(),
            ));
        }
        for index in (1..leaves).rev() {
            if index % 4096 == 0 {
                crate::prepared_analysis::check_cancelled(cancel)?;
            }
            tree[index] = tree[index * 2].combine(tree[index * 2 + 1]);
        }
        crate::prepared_analysis::check_cancelled(cancel)?;
        Ok(Self {
            ranges,
            tree,
            leaves,
            tracks: ownership.tracks.len(),
        })
    }
    fn query(&self, start: u64, end: u64) -> Masks {
        let mut left = self.ranges.partition_point(|range| range.1 <= start) + self.leaves;
        let mut right = self.ranges.partition_point(|range| range.0 < end) + self.leaves;
        let mut result = Masks::default();
        while left < right {
            if left % 2 == 1 {
                result = result.combine(self.tree[left]);
                left += 1;
            }
            if right % 2 == 1 {
                right -= 1;
                result = result.combine(self.tree[right]);
            }
            left /= 2;
            right /= 2;
        }
        result
    }
    pub fn visible_bands(
        &self,
        track: usize,
        viewport: &TimelineViewport,
        width: f32,
    ) -> Vec<OverviewBand> {
        if track >= self.tracks || !width.is_finite() || width <= 0.0 {
            return vec![];
        }
        let columns = (width.ceil() as usize).clamp(1, 4096);
        let duration = self.ranges.last().map_or(0, |range| range.1);
        let bit = 1_u8 << track;
        let mut bands: Vec<OverviewBand> = vec![];
        for column in 0..columns {
            let start = visible_boundary(viewport, width * column as f32 / columns as f32, width)
                .min(duration);
            let end = visible_boundary(
                viewport,
                width * (column + 1) as f32 / columns as f32,
                width,
            )
            .min(duration);
            if start >= end {
                continue;
            }
            let masks = self.query(start, end);
            let state = if masks.uncertain & bit != 0
                || masks.owned & bit != 0 && masks.absent & bit != 0
            {
                OverviewState::MixedOrUncertain
            } else if masks.owned & bit != 0 {
                OverviewState::Owned
            } else {
                OverviewState::Absent
            };
            if let Some(previous) = bands.last_mut()
                && previous.state == state
                && previous.end == start
            {
                previous.end = end;
            } else {
                bands.push(OverviewBand { start, end, state });
            }
        }
        bands
    }
}

pub fn visible_gain_curve(
    viewport: &TimelineViewport,
    width: f32,
    envelope: &GainEnvelope,
) -> Vec<EnvelopePoint> {
    sample_visible_curve(viewport, width, |time| envelope.gain_at_time(time))
}
pub fn visible_gain_buckets(
    viewport: &TimelineViewport,
    width: f32,
    envelope: &GainEnvelope,
) -> Vec<GainDrawingBucket> {
    visible_bounds(viewport, width, |start, end| {
        envelope.gain_bounds_time(start, end)
    })
}

pub(crate) fn attenuation_spans(buckets: &[GainDrawingBucket]) -> Vec<(u64, u64)> {
    let mut spans: Vec<(u64, u64)> = vec![];
    for bucket in buckets {
        if bucket.lower < 1.0 && bucket.start < bucket.end {
            if let Some(last) = spans.last_mut()
                && last.1 == bucket.start
            {
                last.1 = bucket.end;
            } else {
                spans.push((bucket.start, bucket.end));
            }
        }
    }
    spans
}

pub fn visible_automatic_buckets(
    viewport: &TimelineViewport,
    width: f32,
    envelope: &crate::AutomaticEnvelope,
) -> Vec<GainDrawingBucket> {
    visible_bounds(viewport, width, |start, end| {
        envelope.gain_bounds_time(start, end)
    })
}
fn visible_bounds(
    viewport: &TimelineViewport,
    width: f32,
    bounds: impl Fn(u64, u64) -> Option<(f32, f32)>,
) -> Vec<GainDrawingBucket> {
    if !width.is_finite() || width <= 0.0 {
        return vec![];
    }
    let columns = (width.ceil() as usize).clamp(1, 4096);
    (0..columns)
        .filter_map(|column| {
            let start = visible_boundary(viewport, width * column as f32 / columns as f32, width);
            let end = visible_boundary(
                viewport,
                width * (column + 1) as f32 / columns as f32,
                width,
            );
            bounds(start, end).map(|(lower, upper)| GainDrawingBucket {
                start,
                end,
                lower,
                upper,
            })
        })
        .collect()
}
pub(crate) fn sample_visible_curve(
    viewport: &TimelineViewport,
    width: f32,
    gain: impl Fn(u64) -> f32,
) -> Vec<EnvelopePoint> {
    if !width.is_finite() || width <= 0.0 {
        return vec![];
    }
    let columns = (width.ceil() as usize).clamp(1, 4096);
    (0..=columns)
        .map(|column| {
            let time = visible_boundary(viewport, width * column as f32 / columns as f32, width);
            EnvelopePoint {
                time,
                gain: gain(time),
            }
        })
        .collect()
}

pub(crate) fn visible_boundary(viewport: &TimelineViewport, x: f32, width: f32) -> u64 {
    if !x.is_finite() || !width.is_finite() || width <= 0.0 {
        return 0;
    }
    (viewport.start() + viewport.visible_samples() * f64::from(x / width))
        .floor()
        .clamp(0.0, viewport.total_samples() as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attenuation_shading_includes_fades_and_short_dips_but_leaves_unity_clear() {
        let buckets = [
            (0, 10, 1.0, 1.0),
            (10, 20, 0.9, 1.0),
            (20, 30, 0.25, 0.25),
            (30, 40, 1.0, 1.0),
            (40, 50, 0.5, 1.0),
            (50, 60, 1.0, 1.0),
            (60, 60, 0.25, 0.25),
        ]
        .map(|(start, end, lower, upper)| GainDrawingBucket {
            start,
            end,
            lower,
            upper,
        });
        assert_eq!(attenuation_spans(&buckets), vec![(10, 30), (40, 50)]);
        assert!(attenuation_spans(&[]).is_empty());
    }
}
