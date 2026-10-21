use crate::{
    AnalysisPreview, RenderBlocker,
    timebase::{Rounding, SourceTimebase, TICKS_PER_SECOND, TimePosition},
    timeline::{TimelineEvent, TimelineViewport},
};
use iced::{
    Point, Rectangle, Renderer, Size, Theme, keyboard, mouse,
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

#[cfg(test)]
mod multichannel_waveform_tests {
    use super::*;
    #[test]
    fn channel_peaks_and_persisted_cache_do_not_alias_shared_file_paths() {
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("poly.wav");
        crate::audio_io::multichannel_tests::write_poly(&path, 3, 32, 30_001);
        let mut inputs = crate::wav_channel_sources(&path).unwrap();
        let cache = root.join("peaks");
        let waveforms = load_source_waveforms_in(&cache, &inputs, |_, _| {}).unwrap();
        for (waveform, value) in waveforms.iter().zip([0.25, -0.5, 0.75]) {
            assert_eq!(waveform.sample_count, 30_001);
            assert!(
                waveform.levels[0]
                    .peaks
                    .iter()
                    .all(|peak| *peak == (value, value))
            );
        }
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 3);
        inputs.reverse();
        let cached = load_source_waveforms_in(&cache, &inputs, |_, _| {}).unwrap();
        assert_eq!(cached[0].levels[0].peaks, waveforms[2].levels[0].peaks);
        assert_eq!(cached[2].levels[0].peaks, waveforms[0].levels[0].peaks);
        fs::remove_dir_all(root).unwrap();
    }
}

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

pub(crate) struct WaveformBuilder {
    peaks: Vec<(f32, f32)>,
    current: (f32, f32),
    used: usize,
    sample_count: usize,
}

impl Default for WaveformBuilder {
    fn default() -> Self {
        Self {
            peaks: Vec::new(),
            current: (f32::INFINITY, f32::NEG_INFINITY),
            used: 0,
            sample_count: 0,
        }
    }
}

impl WaveformBuilder {
    pub(crate) fn push(&mut self, samples: &[f32]) {
        self.sample_count += samples.len();
        for &sample in samples {
            self.current.0 = self.current.0.min(sample);
            self.current.1 = self.current.1.max(sample);
            self.used += 1;
            if self.used == BASE_BUCKET {
                self.peaks.push(self.current);
                self.current = (f32::INFINITY, f32::NEG_INFINITY);
                self.used = 0;
            }
        }
    }

    pub(crate) fn refine_preview(&self, preview: &mut WaveformCache) {
        preview.refine_prefix(&self.peaks);
    }

    pub(crate) fn finish(mut self, sample_rate: u32) -> WaveformCache {
        if self.used != 0 {
            self.peaks.push(self.current);
        }
        WaveformCache::from_peaks(self.peaks, self.sample_count, sample_rate)
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    channel: Option<u16>,
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
            channel: None,
        })
    }

    pub(crate) fn read_source(source: &crate::AudioSource) -> Result<Self, RenderBlocker> {
        let mut stamp = Self::read(&source.path)?;
        stamp.channel = source.channel;
        Ok(stamp)
    }
}

impl WaveformCache {
    fn refine_prefix(&mut self, detailed: &[(f32, f32)]) {
        let level = &mut self.levels[0];
        let complete = detailed.len() * BASE_BUCKET / level.samples_per_bucket;
        for (index, peak) in level.peaks.iter_mut().take(complete).enumerate() {
            let first = index * level.samples_per_bucket / BASE_BUCKET;
            let last = ((index + 1) * level.samples_per_bucket)
                .div_ceil(BASE_BUCKET)
                .min(detailed.len());
            *peak = detailed[first..last].iter().fold(
                (f32::INFINITY, f32::NEG_INFINITY),
                |(min, max), &(low, high)| (min.min(low), max.max(high)),
            );
        }
    }
    pub(crate) fn skeleton(metadata: crate::SourceMetadata) -> Self {
        Self {
            sample_count: metadata.grid.frame_count as usize,
            sample_rate: metadata.grid.sample_rate,
            levels: vec![WaveformLevel {
                samples_per_bucket: BASE_BUCKET,
                peaks: vec![],
            }],
        }
    }

