use crate::{
    AnalysisPreview, Decision, RenderBlocker,
    audio_io::Stem,
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
const ATTENUATION_HATCH_SPACING: f32 = 20.0;

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
    levels: Vec<WaveformLevel>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct WaveformLevel {
    samples_per_bucket: usize,
    peaks: Vec<(f32, f32)>,
}

#[derive(Deserialize, Serialize)]
struct PersistedWaveforms {
    sources: Vec<SourceStamp>,
    waveforms: Vec<WaveformCache>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SourceStamp {
    path: PathBuf,
    bytes: u64,
    modified_nanos: u128,
}

impl SourceStamp {
    pub(crate) fn read(path: &Path) -> Result<Self, RenderBlocker> {
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
    fn from_samples(samples: &[f32]) -> Self {
        let mut samples_per_bucket = BASE_BUCKET;
        let mut peaks = summarize(samples, samples_per_bucket);
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
            sample_count: samples.len(),
            levels,
        }
    }

    fn level(&self, samples_per_pixel: usize) -> &WaveformLevel {
        self.levels
            .iter()
            .rev()
            .find(|level| level.samples_per_bucket <= samples_per_pixel.max(BASE_BUCKET))
            .unwrap_or(&self.levels[0])
    }
}

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

pub(crate) fn load_or_build(
    project_path: &Path,
    sources: &[PathBuf],
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    let cache_path = project_path.with_extension("waveforms.json");
    let stamps = sources
        .iter()
        .map(|path| SourceStamp::read(path))
        .collect::<Result<Vec<_>, _>>()?;
    if let Ok(bytes) = fs::read(&cache_path)
        && let Ok(cache) = serde_json::from_slice::<PersistedWaveforms>(&bytes)
        && cache.sources == stamps
    {
        return Ok(cache.waveforms);
    }
    let waveforms = sources
        .iter()
        .map(|path| Stem::read(path).map(|stem| WaveformCache::from_samples(&stem.samples)))
        .collect::<Result<Vec<_>, _>>()?;
    fs::write(
        cache_path,
        serde_json::to_vec(&PersistedWaveforms {
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
            selecting,
            on_event,
        }
    }
}

#[derive(Default)]
pub(crate) struct TimelineCanvasState {
    dragging: bool,
    selection_anchor: Option<usize>,
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
                    content: format!("{:.1}s", sample as f64 / 48_000.0),
                    position: Point::new(x + 3.0, center + 4.0),
                    color: Color::from_rgb8(180, 185, 195),
                    size: 11.0.into(),
                    ..canvas::Text::default()
                });
            }
        } else if let Some(cache) = self.cache {
            let target =
                (self.viewport.visible_samples() / bounds.width.max(1.0) as f64).ceil() as usize;
            let level = cache.level(target);
            let first =
                (self.viewport.start() as usize / level.samples_per_bucket).min(level.peaks.len());
            let last = ((self.viewport.start() + self.viewport.visible_samples()) as usize)
                .div_ceil(level.samples_per_bucket)
                .min(level.peaks.len());
            let scale = center * 0.9;
            if let Some((preview, track)) = self.preview {
                for region in &preview.regions {
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
                            .x_for_sample(protection.end.min(cache.sample_count), bounds.width)
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
                    let x = self.viewport.x_for_sample(sample, bounds.width);
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
                        let gain =
                            envelope.gain_at(sample.min(envelope.sample_count().saturating_sub(1)));
                        let x = self.viewport.x_for_sample(sample, bounds.width);
                        builder.move_to(Point::new(x, center - max * gain * scale));
                        builder.line_to(Point::new(x, center - min * gain * scale));
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
                        let y = 4.0
                            + (1.0
                                - envelope.gain_at(
                                    sample.min(envelope.sample_count().saturating_sub(1)),
                                ))
                                * 18.0;
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
        let cache = WaveformCache::from_samples(&samples);
        assert_eq!(cache.level(BASE_BUCKET).peaks[0], (-0.75, 0.9));
        let full = cache.level(samples.len());
        assert_eq!(full.peaks, vec![(-0.75, 0.9)]);
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
