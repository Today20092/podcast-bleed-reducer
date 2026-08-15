use crate::{
    RenderBlocker,
    audio_io::Stem,
    timeline::{TimelineEvent, TimelineViewport},
};
use iced::{
    Color, Point, Rectangle, Renderer, Size, Theme, mouse,
    widget::{Action, canvas},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

const BASE_BUCKET: usize = 256;

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

#[derive(Deserialize, Eq, PartialEq, Serialize)]
struct SourceStamp {
    path: PathBuf,
    bytes: u64,
    modified_nanos: u128,
}

impl SourceStamp {
    fn read(path: &Path) -> Result<Self, RenderBlocker> {
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
    on_event: fn(TimelineEvent) -> Message,
}

impl<'a, Message> TimelineCanvas<'a, Message> {
    pub fn waveform(
        cache: &'a WaveformCache,
        viewport: &'a TimelineViewport,
        on_event: fn(TimelineEvent) -> Message,
    ) -> Self {
        Self {
            cache: Some(cache),
            viewport,
            ruler: false,
            on_event,
        }
    }

    pub fn ruler(viewport: &'a TimelineViewport, on_event: fn(TimelineEvent) -> Message) -> Self {
        Self {
            cache: None,
            viewport,
            ruler: true,
            on_event,
        }
    }
}

#[derive(Default)]
pub(crate) struct TimelineCanvasState {
    dragging: bool,
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
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let position = position?;
                state.dragging = true;
                Some(
                    Action::publish((self.on_event)(TimelineEvent::Seek(
                        self.viewport.sample_at(position.x, bounds.width),
                    )))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                let position = cursor.position_from(bounds.position())?;
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
                Some(
                    Action::publish((self.on_event)(TimelineEvent::DragEnd(
                        self.viewport.sample_at(position.x, bounds.width),
                    )))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let position = position?;
                let direction = match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                        *y
                    }
                };
                Some(
                    Action::publish((self.on_event)(TimelineEvent::Zoom {
                        cursor_x: position.x,
                        width: bounds.width,
                        direction,
                    }))
                    .and_capture(),
                )
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
            let path = canvas::Path::new(|builder| {
                for (index, (min, max)) in level.peaks[first..last].iter().enumerate() {
                    let sample = (first + index) * level.samples_per_bucket;
                    let x = self.viewport.x_for_sample(sample, bounds.width);
                    builder.move_to(Point::new(x, center - max * scale));
                    builder.line_to(Point::new(x, center - min * scale));
                }
            });
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_width(1.0)
                    .with_color(Color::from_rgb8(93, 211, 198)),
            );
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
        if state.dragging {
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