    pub(crate) fn coarse(
        peaks: Vec<(f32, f32)>,
        metadata: crate::SourceMetadata,
        bucket: usize,
    ) -> Self {
        Self {
            sample_count: metadata.grid.frame_count as usize,
            sample_rate: metadata.grid.sample_rate,
            levels: vec![WaveformLevel {
                samples_per_bucket: bucket,
                peaks,
            }],
        }
    }
    #[cfg(test)]
    fn from_samples(samples: &[f32], sample_rate: u32) -> Self {
        let mut builder = WaveformBuilder::default();
        builder.push(samples);
        builder.finish(sample_rate)
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

pub(crate) fn import_cache(
    fingerprint: &str,
    source: crate::SourceMetadata,
) -> Option<WaveformCache> {
    let bytes = fs::read(import_cache_root().join(format!("{fingerprint}.json"))).ok()?;
    let cache: PersistedWaveform = serde_json::from_slice(&bytes).ok()?;
    (cache.version == WAVEFORM_CACHE_VERSION
        && cache.fingerprint == fingerprint
        && cache.decoder_recipe == crate::analysis_audio::AUDIO_DECODER_RECIPE
        && cache.source == source
        && waveform_matches_source(&cache.waveform, source))
    .then_some(cache.waveform)
}

fn import_cache_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("CleanSlate/waveforms-v3")
}

pub(crate) fn save_import_cache(
    fingerprint: String,
    source: crate::SourceMetadata,
    waveform: &WaveformCache,
) {
    let root = import_cache_root();
    if let Ok(bytes) = serde_json::to_vec(&PersistedWaveform {
        version: WAVEFORM_CACHE_VERSION,
        fingerprint: fingerprint.clone(),
        decoder_recipe: crate::analysis_audio::AUDIO_DECODER_RECIPE.into(),
        source,
        waveform: waveform.clone(),
    }) && fs::create_dir_all(&root).is_ok()
    {
        let _ = crate::project::atomic_save(&root.join(format!("{fingerprint}.json")), &bytes);
    }
}

#[cfg(test)]
pub(crate) fn load_or_build(
    project_path: &Path,
    sources: &[PathBuf],
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    load_or_build_progress(project_path, sources, |_, _| {})
}

#[cfg(test)]
pub(crate) fn load_or_build_progress(
    _project_path: &Path,
    sources: &[PathBuf],
    ready: impl FnMut(usize, &WaveformCache),
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    load_source_waveforms(
        &sources
            .iter()
            .cloned()
            .map(crate::AudioSource::mono)
            .collect::<Vec<_>>(),
        ready,
    )
}

pub(crate) fn load_source_waveforms(
    sources: &[crate::AudioSource],
    ready: impl FnMut(usize, &WaveformCache),
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    let cache_root = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("CleanSlate/waveforms-v3");
    load_source_waveforms_in(&cache_root, sources, ready)
}

#[cfg(test)]
fn load_or_build_progress_in(
    cache_root: &Path,
    sources: &[PathBuf],
    ready: impl FnMut(usize, &WaveformCache),
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    load_source_waveforms_in(
        cache_root,
        &sources
            .iter()
            .cloned()
            .map(crate::AudioSource::mono)
            .collect::<Vec<_>>(),
        ready,
    )
}

fn load_source_waveforms_in(
    cache_root: &Path,
    sources: &[crate::AudioSource],
    mut ready: impl FnMut(usize, &WaveformCache),
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    let stamps = sources
        .iter()
        .map(SourceStamp::read_source)
        .collect::<Result<Vec<_>, _>>()?;
    let waveforms = sources
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let fingerprint = path.fingerprint(None)?;
            let source = path.metadata(None)?;
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
            let mut builder = WaveformBuilder::default();
            let decoded = crate::audio_io::decode_source_chunks(path, None, |_, samples| {
                builder.push(samples);
                Ok(())
            })?;
            let waveform = builder.finish(decoded.metadata.grid.sample_rate);
            if decoded.metadata != source || SourceStamp::read_source(path)? != stamps[index] {
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
        .map(SourceStamp::read_source)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExampleEvent {
    Select(u64),
    Resize(u64, crate::TimeRange, bool),
    CancelResize,
}

fn visible_selection_edges(viewport: &TimelineViewport, width: f32) -> [Option<f32>; 2] {
    let Some((start, end)) = viewport.selection().filter(|(start, end)| start < end) else {
        return [None, None];
    };
    if !width.is_finite() || width <= 0.0 {
        return [None, None];
    }
    [start, end].map(|time| {
        let x = viewport.x_for_sample(time, width);
        (0.0..=width).contains(&x).then_some(x)
    })
}

pub(crate) fn draw_selection_edges(
    frame: &mut canvas::Frame,
    viewport: &TimelineViewport,
    color: iced::Color,
    emphasized: bool,
) {
    for x in visible_selection_edges(viewport, frame.width())
        .into_iter()
        .flatten()
    {
        frame.stroke(
            &canvas::Path::line(Point::new(x, 0.0), Point::new(x, frame.height())),
            canvas::Stroke::default()
                .with_color(color)
                .with_width(if emphasized { 2.0 } else { 1.0 }),
        );
    }
}

pub(crate) struct TimelineCanvas<'a, Message> {
    revision: u64,
    cache: Option<&'a WaveformCache>,
    viewport: &'a TimelineViewport,
    ruler: bool,
    preview: Option<(&'a AnalysisPreview, usize)>,
    selecting: bool,
    editing_target: bool,
    on_event: Box<dyn Fn(TimelineEvent) -> Message + 'a>,
    on_context: Option<Box<dyn Fn(Point, u64) -> Message + 'a>>,
    on_selection: Option<Box<SelectionCallback<'a, Message>>>,
    examples: &'a [crate::ReferenceLabel],
    selected_example: Option<u64>,
    example_draft: Option<crate::TimeRange>,
    on_example: Option<Box<dyn Fn(ExampleEvent) -> Message + 'a>>,
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
            editing_target: false,
            on_event: Box::new(on_event),
            on_context: None,
            on_selection: None,
            examples: &[],
            selected_example: None,
            example_draft: None,
            on_example: None,
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
            editing_target: false,
            on_event: Box::new(on_event),
            on_context: None,
            on_selection: None,
            examples: &[],
            selected_example: None,
            example_draft: None,
            on_example: None,
        }
    }
}

impl<'a, Message> TimelineCanvas<'a, Message> {
    pub fn with_editing_target(mut self, editing_target: bool) -> Self {
        self.editing_target = editing_target;
        self
    }

    pub fn with_examples(
        mut self,
        examples: &'a [crate::ReferenceLabel],
        selected: Option<u64>,
        draft: Option<crate::TimeRange>,
        on_example: impl Fn(ExampleEvent) -> Message + 'a,
    ) -> Self {
        self.examples = examples;
        self.selected_example = selected;
        self.example_draft = draft;
        self.on_example = Some(Box::new(on_example));
        self
    }

