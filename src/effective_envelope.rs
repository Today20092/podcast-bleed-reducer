//! Sparse source-aware envelopes; shared by display, audition and delivery.
use crate::{
    RenderBlocker, Rounding, SourceTimebase, TemporalOwnership, TemporalTrackState, TrackId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub const EFFECTIVE_ENVELOPE_RECIPE: &str = "finite-exp4-source-time-halfopen-linear-manual-v1";
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GainFadeSettings {
    pub attenuation: f32,
    pub fade_in: u64,
    pub fade_out: u64,
}
impl Default for GainFadeSettings {
    fn default() -> Self {
        Self {
            attenuation: 0.1,
            fade_in: crate::TICKS_PER_SECOND / 20,
            fade_out: crate::TICKS_PER_SECOND / 2,
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
        let shape = (-4.0 * u).exp_m1() / (-4.0_f64).exp_m1();
        (f64::from(self.from) + (f64::from(self.to) - f64::from(self.from)) * shape) as f32
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
            || settings.fade_in == 0
            || settings.fade_out == 0
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
            if region.states[track] != TemporalTrackState::Absent {
                let (start, end) = rounded_range(grid, region.start, region.end.min(duration))?;
                if let Some(previous) = preserved.last_mut().filter(|previous| {
                    start <= previous.1
                        || start - previous.1 <= settings.fade_in + settings.fade_out
                }) {
                    previous.1 = previous.1.max(end);
                } else {
                    preserved.push((start, end));
                }
            }
        }
        if position != ownership.duration {
            return Err(RenderBlocker(
                "Ownership regions do not cover the recording tail.".into(),
            ));
        }
        if let Some(first) = preserved.first_mut()
            && first.0 < settings.fade_in + settings.fade_out
        {
            first.0 = 0;
        }
        if let Some(last) = preserved.last_mut()
            && duration - last.1 < settings.fade_out
        {
            last.1 = duration;
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
                append(
                    cursor,
                    cursor + settings.fade_out,
                    1.0,
                    settings.attenuation,
                );
                append(
                    cursor + settings.fade_out,
                    start - settings.fade_in,
                    settings.attenuation,
                    settings.attenuation,
                );
                append(start - settings.fade_in, start, settings.attenuation, 1.0);
            }
            append(start, end, 1.0, 1.0);
            cursor = end;
        }
        if cursor < duration {
            if duration - cursor >= settings.fade_out {
                append(
                    cursor,
                    cursor + settings.fade_out,
                    1.0,
                    settings.attenuation,
                );
                append(
                    cursor + settings.fade_out,
                    duration,
                    settings.attenuation,
                    settings.attenuation,
                );
            } else {
                append(cursor, duration, 1.0, 1.0);
                preserved.push((cursor, duration));
            }
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
    pub fn duration(&self) -> u64 {
        self.duration
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EffectiveTrackEnvelope {
    automatic: Arc<AutomaticEnvelope>,
    manual: Vec<crate::ManualGainEdit>,
    protections: Vec<(u64, u64)>,
    edges: Vec<u64>,
}
impl EffectiveTrackEnvelope {
    fn new(
        automatic: Arc<AutomaticEnvelope>,
        track: TrackId,
        grid: SourceTimebase,
        constraints: &crate::GainConstraints,
    ) -> Result<Self, RenderBlocker> {
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
        Ok(Self {
            automatic,
            manual,
            protections: united,
            edges,
        })
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
}
#[derive(Default)]
pub struct EffectiveEnvelopeCache {
    accepted: Option<Arc<EffectiveEnvelopes>>,
    builds: u64,
    automatic: Option<(String, Vec<Arc<AutomaticEnvelope>>)>,
    automatic_builds: u64,
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
        for (source, auto) in project.sources.iter().zip(&automatic) {
            crate::prepared_analysis::check_cancelled(cancelled)?;
            let effective = EffectiveTrackEnvelope::new(
                auto.clone(),
                source.track_id,
                source.metadata.grid,
                constraints,
            )?;
            envelopes.push(crate::GainEnvelope::from_effective(
                source.metadata.grid,
                Arc::new(effective),
            )?);
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
        });
        if install {
            if new_automatic {
                self.automatic_builds += 1;
            }
            self.automatic = Some((automatic_key, result.automatic.clone()));
            self.builds += 1;
            self.accepted = Some(result.clone());
        }
        Ok(result)
    }
}
