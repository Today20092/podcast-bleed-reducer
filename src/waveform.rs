use crate::{
    AnalysisPreview, Decision, RenderBlocker,
    audio_io::decode_chunks,
    timebase::{Rounding, SourceTimebase, TICKS_PER_SECOND, TimePosition},
    timeline::{TimelineEvent, TimelineViewport},
};
use iced::{
    Color, Point, Rectangle, Renderer, Size, Theme, keyboard, mouse,
    widget::{Action, canvas},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

const BASE_BUCKET: usize = 256;
const WAVEFORM_CACHE_VERSION: u32 = 2;
const ATTENUATION_HATCH_SPACING: f32 = 20.0;

fn processed_peak((min, max): (f32, f32), (low, high): (f32, f32)) -> (f32, f32) {
    // ponytail: cached peaks lack sample positions, so these are conservative
    // bounds. Use source samples if exact processed extrema become necessary.
    ((min * low).min(min * high), (max * low).max(max * high))
}

fn attenuation_hatches(left: f32, right: f32, height: f32) -> Vec<(Point, Point)> {
    let first = ((left - height) / ATTENUATION_HATCH_SPACING).floor() * ATTENUATION_HATCH_SPACING;
    let mut base = first;
    let mut lines = Vec::new();
    while base < right {
        let start_x = base.max(left);
        let end_x = (base + height).min(right);
        if end_x > start_x {
            lines.push((
                Point::new(start_x, height - (start_x - base)),
                Point::new(end_x, height - (end_x - base)),
            ));
        }
        base += ATTENUATION_HATCH_SPACING;
    }
    lines
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WaveformCache {
    pub sample_count: usize,
    pub sample_rate: u32,
    levels: Vec<WaveformLevel>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct WaveformLevel {
    samples_per_bucket: usize,
    peaks: Vec<(f32, f32)>,
}

#[derive(Deserialize, Serialize)]
struct PersistedWaveforms {
    version: u32,
    sources: Vec<SourceStamp>,
    waveforms: Vec<WaveformCache>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceStamp {
    path: PathBuf,
    bytes: u64,
    modified_nanos: u128,
}

impl SourceStamp {
    pub fn read(path: &Path) -> Result<Self, RenderBlocker> {
        let metadata = fs::metadata(path)?;
        Ok(Self {
            path: path.into(),
            bytes: metadata.len(),
            modified_nanos: metadata
                .modified()?
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        })
    }
}

impl WaveformCache {
    #[cfg(test)]
    fn from_samples(samples: &[f32], sample_rate: u32) -> Self {
        Self::from_peaks(summarize(samples, BASE_BUCKET), samples.len(), sample_rate)
    }

    fn from_peaks(mut peaks: Vec<(f32, f32)>, sample_count: usize, sample_rate: u32) -> Self {
        let mut samples_per_bucket = BASE_BUCKET;
        let mut levels = vec![WaveformLevel {
            samples_per_bucket,
            peaks: peaks.clone(),
        }];
        while peaks.len() > 1 {
            peaks = peaks
                .chunks(2)
                .map(|pair| {
                    pair.iter().fold((1.0_f32, -1.0_f32), |(min, max), peak| {
                        (min.min(peak.0), max.max(peak.1))
                    })
                })
                .collect();
            samples_per_bucket *= 2;
            levels.push(WaveformLevel {
                samples_per_bucket,
                peaks: peaks.clone(),
            });
        }
        Self {
            sample_count,
            sample_rate,
            levels,
        }
    }

    pub fn duration(&self) -> TimePosition {
        self.time_at_frame(self.sample_count)
    }

    fn time_at_frame(&self, frame: usize) -> TimePosition {
        self.grid()
            .time_at_frame(frame as u64, Rounding::Ceil)
            .expect("validated waveform time")
    }

    fn grid(&self) -> SourceTimebase {
        SourceTimebase::new(self.sample_rate, self.sample_count as u64)
            .expect("validated waveform grid")
    }

    fn frame_at_time(&self, time: TimePosition, rounding: Rounding) -> usize {
        self.grid()
            .frame_at_time(time.min(self.duration()), rounding)
            .expect("bounded waveform time") as usize
    }

    fn level(&self, samples_per_pixel: usize) -> &WaveformLevel {
        self.levels
            .iter()
            .rev()
            .find(|level| level.samples_per_bucket <= samples_per_pixel.max(BASE_BUCKET))
            .unwrap_or(&self.levels[0])
    }

    fn visible_level(
        &self,
        viewport: &TimelineViewport,
        width: f32,
    ) -> (&WaveformLevel, std::ops::Range<usize>) {
        let start = self.frame_at_time(viewport.start().max(0.0) as u64, Rounding::Floor);
        let end = self.frame_at_time(
            (viewport.start() + viewport.visible_samples()).ceil() as u64,
            Rounding::Ceil,
        );
        let target = ((end - start) as f64 / width.max(1.0) as f64).ceil() as usize;
        let level = self.level(target);
        let first = (start / level.samples_per_bucket).min(level.peaks.len());
        let last = end
            .div_ceil(level.samples_per_bucket)
            .min(level.peaks.len());
        (level, first..last)
    }
}

#[cfg(test)]
fn summarize(samples: &[f32], bucket: usize) -> Vec<(f32, f32)> {
    samples
        .chunks(bucket)
        .map(|chunk| {
            chunk
                .iter()
                .fold((1.0_f32, -1.0_f32), |(min, max), sample| {
                    (min.min(*sample), max.max(*sample))
                })
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn load_or_build(
    project_path: &Path,
    sources: &[PathBuf],
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    load_or_build_progress(project_path, sources, |_, _| {})
}

pub(crate) fn load_or_build_progress(
    project_path: &Path,
    sources: &[PathBuf],
    mut ready: impl FnMut(usize, &WaveformCache),
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    let cache_path = project_path.with_extension("waveforms.json");
    let stamps = sources
        .iter()
        .map(|path| SourceStamp::read(path))
        .collect::<Result<Vec<_>, _>>()?;
    if let Ok(bytes) = fs::read(&cache_path)
        && let Ok(cache) = serde_json::from_slice::<PersistedWaveforms>(&bytes)
        && cache.version == WAVEFORM_CACHE_VERSION
        && cache.sources == stamps
        && cache.waveforms.iter().all(|waveform| {
            SourceTimebase::new(waveform.sample_rate, waveform.sample_count as u64)
                .and_then(|grid| grid.duration())
                .is_ok()
        })
    {
        for (index, waveform) in cache.waveforms.iter().enumerate() {
            ready(index, waveform);
        }
        return Ok(cache.waveforms);
    }
    let waveforms = sources
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let mut peaks = Vec::new();
            let mut current = (f32::INFINITY, f32::NEG_INFINITY);
            let mut used = 0;
            let decoded = decode_chunks(path, None, |_, samples| {
                for &sample in samples {
                    current.0 = current.0.min(sample);
                    current.1 = current.1.max(sample);
                    used += 1;
                    if used == BASE_BUCKET {
                        peaks.push(current);
                        current = (f32::INFINITY, f32::NEG_INFINITY);
                        used = 0;
                    }
                }
                Ok(())
            })?;
            if used != 0 {
                peaks.push(current);
            }
            let waveform = WaveformCache::from_peaks(
                peaks,
                decoded.metadata.grid.frame_count as usize,
                decoded.metadata.grid.sample_rate,
            );
            ready(index, &waveform);
            Ok(waveform)
        })
        .collect::<Result<Vec<_>, RenderBlocker>>()?;
    let finished_stamps = sources
        .iter()
        .map(|path| SourceStamp::read(path))
        .collect::<Result<Vec<_>, _>>()?;
    if finished_stamps != stamps {
        return Err(RenderBlocker(
            "Source changed while loading waveforms; import again.".into(),
        ));
    }
    fs::write(
        cache_path,
        serde_json::to_vec(&PersistedWaveforms {
            version: WAVEFORM_CACHE_VERSION,
            sources: stamps,
            waveforms: waveforms.clone(),
        })
        .map_err(|error| RenderBlocker(error.to_string()))?,
    )?;
    Ok(waveforms)
}

pub(crate) struct TimelineCanvas<'a, Message> {
    cache: Option<&'a WaveformCache>,
    viewport: &'a TimelineViewport,
    ruler: bool,
    preview: Option<(&'a AnalysisPreview, usize)>,
    overview: Option<&'a crate::OwnershipOverview>,
    selecting: bool,
    on_event: fn(TimelineEvent) -> Message,
}

impl<'a, Message> TimelineCanvas<'a, Message> {
    pub fn waveform(
        cache: &'a WaveformCache,
        viewport: &'a TimelineViewport,
        on_event: fn(TimelineEvent) -> Message,
        preview: Option<(&'a AnalysisPreview, usize)>,
        selecting: bool,
    ) -> Self {
        Self {
            cache: Some(cache),
            viewport,
            ruler: false,
            preview,
            overview: None,
            selecting,
            on_event,
        }
    }

    pub fn ruler(
        viewport: &'a TimelineViewport,
        selecting: bool,
        on_event: fn(TimelineEvent) -> Message,
    ) -> Self {
        Self {
            cache: None,
            viewport,
            ruler: true,
            preview: None,
            overview: None,
            selecting,
            on_event,
        }
    }
}

impl<'a, Message> TimelineCanvas<'a, Message> {
    pub fn with_overview(mut self, overview: Option<&'a crate::OwnershipOverview>) -> Self {
        self.overview = overview;
        self
    }
}

#[derive(Default)]
pub(crate) struct TimelineCanvasState {
    dragging: bool,
    selection_anchor: Option<TimePosition>,
    pan_x: Option<f32>,
    modifiers: keyboard::Modifiers,
}

impl<Message: 'static> canvas::Program<Message> for TimelineCanvas<'_, Message> {
    type State = TimelineCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let position = cursor.position_in(bounds);
        match event {
            canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.modifiers = *modifiers;
                None
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                state.pan_x = Some(position?.x);
                Some(Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let position = position?;
                state.dragging = true;
                let sample = self.viewport.sample_at(position.x, bounds.width);
                if self.selecting {
                    state.selection_anchor = Some(sample);
                    return Some(
                        Action::publish((self.on_event)(TimelineEvent::Select(sample, sample)))
                            .and_capture(),
                    );
                }
                Some(Action::publish((self.on_event)(TimelineEvent::Seek(sample))).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.pan_x.is_some() => {
                let x = cursor.position_from(bounds.position())?.x;
                let delta_x = x - state.pan_x.replace(x)?;
                Some(
                    Action::publish((self.on_event)(TimelineEvent::Pan {
                        delta_x,
                        width: bounds.width,
                    }))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                let position = cursor.position_from(bounds.position())?;
                if let Some(anchor) = state.selection_anchor {
                    return Some(
                        Action::publish((self.on_event)(TimelineEvent::Select(
                            anchor,
                            self.viewport.sample_at(position.x, bounds.width),
                        )))
                        .and_capture(),
                    );
                }
                Some(
                    Action::publish((self.on_event)(TimelineEvent::Drag(
                        self.viewport.sample_at(position.x, bounds.width),
                    )))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging =>
            {
                state.dragging = false;
                let position = cursor.position_from(bounds.position())?;
                if let Some(anchor) = state.selection_anchor.take() {
                    return Some(
                        Action::publish((self.on_event)(TimelineEvent::Select(
                            anchor,
                            self.viewport.sample_at(position.x, bounds.width),
                        )))
                        .and_capture(),
                    );
                }
                Some(
                    Action::publish((self.on_event)(TimelineEvent::DragEnd(
                        self.viewport.sample_at(position.x, bounds.width),
                    )))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                state.pan_x = None;
                Some(Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                state.dragging = false;
                state.selection_anchor = None;
                state.pan_x = None;
                None
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let position = position?;
                let direction = match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                        *y
                    }
                };
                let event = if state.modifiers.control() {
                    TimelineEvent::Pan {
                        delta_x: match delta {
                            mouse::ScrollDelta::Lines { .. } => direction * 40.0,
                            mouse::ScrollDelta::Pixels { .. } => direction,
                        },
                        width: bounds.width,
                    }
                } else {
                    TimelineEvent::Zoom {
                        cursor_x: position.x,
                        width: bounds.width,
                        direction,
                    }
                };
                Some(Action::publish((self.on_event)(event)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = bounds.height / 2.0;
        if let Some((start, end)) = self.viewport.selection() {
            let left = self.viewport.x_for_sample(start, bounds.width).max(0.0);
            let right = self
                .viewport
                .x_for_sample(end, bounds.width)
                .min(bounds.width);
            if right > left {
                frame.fill(
                    &canvas::Path::rectangle(
                        Point::new(left, 0.0),
                        Size::new(right - left, bounds.height),
                    ),
                    Color::from_rgba8(70, 120, 220, 0.18),
                );
            }
        }
        if self.ruler {
            for tick in 0..=5 {
                let x = bounds.width * tick as f32 / 5.0;
                let sample = self.viewport.sample_at(x, bounds.width);
                frame.fill_text(canvas::Text {
                    content: format!("{:.1}s", sample as f64 / TICKS_PER_SECOND as f64),
                    position: Point::new(x + 3.0, center + 4.0),
                    color: Color::from_rgb8(180, 185, 195),
                    size: 11.0.into(),
                    ..canvas::Text::default()
                });
            }
        } else if let Some(cache) = self.cache {
            let (level, visible) = cache.visible_level(self.viewport, bounds.width);
            let first = visible.start;
            let last = visible.end;
            let scale = center * 0.9;
            if let Some((preview, track)) = self.preview {
                let summarized;
                let visible_regions = if let Some(overview) = self.overview {
                    summarized = overview
                        .visible_bands(track, self.viewport, bounds.width)
                        .into_iter()
                        .map(|band| crate::Region {
                            start: band.start,
                            end: band.end,
                            decision: match band.state {
                                crate::OverviewState::Owned => Decision::Owned(vec![track]),
                                crate::OverviewState::Absent => Decision::NoSpeech,
                                crate::OverviewState::MixedOrUncertain => Decision::Uncertain,
                            },
                        })
                        .collect::<Vec<_>>();
                    summarized.as_slice()
                } else {
                    let first = preview.regions.partition_point(|region| {
                        region.end <= self.viewport.start().max(0.0) as u64
                    });
                    let last = preview.regions.partition_point(|region| {
                        region.start
                            < (self.viewport.start() + self.viewport.visible_samples()).ceil()
                                as u64
                    });
                    &preview.regions[first..last]
                };
                for region in visible_regions {
                    let left = self
                        .viewport
                        .x_for_sample(region.start, bounds.width)
                        .max(0.0);
                    let right = self
                        .viewport
                        .x_for_sample(region.end, bounds.width)
                        .min(bounds.width);
                    if right <= left {
                        continue;
                    }
                    let (color, pattern) = match &region.decision {
                        Decision::Owned(owners) if owners.contains(&track) => {
                            (Color::from_rgba8(70, 190, 120, 0.14), 0)
                        }
                        Decision::Uncertain => (Color::from_rgba8(235, 180, 55, 0.14), 2),
                        _ => (Color::from_rgba8(220, 75, 85, 0.10), 1),
                    };
                    frame.fill(
                        &canvas::Path::rectangle(
                            Point::new(left, 0.0),
                            Size::new(right - left, bounds.height),
                        ),
                        color,
                    );
                    if pattern == 1 {
                        for (start, end) in attenuation_hatches(left, right, bounds.height) {
                            frame.stroke(
                                &canvas::Path::line(start, end),
                                canvas::Stroke::default()
                                    .with_width(0.5)
                                    .with_color(Color::from_rgba8(230, 230, 235, 0.16)),
                            );
                        }
                    } else if pattern == 2 {
                        let mut x = left + 4.0;
                        while x < right {
                            frame.fill(
                                &canvas::Path::circle(Point::new(x, 8.0), 1.5),
                                Color::from_rgb8(245, 195, 70),
                            );
                            x += 10.0;
                        }
                    }
                }
                for protection in &preview.protections {
                    if protection.track.is_none_or(|protected| protected == track) {
                        let left = self
                            .viewport
                            .x_for_sample(protection.start, bounds.width)
                            .max(0.0);
                        let right = self
                            .viewport
                            .x_for_sample(protection.end.min(cache.duration()), bounds.width)
                            .min(bounds.width);
                        if right > left {
                            frame.stroke(
                                &canvas::Path::rectangle(
                                    Point::new(left, 1.0),
                                    Size::new(right - left, bounds.height - 2.0),
                                ),
                                canvas::Stroke::default()
                                    .with_width(2.0)
                                    .with_color(Color::from_rgb8(90, 150, 245)),
                            );
                        }
                    }
                }
            }
            let original = canvas::Path::new(|builder| {
                for (index, (min, max)) in level.peaks[first..last].iter().enumerate() {
                    let sample = (first + index) * level.samples_per_bucket;
                    let x = self
                        .viewport
                        .x_for_sample(cache.time_at_frame(sample), bounds.width);
                    builder.move_to(Point::new(x, center - max * scale));
                    builder.line_to(Point::new(x, center - min * scale));
                }
            });
            frame.stroke(
                &original,
                canvas::Stroke::default()
                    .with_width(1.0)
                    .with_color(Color::from_rgba8(93, 211, 198, 0.35)),
            );
            if let Some((preview, track)) = self.preview
                && let Some(envelope) = preview.envelopes.get(track)
            {
                let processed = canvas::Path::new(|builder| {
                    for (index, (min, max)) in level.peaks[first..last].iter().enumerate() {
                        let sample = (first + index) * level.samples_per_bucket;
                        let Some(gains) = envelope.gain_bounds_time(
                            cache.time_at_frame(sample),
                            cache.time_at_frame(
                                sample
                                    .saturating_add(level.samples_per_bucket)
                                    .min(cache.sample_count),
                            ),
                        ) else {
                            continue;
                        };
                        let (min, max) = processed_peak((*min, *max), gains);
                        let x = self
                            .viewport
                            .x_for_sample(cache.time_at_frame(sample), bounds.width);
                        builder.move_to(Point::new(x, center - max * scale));
                        builder.line_to(Point::new(x, center - min * scale));
                    }
                });
                frame.stroke(
                    &processed,
                    canvas::Stroke::default()
                        .with_width(1.4)
                        .with_color(Color::from_rgb8(93, 211, 198)),
                );
                let gain_path = canvas::Path::new(|builder| {
                    for x in 0..bounds.width as usize {
                        let sample = self.viewport.sample_at(x as f32, bounds.width);
                        let y = 4.0 + (1.0 - envelope.gain_at_time(sample)) * 18.0;
                        if x == 0 {
                            builder.move_to(Point::new(0.0, y));
                        } else {
                            builder.line_to(Point::new(x as f32, y));
                        }
                    }
                });
                frame.stroke(
                    &gain_path,
                    canvas::Stroke::default()
                        .with_width(1.5)
                        .with_color(Color::from_rgb8(210, 130, 245)),
                );
            }
        }
        let playhead_x = self
            .viewport
            .x_for_sample(self.viewport.playhead(), bounds.width);
        if (0.0..=bounds.width).contains(&playhead_x) {
            frame.stroke(
                &canvas::Path::line(
                    Point::new(playhead_x, 0.0),
                    Point::new(playhead_x, bounds.height),
                ),
                canvas::Stroke::default()
                    .with_width(2.0)
                    .with_color(Color::WHITE),
            );
        }
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging || state.pan_x.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bwavfile::{WaveFmt, WaveWriter};

    #[test]
    fn processed_bucket_bounds_cover_changing_gain_for_both_sample_signs() {
        assert_eq!(processed_peak((-0.8, 0.6), (0.25, 1.0)), (-0.8, 0.6));
        assert_eq!(processed_peak((0.2, 0.8), (0.25, 1.0)), (0.05, 0.8));
        assert_eq!(processed_peak((-0.8, -0.2), (0.25, 1.0)), (-0.8, -0.05));
        assert_eq!(processed_peak((-0.8, 0.6), (0.5, 0.5)), (-0.4, 0.3));
        assert_eq!(processed_peak((0.0, 0.0), (0.25, 1.0)), (0.0, 0.0));
    }

    #[test]
    fn attenuation_hatches_keep_screen_phase_and_clip_to_region() {
        let wide = attenuation_hatches(17.0, 93.0, 40.0);
        let clipped = attenuation_hatches(31.0, 71.0, 40.0);

        assert!(wide.iter().chain(&clipped).all(|(start, end)| {
            start.x >= 17.0 && end.x <= 93.0 && start.y >= 0.0 && end.y <= 40.0
        }));
        assert_eq!(
            clipped,
            wide.into_iter()
                .filter_map(|(start, end)| {
                    let base = start.x + start.y - 40.0;
                    let start_x = start.x.max(31.0);
                    let end_x = end.x.min(71.0);
                    (end_x > start_x).then(|| {
                        (
                            Point::new(start_x, 40.0 - (start_x - base)),
                            Point::new(end_x, 40.0 - (end_x - base)),
                        )
                    })
                })
                .collect::<Vec<_>>()
        );
    }

    fn wav(path: &Path, samples: &[f32]) {
        let writer = WaveWriter::create(path, WaveFmt::new_pcm_mono(48_000, 16)).unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(samples).unwrap();
        frames.end().unwrap();
    }

    #[test]
    fn cache_preserves_peaks_at_close_and_full_session_levels() {
        let mut samples = vec![0.0; BASE_BUCKET * 8];
        samples[20] = -0.75;
        samples[21] = 0.9;
        let cache = WaveformCache::from_samples(&samples, 48_000);
        assert_eq!(cache.level(BASE_BUCKET).peaks[0], (-0.75, 0.9));
        let full = cache.level(samples.len());
        assert_eq!(full.peaks, vec![(-0.75, 0.9)]);
    }

    #[test]
    fn two_hour_visible_peak_drawing_work_is_bounded_at_mixed_native_rates() {
        let mut elapsed = std::time::Duration::ZERO;
        for rate in [44_100, 48_000, 96_000] {
            let samples = rate as usize * 7200;
            let cache = WaveformCache::from_peaks(
                vec![(-0.2, 0.4); samples.div_ceil(BASE_BUCKET)],
                samples,
                rate,
            );
            let mut viewport = TimelineViewport::new(cache.duration());
            let start = std::time::Instant::now();
            for _ in 0..100 {
                let (level, range) = cache.visible_level(&viewport, 1000.0);
                assert!(range.len() <= 2002);
                let peak = level.peaks[range]
                    .iter()
                    .fold((0.0_f32, 0.0_f32), |(lo, hi), &(min, max)| {
                        (lo.min(min), hi.max(max))
                    });
                assert_eq!(peak, (-0.2, 0.4));
                viewport.zoom_at(500.0, 1000.0, 1.0);
                viewport.pan_by_pixels(-30.0, 1000.0);
            }
            elapsed += start.elapsed();
        }
        eprintln!(
            "300 two-hour mixed-rate production visible peak queries/navigation: {elapsed:?}"
        );
    }

    #[test]
    fn persisted_cache_reloads_and_invalidates_with_its_source() {
        let root = std::env::temp_dir().join(format!("cleanslate-waveform-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let source = root.join("voice.wav");
        let project = root.join("session.cleanslate");
        wav(&source, &[0.0, 0.8, -0.6, 0.0]);
        let first = load_or_build(&project, std::slice::from_ref(&source)).unwrap();
        assert!(project.with_extension("waveforms.json").is_file());
        assert_eq!(
            load_or_build(&project, std::slice::from_ref(&source)).unwrap()[0].sample_count,
            4
        );
        let cache_path = project.with_extension("waveforms.json");
        let mut obsolete: serde_json::Value =
            serde_json::from_slice(&fs::read(&cache_path).unwrap()).unwrap();
        obsolete["version"] = 99.into();
        obsolete["waveforms"][0]["sample_rate"] = 96_000.into();
        fs::write(&cache_path, serde_json::to_vec(&obsolete).unwrap()).unwrap();
        assert_eq!(
            load_or_build(&project, std::slice::from_ref(&source)).unwrap()[0].sample_rate,
            48_000
        );
        fs::remove_file(&source).unwrap();
        wav(&source, &[0.0, 0.2]);
        let changed = load_or_build(&project, &[source]).unwrap();
        assert_eq!(first[0].sample_count, 4);
        assert_eq!(changed[0].sample_count, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "visual evidence uses the local canonical reference session"]
    fn canonical_reference_waveforms_are_detailed_and_aligned() {
        let sources = [
            PathBuf::from(r"C:\Users\User\Downloads\00005603_A01.wav"),
            PathBuf::from(r"C:\Users\User\Downloads\00005603_A02.wav"),
            PathBuf::from(r"C:\Users\User\Downloads\00005603_A03.wav"),
        ];
        assert!(sources.iter().all(|path| path.is_file()));
        let root = std::env::temp_dir().join("cleanslate-canonical-waveform-evidence");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let caches = load_or_build(&root.join("session.cleanslate"), &sources).unwrap();
        assert_eq!(caches.len(), sources.len());
        assert!(
            caches
                .iter()
                .all(|cache| cache.sample_count == caches[0].sample_count)
        );
        assert!(caches.iter().all(|cache| {
            cache
                .level(BASE_BUCKET)
                .peaks
                .iter()
                .any(|(min, max)| *min < 0.0 && *max > 0.0)
        }));
        fs::remove_dir_all(root).unwrap();
    }
}