    fn example_handle(&self, position: Point, width: f32) -> Option<(u64, crate::TimeRange, bool)> {
        let label = self
            .examples
            .iter()
            .find(|label| Some(label.id) == self.selected_example)?;
        let range = self.example_draft.unwrap_or(label.range);
        let left = self.viewport.x_for_sample(range.start, width);
        let right = self.viewport.x_for_sample(range.end, width);
        if (position.x - left).abs() <= 6.0
            && (position.x - left).abs() <= (position.x - right).abs()
        {
            Some((label.id, range, true))
        } else if (position.x - right).abs() <= 6.0 {
            Some((label.id, range, false))
        } else {
            None
        }
    }

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
    pub fn with_context(mut self, on_context: impl Fn(Point, u64) -> Message + 'a) -> Self {
        self.on_context = Some(Box::new(on_context));
        self
    }
}

#[derive(Default)]
pub(crate) struct TimelineCanvasState {
    appearance: Cell<Option<crate::appearance::Appearance>>,
    geometry: canvas::Cache,
    revision: Cell<Option<u64>>,
    dragging: bool,
    selection_anchor: Option<TimePosition>,
    anchor_x: f32,
    pan_x: Option<f32>,
    pan_epoch: u64,
    modifiers: keyboard::Modifiers,
    example_drag: Option<(u64, crate::TimeRange, bool)>,
    hovered_example_handle: Option<(u64, bool)>,
    zoom_drag: Option<ZoomDrag>,
}

pub(crate) struct ZoomDrag {
    anchor: TimePosition,
    anchor_x: f32,
    end_x: f32,
}

impl ZoomDrag {
    pub fn begin(viewport: &TimelineViewport, x: f32, width: f32) -> Self {
        Self {
            anchor: crate::editor_drawing::visible_boundary(viewport, x, width),
            anchor_x: x,
            end_x: x,
        }
    }

