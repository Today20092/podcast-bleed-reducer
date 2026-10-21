//! Sparse source-aware envelopes; shared by display, audition and delivery.
use crate::{
    RenderBlocker, Rounding, SourceTimebase, TemporalOwnership, TemporalTrackState, TrackId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub const EFFECTIVE_ENVELOPE_RECIPE: &str =
    "finite-smootherstep5-zero-fades-mute-live-bounded-gap-source-time-halfopen-linear-manual-v5";

#[cfg(test)]
mod zoom_tests {
    use super::*;

    #[test]
    #[ignore = "timed production zoom-data workload; run with --ignored --nocapture"]
    fn three_track_zoom_drawing_budget() {
        let duration = crate::TICKS_PER_SECOND * 600;
        let automatic = Arc::new(AutomaticEnvelope {
            duration,
            segments: (0..1200)
                .map(|i| FadeSegment {
                    start: i * crate::TICKS_PER_SECOND / 2,
                    end: (i + 1) * crate::TICKS_PER_SECOND / 2,
                    from: if i % 2 == 0 { 1.0 } else { 0.1 },
                    to: if i % 2 == 0 { 0.1 } else { 1.0 },
                })
                .collect(),
            preserved: vec![],
        });
        let constraints = crate::GainConstraints {
            manual_edits: vec![],
            protections: vec![],
        };
        let envelopes: Vec<_> = [44100, 48000, 96000]
            .into_iter()
            .map(|rate| {
                let grid = SourceTimebase::new(rate, u64::from(rate) * 600).unwrap();
                crate::GainEnvelope::from_effective(
                    grid,
                    Arc::new(
                        EffectiveTrackEnvelope::new(
                            automatic.clone(),
                            TrackId(1),
                            grid,
                            &constraints,
                        )
                        .unwrap(),
                    ),
                )
                .unwrap()
            })
            .collect();
        let mut viewport = crate::timeline::TimelineViewport::new(duration);
        let mut times = vec![];
        let width = 1900.0;
        for i in 0..24 {
            viewport.zoom_at(950.0, width, if i < 12 { 1.0 } else { -1.0 });
            let start = std::time::Instant::now();
            for envelope in &envelopes {
                for _ in 0..3 {
                    std::hint::black_box(crate::visible_gain_buckets(&viewport, width, envelope));
                }
                std::hint::black_box(crate::visible_gain_curve(&viewport, width, envelope));
                std::hint::black_box(crate::editor_drawing::sample_visible_curve(
                    &viewport,
                    width,
                    |time| automatic.gain_at_time(time),
                ));
            }
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        times.sort_by(f64::total_cmp);
        let median = times[times.len() / 2];
        eprintln!(
            "Three-track 10-minute, 1900-column production zoom-data preparation: median {median:.2} ms, max {:.2} ms",
            times[times.len() - 1]
        );
        assert!(
            median < 16.7,
            "zoom data alone exceeds a 60 Hz frame budget: {median:.2} ms"
        );
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GainFadeSettings {
    pub attenuation: f32,
    pub fade_in: u64,
    pub fade_out: u64,
    #[serde(default = "default_minimum_gap")]
    pub minimum_gap: u64,
}
fn default_minimum_gap() -> u64 {
    crate::TICKS_PER_SECOND * 768 / 1000
}
impl Default for GainFadeSettings {
    fn default() -> Self {
        Self {
            attenuation: 0.1,
            fade_in: crate::TICKS_PER_SECOND / 20,
            fade_out: crate::TICKS_PER_SECOND / 2,
            minimum_gap: default_minimum_gap(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
struct FadeSegment {
    start: u64,
    end: u64,
    from: f32,
    to: f32,
}
impl FadeSegment {
    fn gain(&self, time: u64) -> f32 {
        if time <= self.start {
            return self.from;
        }
        if time >= self.end {
            return self.to;
        }
        if self.from == self.to {
            return self.from;
        }
        let u = (time - self.start) as f64 / (self.end - self.start) as f64;
        // Zero endpoint slope and curvature soften joins to constant gain and adjacent fades.
        let shape = (u * u * u * (u * (6.0 * u - 15.0) + 10.0)).clamp(0.0, 1.0);
        (f64::from(self.from) + (f64::from(self.to) - f64::from(self.from)) * shape) as f32
    }
}
#[cfg(test)]
mod fade_tests {
    use super::FadeSegment;

    #[test]
    fn s_fades_are_bounded_monotonic_and_flat_at_both_endpoints() {
        for (from, to) in [(1.0, 0.0), (0.0, 1.0), (1.0, 0.17), (0.17, 1.0)] {
            let fade = FadeSegment {
                start: 100,
                end: 1_000_100,
                from,
                to,
            };
            assert_eq!(fade.gain(0), from);
            assert_eq!(fade.gain(fade.start), from);
            assert_eq!(fade.gain(fade.end), to);
            assert_eq!(fade.gain(fade.end + 1), to);
            assert_eq!(fade.gain(fade.start + 500_000), (from + to) / 2.0);
            let mut previous = from;
            for offset in (0..=1_000_000).step_by(100) {
                let gain = fade.gain(fade.start + offset);
                assert!(gain.is_finite() && (from.min(to)..=from.max(to)).contains(&gain));
                assert!((gain - previous) * (to - from) >= 0.0);
                previous = gain;
            }
            let h = 1_000;
            let first = fade.gain(fade.start + h);
            let second = fade.gain(fade.start + 2 * h);
            let penultimate = fade.gain(fade.end - h);
            let before_penultimate = fade.gain(fade.end - 2 * h);
            assert!((first - from).abs() < 1e-7);
            assert!((to - penultimate).abs() < 1e-7);
            assert!((second - 2.0 * first + from).abs() < 2e-7);
            assert!((to - 2.0 * penultimate + before_penultimate).abs() < 2e-7);
            assert!(
                (fade.gain(fade.start + 250_000) - (from + (to - from) * 0.103_515_625)).abs()
                    < 1e-7
            );
        }
        let flat = FadeSegment {
            start: 100,
            end: 200,
            from: 0.17,
            to: 0.17,
        };
        assert_eq!(flat.gain(150), 0.17);
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct AutomaticEnvelope {
    duration: u64,
    segments: Vec<FadeSegment>,
    preserved: Vec<(u64, u64)>,
}
fn rounded_range(grid: SourceTimebase, start: u64, end: u64) -> Result<(u64, u64), RenderBlocker> {
    Ok((
        grid.time_at_frame(grid.frame_at_time(start, Rounding::Floor)?, Rounding::Floor)?,
        grid.time_at_frame(grid.frame_at_time(end, Rounding::Ceil)?, Rounding::Ceil)?
            .min(grid.duration()?),
    ))
}
impl AutomaticEnvelope {
    pub fn from_ownership(
        ownership: &TemporalOwnership,
        track: TrackId,
        grid: SourceTimebase,
        settings: GainFadeSettings,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Self, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let duration = grid.duration()?;
        if !settings.attenuation.is_finite()
            || !(0.0..=1.0).contains(&settings.attenuation)
            || settings.fade_in.checked_add(settings.fade_out).is_none()
            || duration == 0
            || duration > ownership.duration
        {
            return Err(RenderBlocker(
                "Invalid finite envelope settings or source duration.".into(),
            ));
        }
        let track = ownership
            .tracks
            .iter()
            .position(|id| *id == track)
            .ok_or_else(|| RenderBlocker("Unknown envelope track.".into()))?;
        let mut preserved: Vec<(u64, u64)> = vec![];
        let mut position = 0;
        let mut human_absence = false;
        let mut previous_preserved_end = 0;
        for (index, region) in ownership.regions().iter().enumerate() {
            if index % 1024 == 0 {
                crate::prepared_analysis::check_cancelled(cancelled)?;
            }
            if region.start != position
                || region.end <= region.start
                || region.end > ownership.duration
                || region.states.len() != ownership.tracks.len()
            {
                return Err(RenderBlocker(
                    "Ownership regions must completely cover the recording in order.".into(),
                ));
            }
            position = region.end;
            if region.start >= duration {
                continue;
            }
            if region.human_truth && region.states[track] == TemporalTrackState::Absent {
                human_absence = true;
            }
            if region.states[track] != TemporalTrackState::Absent {
                let (start, end) = rounded_range(grid, region.start, region.end.min(duration))?;
                if let Some(previous) = preserved.last_mut().filter(|previous| {
                    start <= previous.1
                        || start - previous.1 <= 1
                        || (!human_absence
                            && region.start.saturating_sub(previous_preserved_end)
                                < settings.minimum_gap)
                }) {
                    previous.1 = previous.1.max(end);
                } else {
                    preserved.push((start, end));
                }
                human_absence = false;
                previous_preserved_end = region.end;
            }
        }
        if position != ownership.duration {
            return Err(RenderBlocker(
                "Ownership regions do not cover the recording tail.".into(),
            ));
        }
        let mut segments = vec![];
        let mut append = |start, end, from, to| {
            if start < end {
                segments.push(FadeSegment {
                    start,
                    end,
                    from,
                    to,
                });
            }
        };
        let mut cursor = 0;
        for &(start, end) in &preserved {
            if cursor < start {
                let gap = start - cursor;
                if gap <= 1 {
                    append(cursor, start, 1.0, 1.0);
                    append(start, end, 1.0, 1.0);
                    cursor = end;
                    continue;
                }
                // Fit fades inside absence instead of preserving the entire short gap.
                let close = if gap < settings.fade_in + settings.fade_out {
                    ((u128::from(gap) * u128::from(settings.fade_out)
                        / u128::from(settings.fade_in + settings.fade_out))
                        as u64)
                        .min(gap)
                } else {
                    settings.fade_out
                };
                let open = settings.fade_in.min(gap - close);
                append(cursor, cursor + close, 1.0, settings.attenuation);
                append(
                    cursor + close,
                    start - open,
                    settings.attenuation,
                    settings.attenuation,
                );
                append(start - open, start, settings.attenuation, 1.0);
            }
            append(start, end, 1.0, 1.0);
            cursor = end;
        }
        if cursor < duration {
            let close = settings.fade_out.min(duration - cursor);
            append(cursor, cursor + close, 1.0, settings.attenuation);
            append(
                cursor + close,
                duration,
                settings.attenuation,
                settings.attenuation,
            );
        }
        crate::prepared_analysis::check_cancelled(cancelled)?;
        Ok(Self {
            duration,
            segments,
            preserved,
        })
    }
    pub fn gain_at_time(&self, time: u64) -> f32 {
        if time >= self.duration {
            return 1.0;
        }
        self.segments[self.segments.partition_point(|s| s.end <= time)].gain(time)
    }
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
    pub fn gain_bounds_time(&self, start: u64, end: u64) -> Option<(f32, f32)> {
        let end = end.min(self.duration);
        if start >= end {
            return None;
        }
        let mut bounds = (1.0_f32, 0.0_f32);
        let first = self
            .segments
            .partition_point(|segment| segment.end <= start);
        for segment in self.segments[first..]
            .iter()
            .take_while(|segment| segment.start < end)
        {
            for time in [start.max(segment.start), (end - 1).min(segment.end)] {
                let gain = segment.gain(time);
                bounds.0 = bounds.0.min(gain);
                bounds.1 = bounds.1.max(gain);
            }
        }
        Some(bounds)
    }
    pub fn duration(&self) -> u64 {
        self.duration
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EffectiveTrackEnvelope {
    automatic: Arc<AutomaticEnvelope>,
    grid: SourceTimebase,
    manual: Vec<crate::ManualGainEdit>,
    protections: Vec<(u64, u64)>,
    edges: Vec<u64>,
}
impl EffectiveTrackEnvelope {
    fn prepare(
        automatic: Arc<AutomaticEnvelope>,
        track: TrackId,
        grid: SourceTimebase,
        constraints: &crate::GainConstraints,
        previous: Option<&Arc<Self>>,
    ) -> Result<Arc<Self>, RenderBlocker> {
        let mut manual: Vec<_> = constraints
            .manual_edits
            .iter()
            .filter(|edit| edit.source.track == track)
            .cloned()
            .collect();
        manual.sort_by_key(|edit| edit.points[0].time);
        let mut protections = constraints
            .protections
            .iter()
            .filter(|p| {
                p.track.is_none_or(|id| id == track)
                    && p.sources.iter().any(|binding| binding.track == track)
            })
            .map(|p| rounded_range(grid, p.range.start, p.range.end))
            .collect::<Result<Vec<_>, _>>()?;
        protections.sort_unstable();
        let mut united: Vec<(u64, u64)> = vec![];
        for (start, end) in protections {
            if let Some(previous) = united.last_mut().filter(|previous| start <= previous.1) {
                previous.1 = previous.1.max(end);
            } else {
                united.push((start, end));
            }
        }
        // A draft on another track cannot change this track's dense edge list.
        // Compare all source-grid, automatic and constraint inputs before reuse.
        if let Some(previous) = previous
            && previous.grid == grid
            && Arc::ptr_eq(&previous.automatic, &automatic)
            && previous.manual == manual
            && previous.protections == united
        {
            return Ok(previous.clone());
        }
        let mut edges: Vec<_> = automatic
            .segments
            .iter()
            .flat_map(|s| [s.start, s.end])
            .chain(
                manual
                    .iter()
                    .flat_map(|edit| edit.points.iter().map(|p| p.time)),
            )
            .chain(united.iter().flat_map(|&(start, end)| [start, end]))
            .collect();
        edges.sort_unstable();
        edges.dedup();
        Ok(Arc::new(Self {
            automatic,
            grid,
            manual,
            protections: united,
            edges,
        }))
    }
    fn manual_gain(&self, time: u64) -> Option<f32> {
        let index = self
            .manual
            .partition_point(|edit| edit.points.last().unwrap().time <= time);
        let edit = self
            .manual
            .get(index)
            .filter(|edit| edit.points[0].time <= time)?;
        let next = edit.points.partition_point(|point| point.time <= time);
        let (left, right) = (&edit.points[next - 1], &edit.points[next]);
        if time == left.time {
            return Some(left.gain);
        }
        let u = (time - left.time) as f64 / (right.time - left.time) as f64;
        Some((f64::from(left.gain) + (f64::from(right.gain) - f64::from(left.gain)) * u) as f32)
    }
    pub(crate) fn gain_at_time(&self, time: u64) -> f32 {
        if time >= self.automatic.duration {
            return 1.0;
        }
        if let Some(gain) = self.manual_gain(time) {
            return gain;
        }
        let index = self.protections.partition_point(|range| range.1 <= time);
        if self
            .protections
            .get(index)
            .is_some_and(|range| range.0 <= time)
        {
            return 1.0;
        }
        self.automatic.gain_at_time(time)
    }
    pub(crate) fn gain_sequential(&self, time: u64, cursor: &mut usize) -> f32 {
        if time >= self.automatic.duration {
            return 1.0;
        }
        let segments = &self.automatic.segments;
        if *cursor >= segments.len() || time < segments[*cursor].start {
            *cursor = segments.partition_point(|segment| segment.end <= time);
        }
        while *cursor + 1 < segments.len() && segments[*cursor].end <= time {
            *cursor += 1;
        }
        if let Some(gain) = self.manual_gain(time) {
            return gain;
        }
        let index = self.protections.partition_point(|range| range.1 <= time);
        if self
            .protections
            .get(index)
            .is_some_and(|range| range.0 <= time)
        {
            return 1.0;
        }
        segments[*cursor].gain(time)
    }
    pub(crate) fn bounds(
        &self,
        grid: SourceTimebase,
        start: usize,
        end: usize,
    ) -> Option<(f32, f32)> {
        let end = end.min(grid.frame_count as usize);
        if start >= end {
            return None;
        }
        let time = |frame| grid.time_at_frame(frame as u64, Rounding::Floor).ok();
        let mut values = vec![
            self.gain_at_time(time(start)?),
            self.gain_at_time(time(end - 1)?),
        ];
        let start_time = time(start)?;
        let end_time = grid.time_at_frame(end as u64, Rounding::Ceil).ok()?;
        let first = self.edges.partition_point(|edge| *edge < start_time);
        for &edge in self.edges[first..]
            .iter()
            .take_while(|&&edge| edge <= end_time)
        {
            for rounding in [Rounding::Floor, Rounding::Ceil] {
                let frame = usize::try_from(grid.frame_at_time(edge, rounding).ok()?).ok()?;
                for candidate in [frame.saturating_sub(1), frame, frame.saturating_add(1)] {
                    if candidate >= start && candidate < end {
                        values.push(self.gain_at_time(time(candidate)?));
                    }
                }
            }
        }
        Some(
            values
                .into_iter()
                .fold((1.0_f32, 0.0_f32), |(min, max), gain| {
                    (min.min(gain), max.max(gain))
                }),
        )
    }
}
#[derive(Clone, Debug)]
pub struct EffectiveEnvelopes {
    pub cache_key: String,
    pub analysis_current: bool,
    pub envelopes: Vec<crate::GainEnvelope>,
    pub automatic: Vec<Arc<AutomaticEnvelope>>,
    /// Tracks whose complete effective preparation was retained from the preceding result.
    pub reused_tracks: usize,
}
#[derive(Clone, Default)]
pub struct EffectiveEnvelopeCache {
    accepted: Option<Arc<EffectiveEnvelopes>>,
    builds: u64,
    automatic: Option<(String, Vec<Arc<AutomaticEnvelope>>)>,
    automatic_builds: u64,
    effective: Vec<Arc<EffectiveTrackEnvelope>>,
}
impl EffectiveEnvelopeCache {
    pub fn last_completed(&self) -> Option<&Arc<EffectiveEnvelopes>> {
        self.accepted.as_ref()
    }
    pub fn builds(&self) -> u64 {
        self.builds
    }
    pub fn automatic_builds(&self) -> u64 {
        self.automatic_builds
    }
    pub fn update(
        &mut self,
        project: &crate::Project,
        ownership: &TemporalOwnership,
        settings: GainFadeSettings,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Arc<EffectiveEnvelopes>, RenderBlocker> {
        self.build(
            project,
            ownership,
            settings,
            &project.gain_constraints(),
            cancelled,
            true,
        )
    }
    pub fn preview_edit(
        &self,
        project: &crate::Project,
        ownership: &TemporalOwnership,
        settings: GainFadeSettings,
        draft: &crate::GainEditDraft,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Arc<EffectiveEnvelopes>, RenderBlocker> {
        project
            .validate_gain_edit(draft)
            .map_err(|e| RenderBlocker(e.to_string()))?;
        Self {
            automatic: self.automatic.clone(),
            effective: self.effective.clone(),
            ..Self::default()
        }
        .build(
            project,
            ownership,
            settings,
            &draft.constraints,
            cancelled,
            false,
        )
    }
    fn build(
        &mut self,
        project: &crate::Project,
        ownership: &TemporalOwnership,
        settings: GainFadeSettings,
        constraints: &crate::GainConstraints,
        cancelled: Option<&AtomicBool>,
        install: bool,
    ) -> Result<Arc<EffectiveEnvelopes>, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        project
            .require_alignment()
            .map_err(|e| RenderBlocker(e.to_string()))?;
        project
            .validate_interactive_sources()
            .map_err(|e| RenderBlocker(e.to_string()))?;
        project
            .validate_gain_constraints(constraints)
            .map_err(|e| RenderBlocker(e.to_string()))?;
        if !ownership.sources_match(project) {
            return Err(RenderBlocker("Automatic ownership belongs to different source audio; Analyze current recordings.".into()));
        }
        let analysis_current = ownership.analysis_matches(project);
        let key = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(
                    EFFECTIVE_ENVELOPE_RECIPE,
                    &ownership.cache_key,
                    analysis_current,
                    settings,
                    constraints
                ))
                .map_err(|e| RenderBlocker(e.to_string()))?
            )
        );
        if let Some(old) = &self.accepted
            && old.cache_key == key
        {
            project
                .validate_interactive_sources()
                .map_err(|e| RenderBlocker(e.to_string()))?;
            crate::prepared_analysis::check_cancelled(cancelled)?;
            return Ok(old.clone());
        }
        let automatic_key = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(
                    EFFECTIVE_ENVELOPE_RECIPE,
                    &ownership.cache_key,
                    crate::SessionOwnership::source_identity(project),
                    settings
                ))
                .map_err(|e| RenderBlocker(e.to_string()))?
            )
        );
        let reused_automatic = self
            .automatic
            .as_ref()
            .filter(|(key, _)| key == &automatic_key);
        let automatic = if let Some((_, automatic)) = reused_automatic {
            automatic.clone()
        } else {
            project
                .sources
                .iter()
                .map(|source| {
                    AutomaticEnvelope::from_ownership(
                        ownership,
                        source.track_id,
                        source.metadata.grid,
                        settings,
                        cancelled,
                    )
                    .map(Arc::new)
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        let new_automatic = reused_automatic.is_none();
        let mut envelopes = vec![];
        let mut effective_tracks = vec![];
        let mut reused_tracks = 0;
        for (index, (source, auto)) in project.sources.iter().zip(&automatic).enumerate() {
            crate::prepared_analysis::check_cancelled(cancelled)?;
            let previous = self.effective.get(index);
            let effective = EffectiveTrackEnvelope::prepare(
                auto.clone(),
                source.track_id,
                source.metadata.grid,
                constraints,
                previous,
            )?;
            if previous.is_some_and(|old| Arc::ptr_eq(old, &effective)) {
                reused_tracks += 1;
            }
            envelopes.push(crate::GainEnvelope::from_effective(
                source.metadata.grid,
                effective.clone(),
            )?);
            effective_tracks.push(effective);
        }
        project
            .validate_interactive_sources()
            .map_err(|e| RenderBlocker(e.to_string()))?;
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let result = Arc::new(EffectiveEnvelopes {
            cache_key: key,
            analysis_current,
            envelopes,
            automatic,
            reused_tracks,
        });
        if install {
            if new_automatic {
                self.automatic_builds += 1;
            }
            self.automatic = Some((automatic_key, result.automatic.clone()));
            self.effective = effective_tracks;
            self.builds += 1;
            self.accepted = Some(result.clone());
        }
        Ok(result)
    }
}
