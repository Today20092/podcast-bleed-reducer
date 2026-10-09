use crate::{
    AnalysisPreview, RenderBlocker,
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
    cell::Cell,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

const BASE_BUCKET: usize = 256;
const WAVEFORM_CACHE_VERSION: u32 = 3;

fn processed_peak((min, max): (f32, f32), (low, high): (f32, f32)) -> (f32, f32) {
    // ponytail: cached peaks lack sample positions, so these are conservative
    // bounds. Use source samples if exact processed extrema become necessary.
    ((min * low).min(min * high), (max * low).max(max * high))
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
struct PersistedWaveform {
    version: u32,
    fingerprint: String,
    decoder_recipe: String,
    source: crate::SourceMetadata,
    waveform: WaveformCache,
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
    _project_path: &Path,
    sources: &[PathBuf],
    ready: impl FnMut(usize, &WaveformCache),
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    let cache_root = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("CleanSlate/waveforms-v3");
    load_or_build_progress_in(&cache_root, sources, ready)
}

fn load_or_build_progress_in(
    cache_root: &Path,
    sources: &[PathBuf],
    mut ready: impl FnMut(usize, &WaveformCache),
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    let stamps = sources
        .iter()
        .map(|path| SourceStamp::read(path))
        .collect::<Result<Vec<_>, _>>()?;
    let waveforms = sources
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let fingerprint = crate::project::fingerprint_cancellable(path, None)?;
            let source = crate::SourceMetadata::read(path)?;
            if source.channels != 1 {
                return Err(RenderBlocker("Waveforms require a mono source.".into()));
            }
            let cache_path = cache_root.join(format!("{fingerprint}.json"));
            if let Ok(bytes) = fs::read(&cache_path)
                && let Ok(cache) = serde_json::from_slice::<PersistedWaveform>(&bytes)
                && cache.version == WAVEFORM_CACHE_VERSION
                && cache.fingerprint == fingerprint
                && cache.decoder_recipe == crate::analysis_audio::AUDIO_DECODER_RECIPE
                && cache.source == source
                && waveform_matches_source(&cache.waveform, source)
            {
                ready(index, &cache.waveform);
                return Ok(cache.waveform);
            }
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
            if decoded.metadata != source || SourceStamp::read(path)? != stamps[index] {
                return Err(RenderBlocker(
                    "Source changed while loading waveforms; import again.".into(),
                ));
            }
            // Waveforms remain usable when the rebuildable cache cannot be saved.
            if let Ok(bytes) = serde_json::to_vec(&PersistedWaveform {
                version: WAVEFORM_CACHE_VERSION,
                fingerprint,
                decoder_recipe: crate::analysis_audio::AUDIO_DECODER_RECIPE.into(),
                source,
                waveform: waveform.clone(),
            }) && fs::create_dir_all(cache_root).is_ok()
            {
                let _ = crate::project::atomic_save(&cache_path, &bytes);
            }
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
    Ok(waveforms)
}

fn waveform_matches_source(waveform: &WaveformCache, source: crate::SourceMetadata) -> bool {
    if waveform.sample_rate != source.grid.sample_rate
        || waveform.sample_count as u64 != source.grid.frame_count
        || waveform.levels.is_empty()
    {
        return false;
    }
    let mut bucket = BASE_BUCKET;
    let mut count = waveform.sample_count.div_ceil(bucket);
    for (index, level) in waveform.levels.iter().enumerate() {
        if level.samples_per_bucket != bucket
            || level.peaks.len() != count
            || level
                .peaks
                .iter()
                .any(|(min, max)| !min.is_finite() || !max.is_finite() || min > max)
        {
            return false;
        }
        if count <= 1 {
            return index + 1 == waveform.levels.len();
        }
        let Some(next) = bucket.checked_mul(2) else {
            return false;
        };
        bucket = next;
        count = count.div_ceil(2);
    }
    false
}

type SelectionCallback<'a, Message> = dyn Fn(TimePosition, TimePosition, Point) -> Message + 'a;

pub(crate) struct TimelineCanvas<'a, Message> {
    revision: u64,
    cache: Option<&'a WaveformCache>,
    viewport: &'a TimelineViewport,
    ruler: bool,
    preview: Option<(&'a AnalysisPreview, usize)>,
    selecting: bool,
    on_event: Box<dyn Fn(TimelineEvent) -> Message + 'a>,
    on_context: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_selection: Option<Box<SelectionCallback<'a, Message>>>,
}

impl<'a, Message: 'a> TimelineCanvas<'a, Message> {
    pub fn waveform(
        cache: &'a WaveformCache,
        viewport: &'a TimelineViewport,
        on_event: impl Fn(TimelineEvent) -> Message + 'a,
        preview: Option<(&'a AnalysisPreview, usize)>,
        selecting: bool,
    ) -> Self {
        Self {
            revision: 0,
            cache: Some(cache),
            viewport,
            ruler: false,
            preview,
            selecting,
            on_event: Box::new(on_event),
            on_context: None,
            on_selection: None,
        }
    }

    pub fn ruler(
        viewport: &'a TimelineViewport,
        selecting: bool,
        on_event: fn(TimelineEvent) -> Message,
    ) -> Self {
        Self {
            revision: 0,
            cache: None,
            viewport,
            ruler: true,
            preview: None,
            selecting,
            on_event: Box::new(on_event),
            on_context: None,
            on_selection: None,
        }
    }
}

impl<'a, Message> TimelineCanvas<'a, Message> {
    pub fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }
    pub fn with_selection(
        mut self,
        on_selection: impl Fn(TimePosition, TimePosition, Point) -> Message + 'a,
    ) -> Self {
        self.on_selection = Some(Box::new(on_selection));
        self
    }
    pub fn with_context(mut self, on_context: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_context = Some(Box::new(on_context));
        self
    }
}

#[derive(Default)]
pub(crate) struct TimelineCanvasState {
    geometry: canvas::Cache,
    revision: Cell<Option<u64>>,
    dragging: bool,
    selection_anchor: Option<TimePosition>,
    anchor_x: f32,
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
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                position?;
                Some(Action::publish((self.on_context.as_ref()?)(cursor.position()?)).and_capture())
            }
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
                state.anchor_x = position.x;
                let sample = self.viewport.sample_at(position.x, bounds.width);
                if self.selecting || !self.ruler {
                    state.selection_anchor = Some(sample);
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
                    if (position.x - state.anchor_x).abs() < 3.0 {
                        return Some(Action::capture());
                    }
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
                    let end = self.viewport.sample_at(position.x, bounds.width);
                    if (position.x - state.anchor_x).abs() < 3.0 || anchor == end {
                        return Some(
                            Action::publish((self.on_event)(TimelineEvent::Seek(end)))
                                .and_capture(),
                        );
                    }
                    if let Some(on_selection) = &self.on_selection {
                        return Some(
                            Action::publish(on_selection(
                                anchor.min(end),
                                anchor.max(end),
                                cursor.position()?,
                            ))
                            .and_capture(),
                        );
                    }
                    return Some(
                        Action::publish((self.on_event)(TimelineEvent::Select(anchor, end)))
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
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        if state.revision.replace(Some(self.revision)) != Some(self.revision) {
            state.geometry.clear();
        }
        let geometry = state.geometry.draw(renderer, bounds.size(), |frame| {
            let center = bounds.height / 2.0;
            if !self.ruler
                && let Some((preview, track)) = self.preview
                && let Some(envelope) = preview.envelopes.get(track)
            {
                let buckets = crate::visible_gain_buckets(self.viewport, bounds.width, envelope);
                for (start, end) in crate::editor_drawing::attenuation_spans(&buckets) {
                    let left = self.viewport.x_for_sample(start, bounds.width).max(0.0);
                    let right = self
                        .viewport
                        .x_for_sample(end, bounds.width)
                        .min(bounds.width);
                    frame.fill_rectangle(
                        Point::new(left, 0.0),
                        Size::new((right - left).max(0.0), bounds.height),
                        Color::from_rgba8(220, 65, 65, 0.24),
                    );
                }
            }
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
        });
        let mut frame = canvas::Frame::new(renderer, bounds.size());
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
        vec![geometry, frame.into_geometry()]
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
    fn plain_waveform_click_seeks_and_drag_release_opens_label_actions() {
        use canvas::Program;
        #[derive(Debug)]
        enum Input {
            Timeline(TimelineEvent),
            Context(Point),
            Selection(TimePosition, TimePosition, Point),
        }
        let cache = WaveformCache::from_samples(&[0.0; 48000], 48000);
        let viewport = TimelineViewport::new(TICKS_PER_SECOND);
        let waveform = TimelineCanvas::waveform(&cache, &viewport, Input::Timeline, None, false)
            .with_context(Input::Context)
            .with_selection(Input::Selection);
        let bounds = Rectangle::new(Point::new(10.0, 20.0), iced::Size::new(100.0, 80.0));
        let mut state = TimelineCanvasState::default();
        let click = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(35.0, 40.0)),
            )
            .unwrap()
            .into_inner();
        assert!(
            matches!(click.0, Some(Input::Timeline(TimelineEvent::Seek(a))) if a == TICKS_PER_SECOND/4)
        );
        let drag = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(85.0, 40.0),
                }),
                bounds,
                mouse::Cursor::Available(Point::new(85.0, 40.0)),
            )
            .unwrap()
            .into_inner();
        assert!(
            matches!(drag.0, Some(Input::Timeline(TimelineEvent::Select(a,b))) if a == TICKS_PER_SECOND/4 && b == TICKS_PER_SECOND*3/4)
        );
        let context = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(85.0, 40.0)),
            )
            .unwrap()
            .into_inner();
        assert!(
            matches!(context.0, Some(Input::Selection(start, end, point)) if start == TICKS_PER_SECOND/4 && end == TICKS_PER_SECOND*3/4 && point == Point::new(85.0,40.0))
        );
        assert_eq!(context.2, iced::event::Status::Captured);
        let context = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
                bounds,
                mouse::Cursor::Available(Point::new(85.0, 40.0)),
            )
            .unwrap()
            .into_inner();
        assert!(matches!(context.0, Some(Input::Context(point)) if point == Point::new(85.0,40.0)));
        assert_eq!(context.2, iced::event::Status::Captured);
        let _ = waveform.update(
            &mut state,
            &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(Point::new(35.0, 40.0)),
        );
        let short_drag = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(36.0, 40.0)),
            )
            .unwrap()
            .into_inner();
        assert!(matches!(
            short_drag.0,
            Some(Input::Timeline(TimelineEvent::Seek(_)))
        ));
        assert_eq!(short_drag.2, iced::event::Status::Captured);
    }

    #[test]
    fn processed_bucket_bounds_cover_changing_gain_for_both_sample_signs() {
        assert_eq!(processed_peak((-0.8, 0.6), (0.25, 1.0)), (-0.8, 0.6));
        assert_eq!(processed_peak((0.2, 0.8), (0.25, 1.0)), (0.05, 0.8));
        assert_eq!(processed_peak((-0.8, -0.2), (0.25, 1.0)), (-0.8, -0.05));
        assert_eq!(processed_peak((-0.8, 0.6), (0.5, 0.5)), (-0.4, 0.3));
        assert_eq!(processed_peak((0.0, 0.0), (0.25, 1.0)), (0.0, 0.0));
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
    fn waveform_cache_is_shared_across_project_names_and_track_order() {
        let root =
            std::env::temp_dir().join(format!("cleanslate-shared-waveform-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let first = root.join("first.wav");
        let second = root.join("second.wav");
        wav(&first, &[0.0, 0.8, -0.6, 0.0]);
        wav(&second, &[0.0, 0.2]);
        let project = root.join("first-project.cleanslate");
        let waveforms = load_or_build(&project, &[first.clone(), second.clone()]).unwrap();
        assert!(
            !project.with_extension("waveforms.json").exists(),
            "Project names must not create duplicate waveform sidecars"
        );
        let reordered =
            load_or_build(&root.join("second-project.cleanslate"), &[second, first]).unwrap();
        assert_eq!(waveforms[0].sample_count, reordered[1].sample_count);
        assert_eq!(waveforms[1].sample_count, reordered[0].sample_count);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn persisted_cache_reloads_and_invalidates_with_its_source() {
        let root = std::env::temp_dir().join(format!("cleanslate-waveform-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let source = root.join("voice.wav");
        let cache_root = root.join("cache");
        wav(&source, &[0.0, 0.8, -0.6, 0.0]);
        let first =
            load_or_build_progress_in(&cache_root, std::slice::from_ref(&source), |_, _| {})
                .unwrap();
        let cache_path = fs::read_dir(&cache_root)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let saved = fs::metadata(&cache_path).unwrap().modified().unwrap();
        assert_eq!(
            load_or_build_progress_in(&cache_root, std::slice::from_ref(&source), |_, _| {})
                .unwrap()[0]
                .sample_count,
            4
        );
        assert_eq!(
            fs::metadata(&cache_path).unwrap().modified().unwrap(),
            saved,
            "Warm reuse must not rebuild/rewrite the waveform"
        );
        let moved = root.join("moved.wav");
        fs::copy(&source, &moved).unwrap();
        let moved_cache = load_or_build_progress_in(&cache_root, &[moved], |_, _| {}).unwrap();
        assert_eq!(
            moved_cache[0].level(BASE_BUCKET).peaks,
            first[0].level(BASE_BUCKET).peaks
        );
        assert_eq!(
            fs::read_dir(&cache_root).unwrap().count(),
            1,
            "Identical moved audio shares one cache entry"
        );
        let mut obsolete: serde_json::Value =
            serde_json::from_slice(&fs::read(&cache_path).unwrap()).unwrap();
        obsolete["version"] = 99.into();
        obsolete["waveform"]["sample_rate"] = 96_000.into();
        fs::write(&cache_path, serde_json::to_vec(&obsolete).unwrap()).unwrap();
        assert_eq!(
            load_or_build_progress_in(&cache_root, std::slice::from_ref(&source), |_, _| {})
                .unwrap()[0]
                .sample_rate,
            48_000
        );
        fs::write(&cache_path, b"broken json").unwrap();
        assert_eq!(
            load_or_build_progress_in(&cache_root, std::slice::from_ref(&source), |_, _| {})
                .unwrap()[0]
                .sample_count,
            4
        );
        let mut malformed: serde_json::Value =
            serde_json::from_slice(&fs::read(&cache_path).unwrap()).unwrap();
        malformed["waveform"]["levels"] = serde_json::json!([]);
        fs::write(&cache_path, serde_json::to_vec(&malformed).unwrap()).unwrap();
        assert_eq!(
            load_or_build_progress_in(&cache_root, std::slice::from_ref(&source), |_, _| {})
                .unwrap()[0]
                .sample_count,
            4
        );
        fs::remove_file(&source).unwrap();
        wav(&source, &[0.0, 0.2]);
        let changed =
            load_or_build_progress_in(&cache_root, std::slice::from_ref(&source), |_, _| {})
                .unwrap();
        assert_eq!(first[0].sample_count, 4);
        assert_eq!(changed[0].sample_count, 2);
        assert_eq!(fs::read_dir(&cache_root).unwrap().count(), 2);
        let unavailable_cache = root.join("cache-is-a-file");
        fs::write(&unavailable_cache, b"unwritable cache directory").unwrap();
        assert_eq!(
            load_or_build_progress_in(&unavailable_cache, &[source], |_, _| {}).unwrap()[0]
                .sample_count,
            2
        );
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