    pub fn update<Message>(
        drag: &mut Option<Self>,
        event: &canvas::Event,
        viewport: &TimelineViewport,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        on_fit: impl Fn(TimePosition, TimePosition) -> Message,
    ) -> Option<Action<Message>> {
        let current = drag.as_mut()?;
        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(point) = cursor.position_from(bounds.position()) {
                    current.end_x = point.x.clamp(0.0, bounds.width);
                }
                Some(Action::request_redraw().and_capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let current = drag.take()?;
                if let Some(point) = cursor.position_from(bounds.position())
                    && (point.x - current.anchor_x).abs() >= 3.0
                {
                    let end =
                        crate::editor_drawing::visible_boundary(viewport, point.x, bounds.width);
                    if current.anchor != end {
                        return Some(
                            Action::publish(on_fit(
                                current.anchor.min(end),
                                current.anchor.max(end),
                            ))
                            .and_capture(),
                        );
                    }
                }
                Some(Action::request_redraw().and_capture())
            }
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })
            | canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                *drag = None;
                Some(Action::request_redraw().and_capture())
            }
            _ => Some(Action::capture()),
        }
    }

    pub fn draw(&self, frame: &mut canvas::Frame, height: f32, theme: &Theme) {
        let colors = crate::appearance::colors(theme);
        let position = Point::new(self.anchor_x.min(self.end_x), 0.0);
        let size = Size::new((self.end_x - self.anchor_x).abs(), height);
        frame.fill_rectangle(position, size, colors.zoom);
        frame.stroke(
            &canvas::Path::rectangle(position, size),
            canvas::Stroke::default()
                .with_width(1.0)
                .with_color(colors.focus),
        );
    }
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
        // A modal/global release can finish the gesture without delivering this canvas its release.
        if state.pan_x.is_some() && state.pan_epoch != self.viewport.middle_pan_epoch() {
            state.pan_x = None;
        }
        if let canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            state.modifiers = *modifiers;
            return None;
        }
        if state.zoom_drag.is_some() {
            return ZoomDrag::update(
                &mut state.zoom_drag,
                event,
                self.viewport,
                bounds,
                cursor,
                |start, end| (self.on_event)(TimelineEvent::FitRange(start, end)),
            );
        }
        if let Some((id, mut range, left)) = state.example_drag {
            match event {
                canvas::Event::Mouse(mouse::Event::CursorMoved { .. })
                | canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    let x = cursor.position_from(bounds.position())?.x;
                    let time =
                        crate::editor_drawing::visible_boundary(self.viewport, x, bounds.width);
                    let minimum = TICKS_PER_SECOND / 4;
                    if left {
                        range.start = time.min(range.end.saturating_sub(minimum));
                    } else {
                        range.end = time
                            .max(range.start.saturating_add(minimum))
                            .min(self.viewport.total_samples());
                    }
                    let commit =
                        matches!(event, canvas::Event::Mouse(mouse::Event::ButtonReleased(_)));
                    if commit {
                        state.example_drag = None;
                    }
                    return Some(
                        Action::publish((self.on_example.as_ref()?)(ExampleEvent::Resize(
                            id, range, commit,
                        )))
                        .and_capture(),
                    );
                }
                canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                })
                | canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                    state.example_drag = None;
                    return Some(
                        Action::publish((self.on_example.as_ref()?)(ExampleEvent::CancelResize))
                            .and_capture(),
                    );
                }
                _ => return Some(Action::capture()),
            }
        }
        match event {
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })
            | canvas::Event::Window(iced::window::Event::Unfocused)
                if state.pan_x.is_some() =>
            {
                state.pan_x = None;
                Some(Action::publish((self.on_event)(TimelineEvent::PanCancel)).and_capture())
            }
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) if state.dragging || state.pan_x.is_some() => {
                state.dragging = false;
                state.selection_anchor = None;
                state.pan_x = None;
                Some(Action::request_redraw().and_capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let time = crate::editor_drawing::visible_boundary(
                    self.viewport,
                    position?.x,
                    bounds.width,
                );
                Some(
                    Action::publish((self.on_context.as_ref()?)(cursor.position()?, time))
                        .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                state.pan_x = Some(position?.x);
                state.pan_epoch = self.viewport.middle_pan_epoch();
                Some(Action::publish((self.on_event)(TimelineEvent::PanStart)).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let position = position?;
                if state.modifiers.alt() {
                    state.zoom_drag =
                        Some(ZoomDrag::begin(self.viewport, position.x, bounds.width));
                    return Some(Action::request_redraw().and_capture());
                }
                if let Some(handle) = self.example_handle(position, bounds.width) {
                    state.example_drag = Some(handle);
                    return Some(
                        Action::publish((self.on_example.as_ref()?)(ExampleEvent::Resize(
                            handle.0, handle.1, false,
                        )))
                        .and_capture(),
                    );
                }
                if position.y <= 22.0 {
                    let time = crate::editor_drawing::visible_boundary(
                        self.viewport,
                        position.x,
                        bounds.width,
                    );
                    if let Some(label) = self
                        .examples
                        .iter()
                        .rev()
                        .find(|label| label.range.start <= time && time < label.range.end)
                    {
                        return Some(
                            Action::publish((self.on_example.as_ref()?)(ExampleEvent::Select(
                                label.id,
                            )))
                            .and_capture(),
                        );
                    }
                }
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
                if (position.x - state.anchor_x).abs() < 3.0 {
                    return Some(Action::capture());
                }
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
                if (position.x - state.anchor_x).abs() < 3.0 {
                    state.selection_anchor = None;
                    // Mouse-down already sought; locked scrolling changes the time beneath this pixel.
                    return Some(Action::capture());
                }
                if let Some(anchor) = state.selection_anchor.take() {
                    let end = self.viewport.sample_at(position.x, bounds.width);
                    if anchor == end {
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
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle))
                if state.pan_x.is_some() =>
            {
                state.pan_x = None;
                Some(Action::publish((self.on_event)(TimelineEvent::PanEnd)).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let handle = position
                    .and_then(|point| self.example_handle(point, bounds.width))
                    .map(|(id, _, left)| (id, left));
                if handle != state.hovered_example_handle {
                    state.hovered_example_handle = handle;
                    Some(Action::request_redraw())
                } else {
                    None
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                state.dragging = false;
                state.selection_anchor = None;
                if state.pan_x.take().is_some() {
                    return Some(
                        Action::publish((self.on_event)(TimelineEvent::PanCancel)).and_capture(),
                    );
                }
                state
                    .hovered_example_handle
                    .take()
                    .map(|_| Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let position = position?;
                let direction = match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                        *y
                    }
                };
                let event = TimelineEvent::Zoom {
                    cursor_x: position.x,
                    width: bounds.width,
                    direction,
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
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let _draw = crate::performance::measure(crate::performance::Metric::WaveformDraw);
        let appearance = crate::appearance::Appearance::from_theme(theme);
        let colors = appearance.colors();
        let revision_changed = state.revision.replace(Some(self.revision)) != Some(self.revision);
        let appearance_changed = state.appearance.replace(Some(appearance)) != Some(appearance);
        if revision_changed || appearance_changed {
            state.geometry.clear();
        }
        let geometry = state.geometry.draw(renderer, bounds.size(), |frame| {
            let _build = crate::performance::measure(crate::performance::Metric::WaveformRebuild);
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
                        colors.attenuation,
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
                        colors.selection,
                    );
                }
            }
            if self.ruler {
                for tick in 0..=5 {
                    let x = bounds.width * tick as f32 / 5.0;
                    let sample = self.viewport.sample_at(x, bounds.width);
                    frame.fill_text(canvas::Text {
                        content: format!("{:.1}s", sample as f64 / TICKS_PER_SECOND as f64),
                        position: Point::new(
                            (x + 3.0).min((bounds.width - 64.0).max(0.0)),
                            center - 6.0,
                        ),
                        color: colors.muted,
                        size: 13.0.into(),
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
                                        .with_color(colors.protection),
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
                        .with_color(colors.original_waveform),
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
                            .with_color(colors.waveform),
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
                            .with_color(colors.gain),
                    );
                }
            }
        });
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        draw_selection_edges(
            &mut frame,
            self.viewport,
            colors.active_label,
            self.editing_target || self.ruler,
        );
        if self.editing_target && !self.ruler {
            frame.fill_rectangle(
                Point::ORIGIN,
                Size::new(3.0, bounds.height),
                colors.active_label,
            );
        }
        let hovered_handle = cursor
            .position_in(bounds)
            .and_then(|point| self.example_handle(point, bounds.width))
            .map(|(id, _, left)| (id, left));
        for label in self
            .examples
            .iter()
            .filter(|label| Some(label.id) != self.selected_example)
            .chain(
                self.examples
                    .iter()
                    .filter(|label| Some(label.id) == self.selected_example),
            )
        {
            let selected = Some(label.id) == self.selected_example;
            let range = if selected {
                self.example_draft.unwrap_or(label.range)
            } else {
                label.range
            };
            let left = self.viewport.x_for_sample(range.start, bounds.width);
            let right = self.viewport.x_for_sample(range.end, bounds.width);
            if right < 0.0 || left > bounds.width {
                continue;
            }
            let color = if selected {
                colors.selected_label
            } else {
                colors.label
            };
            frame.fill_rectangle(
                Point::new(left.max(0.0), 0.0),
                Size::new((right.min(bounds.width) - left.max(0.0)).max(1.0), 22.0),
                color,
            );
            let name = if label.active_speakers.is_empty() {
                "Room tone".to_owned()
            } else {
                label
                    .active_speakers
                    .iter()
                    .map(|speaker| speaker.0.to_string())
                    .collect::<Vec<_>>()
                    .join(" + ")
            };
            if right.min(bounds.width) - left.max(0.0) > 65.0 {
                let name = if name == "Room tone" {
                    name
                } else {
                    format!("Speaker {name}")
                };
                let characters = ((right.min(bounds.width) - left.max(0.0) - 14.0) / 8.0) as usize;
                let content = if name.chars().count() > characters {
                    format!(
                        "{}…",
                        name.chars()
                            .take(characters.saturating_sub(1))
                            .collect::<String>()
                    )
                } else {
                    name
                };
                frame.fill_text(canvas::Text {
                    content,
                    position: Point::new(left.max(0.0) + 7.0, 3.0),
                    color: colors.label_text,
                    size: 14.0.into(),
                    ..Default::default()
                });
            }
            if selected {
                for (x, is_left) in [(left, true), (right, false)] {
                    if (0.0..=bounds.width).contains(&x) {
                        let hovered = hovered_handle == Some((label.id, is_left));
                        frame.stroke(
                            &canvas::Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
                            canvas::Stroke::default()
                                .with_color(color)
                                .with_width(if hovered { 2.5 } else { 1.5 }),
                        );
                        frame.fill_rectangle(
                            Point::new((x - 3.0).clamp(0.0, (bounds.width - 6.0).max(0.0)), 2.0),
                            Size::new(6.0, 18.0_f32.min(bounds.height - 2.0).max(0.0)),
                            color,
                        );
                        frame.stroke(
                            &canvas::Path::line(
                                Point::new(x.clamp(3.0, (bounds.width - 3.0).max(3.0)), 7.0),
                                Point::new(
                                    x.clamp(3.0, (bounds.width - 3.0).max(3.0)),
                                    15.0_f32.min(bounds.height),
                                ),
                            ),
                            canvas::Stroke::default()
                                .with_color(colors.label_text)
                                .with_width(1.0),
                        );
                    }
                }
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
                    .with_color(colors.playhead),
            );
        }
        if let Some(drag) = &state.zoom_drag {
            drag.draw(&mut frame, bounds.height, theme);
        }
        vec![geometry, frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.zoom_drag.is_some() || (state.modifiers.alt() && cursor.is_over(bounds)) {
            mouse::Interaction::Crosshair
        } else if state.example_drag.is_some()
            || cursor
                .position_in(bounds)
                .is_some_and(|point| self.example_handle(point, bounds.width).is_some())
        {
            mouse::Interaction::ResizingHorizontally
        } else if state.dragging || state.pan_x.is_some() {
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
    fn middle_pan_publishes_phases_and_finishes_outside_the_canvas() {
        use canvas::Program;
        let viewport = TimelineViewport::new(100 * TICKS_PER_SECOND);
        let waveform = TimelineCanvas::ruler(&viewport, false, |event| event);
        let mut state = TimelineCanvasState::default();
        let bounds = Rectangle::new(Point::new(200.0, 100.0), Size::new(100.0, 80.0));
        let press = canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle));
        let release = canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle));
        let cursor = mouse::Cursor::Available(Point::new(220.0, 140.0));
        assert!(matches!(
            waveform
                .update(&mut state, &press, bounds, cursor)
                .unwrap()
                .into_inner()
                .0,
            Some(TimelineEvent::PanStart)
        ));
        let moved = Point::new(245.0, 140.0);
        let movement = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::CursorMoved { position: moved }),
                bounds,
                mouse::Cursor::Available(moved),
            )
            .unwrap()
            .into_inner();
        assert!(
            matches!(movement.0, Some(TimelineEvent::Pan { delta_x, width }) if delta_x == 25.0 && width == 100.0)
        );
        assert_eq!(movement.2, iced::event::Status::Captured);
        assert!(matches!(
            waveform
                .update(
                    &mut state,
                    &release,
                    bounds,
                    mouse::Cursor::Available(Point::new(450.0, 300.0))
                )
                .unwrap()
                .into_inner()
                .0,
            Some(TimelineEvent::PanEnd)
        ));
        assert!(state.pan_x.is_none());
        assert!(
            waveform
                .update(&mut state, &release, bounds, cursor)
                .is_none()
        );
        assert!(state.selection_anchor.is_none());

        let escape = canvas::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            modified_key: keyboard::Key::Named(keyboard::key::Named::Escape),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::empty(),
            text: None,
            repeat: false,
        });
        for cancellation in [
            escape,
            canvas::Event::Window(iced::window::Event::Unfocused),
            canvas::Event::Mouse(mouse::Event::CursorLeft),
        ] {
            waveform.update(&mut state, &press, bounds, cursor);
            assert!(matches!(
                waveform
                    .update(&mut state, &cancellation, bounds, cursor)
                    .unwrap()
                    .into_inner()
                    .0,
                Some(TimelineEvent::PanCancel)
            ));
            assert!(state.pan_x.is_none());
            assert!(
                waveform
                    .update(&mut state, &release, bounds, cursor)
                    .is_none()
            );
        }
    }

    #[test]
    fn global_middle_pan_cleanup_invalidates_a_canvas_that_missed_release() {
        use canvas::Program;
        let mut viewport = TimelineViewport::new(100 * TICKS_PER_SECOND);
        let mut state = TimelineCanvasState::default();
        let bounds = Rectangle::with_size(Size::new(100.0, 80.0));
        let point = Point::new(40.0, 20.0);
        let cursor = mouse::Cursor::Available(point);
        TimelineCanvas::ruler(&viewport, false, |event| event).update(
            &mut state,
            &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)),
            bounds,
            cursor,
        );
        viewport.begin_middle_pan(crate::timeline::PlaybackScroll::Locked);
        viewport.cancel_middle_pan(crate::timeline::PlaybackScroll::Locked);
        let action = TimelineCanvas::ruler(&viewport, false, |event| event).update(
            &mut state,
            &canvas::Event::Mouse(mouse::Event::CursorMoved { position: point }),
            bounds,
            cursor,
        );
        assert!(action.is_none());
        assert!(state.pan_x.is_none());
    }

    #[test]
    fn selection_edges_only_mark_real_boundaries_inside_the_viewport() {
        let mut viewport = TimelineViewport::new(100 * TICKS_PER_SECOND);
        viewport.fit_range(40 * TICKS_PER_SECOND, 60 * TICKS_PER_SECOND);
        assert_eq!(visible_selection_edges(&viewport, 240.0), [None, None]);
        for (range, expected) in [
            ((10, 50), [None, Some(120.0)]),
            ((50, 90), [Some(120.0), None]),
            ((10, 90), [None, None]),
            ((38, 62), [Some(0.0), Some(240.0)]),
            ((50, 50), [None, None]),
        ] {
            viewport.set_selection(Some((
                range.0 * TICKS_PER_SECOND,
                range.1 * TICKS_PER_SECOND,
            )));
            assert_eq!(visible_selection_edges(&viewport, 240.0), expected);
        }
        viewport.set_selection(Some((40 * TICKS_PER_SECOND, 60 * TICKS_PER_SECOND)));
        for width in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(visible_selection_edges(&viewport, width), [None, None]);
        }
    }

    #[test]
    fn example_handle_hover_redraws_without_editing_or_continuous_frames() {
        use canvas::Program;
        let viewport = TimelineViewport::new(TICKS_PER_SECOND);
        let labels = [crate::ReferenceLabel {
            id: 7,
            range: crate::TimeRange::new(TICKS_PER_SECOND / 4, TICKS_PER_SECOND * 3 / 4).unwrap(),
            active_speakers: vec![],
            sources: vec![],
            reference_track: None,
        }];
        let waveform = TimelineCanvas::ruler(&viewport, false, |event| event).with_examples(
            &labels,
            Some(7),
            None,
            |_| unreachable!(),
        );
        let mut state = TimelineCanvasState::default();
        let bounds = Rectangle::with_size(Size::new(400.0, 80.0));
        let point = Point::new(100.0, 10.0);
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved { position: point });
        let cursor = mouse::Cursor::Available(point);
        let (message, redraw, capture) = waveform
            .update(&mut state, &event, bounds, cursor)
            .unwrap()
            .into_inner();
        assert!(message.is_none());
        assert_eq!(redraw, iced::window::RedrawRequest::NextFrame);
        assert_eq!(capture, iced::event::Status::Ignored);
        assert_eq!(
            waveform.mouse_interaction(&state, bounds, cursor),
            mouse::Interaction::ResizingHorizontally
        );
        assert!(
            waveform
                .update(&mut state, &event, bounds, cursor)
                .is_none()
        );
        assert!(
            waveform
                .update(
                    &mut state,
                    &canvas::Event::Mouse(mouse::Event::CursorLeft),
                    bounds,
                    mouse::Cursor::Unavailable
                )
                .unwrap()
                .into_inner()
                .0
                .is_none()
        );
        assert!(state.example_drag.is_none());
        assert!(state.selection_anchor.is_none());
    }

    #[test]
    fn alt_drag_fits_without_seek_or_label_and_control_wheel_zooms() {
        use canvas::Program;
        let cache = WaveformCache::from_samples(&[0.0; 48000], 48000);
        let mut viewport = TimelineViewport::new(10 * TICKS_PER_SECOND);
        viewport.set_selection(Some((1, 2)));
        let labels = [crate::ReferenceLabel {
            id: 7,
            range: crate::TimeRange::new(2 * TICKS_PER_SECOND, 4 * TICKS_PER_SECOND).unwrap(),
            active_speakers: vec![],
            sources: vec![],
            reference_track: None,
        }];
        let waveform = TimelineCanvas::waveform(&cache, &viewport, |event| event, None, false)
            .with_examples(&labels, Some(7), None, |_| {
                panic!("Alt must precede reference handles")
            })
            .with_selection(|_, _, _| panic!("Alt must not label"));
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(100.0, 80.0));
        let mut state = TimelineCanvasState::default();
        waveform.update(
            &mut state,
            &canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(keyboard::Modifiers::ALT)),
            bounds,
            mouse::Cursor::Unavailable,
        );
        let start = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(20.0, 10.0)),
            )
            .unwrap()
            .into_inner();
        assert!(start.0.is_none());
        let end = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(40.0, 10.0)),
            )
            .unwrap()
            .into_inner();
        assert!(
            matches!(end.0, Some(TimelineEvent::FitRange(a, b)) if a.abs_diff(2 * TICKS_PER_SECOND) < 10 && b.abs_diff(4 * TICKS_PER_SECOND) < 10),
            "{end:?}"
        );
        assert_eq!(viewport.selection(), Some((1, 2)));
        let escape = canvas::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            modified_key: keyboard::Key::Named(keyboard::key::Named::Escape),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::ALT,
            text: None,
            repeat: false,
        });
        for cancellation in [escape, canvas::Event::Mouse(mouse::Event::CursorLeft)] {
            waveform.update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(20.0, 10.0)),
            );
            let cancel = waveform
                .update(
                    &mut state,
                    &cancellation,
                    bounds,
                    mouse::Cursor::Available(Point::new(40.0, 10.0)),
                )
                .unwrap()
                .into_inner();
            assert!(cancel.0.is_none());
            assert_eq!(cancel.2, iced::event::Status::Captured);
            assert!(
                waveform
                    .update(
                        &mut state,
                        &canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                        bounds,
                        mouse::Cursor::Available(Point::new(40.0, 10.0))
                    )
                    .is_none()
            );
        }
        waveform.update(
            &mut state,
            &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(Point::new(20.0, 10.0)),
        );
        let tiny = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(21.0, 10.0)),
            )
            .unwrap()
            .into_inner();
        assert!(tiny.0.is_none());
        waveform.update(
            &mut state,
            &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(Point::new(20.0, 60.0)),
        );
        waveform.update(
            &mut state,
            &canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(
                keyboard::Modifiers::empty(),
            )),
            bounds,
            mouse::Cursor::Unavailable,
        );
        waveform.update(
            &mut state,
            &canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(Point::new(21.0, 60.0)),
        );
        let normal = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                bounds,
                mouse::Cursor::Available(Point::new(60.0, 60.0)),
            )
            .unwrap()
            .into_inner();
        assert!(matches!(normal.0, Some(TimelineEvent::Seek(_))));
        waveform.update(
            &mut state,
            &canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(Point::new(60.0, 60.0)),
        );
        waveform.update(
            &mut state,
            &canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(keyboard::Modifiers::CTRL)),
            bounds,
            mouse::Cursor::Unavailable,
        );
        let wheel = waveform
            .update(
                &mut state,
                &canvas::Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
                }),
                bounds,
                mouse::Cursor::Available(Point::new(20.0, 10.0)),
            )
            .unwrap()
            .into_inner();
        assert!(matches!(wheel.0, Some(TimelineEvent::Zoom { .. })));
    }

    #[test]
    fn example_handles_preview_commit_clamp_and_cancel_without_creating_labels() {
        use canvas::Program;
        #[derive(Debug)]
        enum Input {
            Timeline(TimelineEvent),
            Example(ExampleEvent),
        }
        let ticks = TICKS_PER_SECOND;
        let cache = WaveformCache::from_samples(&[0.0; 48000], 48000);
        let viewport = TimelineViewport::new(ticks);
        let labels = [crate::ReferenceLabel {
            id: 7,
            range: crate::TimeRange::new(ticks / 4, ticks * 3 / 4).unwrap(),
            active_speakers: vec![crate::SpeakerId(1)],
            sources: vec![],
            reference_track: None,
        }];
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(400.0, 80.0));
        let waveform = TimelineCanvas::waveform(&cache, &viewport, Input::Timeline, None, false)
            .with_examples(&labels, Some(7), None, Input::Example);
        let mut state = TimelineCanvasState::default();
        let update = |state: &mut TimelineCanvasState, event, x, y| {
            waveform
                .update(
                    state,
                    &canvas::Event::Mouse(event),
                    bounds,
                    mouse::Cursor::Available(Point::new(x, y)),
                )
                .unwrap()
                .into_inner()
        };
        let press = update(
            &mut state,
            mouse::Event::ButtonPressed(mouse::Button::Left),
            100.0,
            40.0,
        );
        assert!(matches!(
            press.0,
            Some(Input::Example(ExampleEvent::Resize(7, _, false)))
        ));
        let moved = update(
            &mut state,
            mouse::Event::CursorMoved {
                position: Point::new(390.0, 40.0),
            },
            390.0,
            40.0,
        );
        assert!(
            matches!(moved.0, Some(Input::Example(ExampleEvent::Resize(7, range, false))) if range.start == ticks / 2 && range.end == ticks * 3 / 4)
        );
        let release = update(
            &mut state,
            mouse::Event::ButtonReleased(mouse::Button::Left),
            -20.0,
            40.0,
        );
        assert!(
            matches!(release.0, Some(Input::Example(ExampleEvent::Resize(7, range, true))) if range.start == 0 && range.end == ticks * 3 / 4)
        );
        assert_eq!(release.2, iced::event::Status::Captured);
        assert!(state.example_drag.is_none());
        let _ = update(
            &mut state,
            mouse::Event::ButtonPressed(mouse::Button::Left),
            300.0,
            40.0,
        );
        let release = update(
            &mut state,
            mouse::Event::ButtonReleased(mouse::Button::Left),
            450.0,
            40.0,
        );
        assert!(
            matches!(release.0, Some(Input::Example(ExampleEvent::Resize(7, range, true))) if range.end == ticks)
        );
        let _ = update(
            &mut state,
            mouse::Event::ButtonPressed(mouse::Button::Left),
            100.0,
            40.0,
        );
        let cancel = update(&mut state, mouse::Event::CursorLeft, 100.0, 40.0);
        assert!(matches!(
            cancel.0,
            Some(Input::Example(ExampleEvent::CancelResize))
        ));
        let selected = update(
            &mut state,
            mouse::Event::ButtonPressed(mouse::Button::Left),
            200.0,
            10.0,
        );
        assert!(matches!(
            selected.0,
            Some(Input::Example(ExampleEvent::Select(7)))
        ));
        let seek = update(
            &mut state,
            mouse::Event::ButtonPressed(mouse::Button::Left),
            200.0,
            40.0,
        );
        assert!(matches!(
            seek.0,
            Some(Input::Timeline(TimelineEvent::Seek(_)))
        ));
    }

    #[test]
    fn locked_click_seeks_once_even_when_the_view_moves_before_release() {
        use crate::timeline::PlaybackScroll;
        use canvas::Program;
        fn make_canvas<'a>(
            view: &'a TimelineViewport,
            cache: &'a WaveformCache,
            ruler: bool,
        ) -> TimelineCanvas<'a, TimelineEvent> {
            if ruler {
                TimelineCanvas::ruler(view, false, |event| event)
            } else {
                TimelineCanvas::waveform(cache, view, |event| event, None, false)
            }
        }
        let cache = WaveformCache::from_samples(&[0.0; 48000], 48000);
        for ruler in [false, true] {
            for jitter in [0.0, 1.0, -1.0] {
                let mut viewport = TimelineViewport::new(100 * TICKS_PER_SECOND);
                for _ in 0..6 {
                    viewport.zoom_at(0.0, 1000.0, 1.0);
                }
                viewport.seek(viewport.sample_at(250.0, 1000.0));
                viewport.lock_playhead();
                let target = viewport.sample_at(750.0, 1000.0);
                let bounds = Rectangle::new(Point::ORIGIN, Size::new(1000.0, 80.0));
                let mut state = TimelineCanvasState::default();
                let press = make_canvas(&viewport, &cache, ruler)
                    .update(
                        &mut state,
                        &canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                        bounds,
                        mouse::Cursor::Available(Point::new(750.0, 40.0)),
                    )
                    .unwrap()
                    .into_inner();
                assert!(matches!(press.0, Some(TimelineEvent::Seek(time)) if time == target));
                viewport.seek(target);
                viewport.follow_playhead(PlaybackScroll::Locked);
                assert_ne!(viewport.sample_at(750.0, 1000.0), target);
                let cursor = mouse::Cursor::Available(Point::new(750.0 + jitter, 40.0));
                let moved = make_canvas(&viewport, &cache, ruler)
                    .update(
                        &mut state,
                        &canvas::Event::Mouse(mouse::Event::CursorMoved {
                            position: Point::new(750.0 + jitter, 40.0),
                        }),
                        bounds,
                        cursor,
                    )
                    .unwrap()
                    .into_inner();
                assert!(moved.0.is_none(), "click jitter must not seek again");
                let release = make_canvas(&viewport, &cache, ruler)
                    .update(
                        &mut state,
                        &canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                        bounds,
                        cursor,
                    )
                    .unwrap()
                    .into_inner();
                assert!(
                    release.0.is_none(),
                    "release must not remap the click through the shifted view"
                );
                assert_eq!(release.2, iced::event::Status::Captured);
                assert!(!state.dragging);
                assert!(state.selection_anchor.is_none());
            }
        }
    }

    #[test]
    fn plain_waveform_click_seeks_and_drag_release_opens_label_actions() {
        use canvas::Program;
        #[derive(Debug)]
        enum Input {
            Timeline(TimelineEvent),
            Context(Point, u64),
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
        assert!(
            matches!(context.0, Some(Input::Context(point, time)) if point == Point::new(85.0,40.0) && time == TICKS_PER_SECOND * 3 / 4)
        );
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
        assert!(short_drag.0.is_none());
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
    fn streaming_waveform_preserves_chunk_boundaries_and_only_refines_complete_buckets() {
        let mut samples = vec![2.0; 256];
        samples[255] = 3.0;
        samples.extend(vec![-2.0; 256]);
        samples[511] = -3.0;
        samples.extend([4.0; 3]);
        let mut builder = WaveformBuilder::default();
        let mut preview = WaveformCache::from_peaks(vec![(0.0, 0.0); 3], 515, 48_000);
        let mut start = 0;
        for end in [255, 256, 257, 511, 515] {
            builder.push(&samples[start..end]);
            builder.refine_preview(&mut preview);
            assert_eq!(
                preview.levels[0].peaks[0],
                if end < 256 { (0.0, 0.0) } else { (2.0, 3.0) }
            );
            assert_eq!(
                preview.levels[0].peaks[1],
                if end < 512 { (0.0, 0.0) } else { (-3.0, -2.0) }
            );
            assert_eq!(preview.levels[0].peaks[2], (0.0, 0.0));
            start = end;
        }
        let cache = builder.finish(48_000);
        assert_eq!(cache.sample_count, 515);
        assert_eq!(cache.sample_rate, 48_000);
        assert_eq!(
            cache.levels[0].peaks,
            [(2.0, 3.0), (-3.0, -2.0), (4.0, 4.0)]
        );
    }

    #[test]
    fn streaming_waveform_finishes_empty_and_exact_buckets_without_an_extra_tail() {
        for count in [0, 256, 512] {
            let mut builder = WaveformBuilder::default();
            builder.push(&[]);
            builder.push(&vec![0.25; count]);
            let cache = builder.finish(44_100);
            assert_eq!(cache.sample_count, count);
            assert_eq!(cache.sample_rate, 44_100);
            assert_eq!(cache.levels[0].peaks, vec![(0.25, 0.25); count / 256]);
            assert_eq!(
                cache.levels.last().unwrap().peaks,
                if count == 0 {
                    vec![]
                } else {
                    vec![(0.25, 0.25)]
                }
            );
        }
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
