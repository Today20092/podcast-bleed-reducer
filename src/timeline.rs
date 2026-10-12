use crate::timebase::{TICKS_PER_SECOND, TimePosition};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlaybackScroll {
    #[default]
    Off,
    Page,
    Locked,
}
impl std::fmt::Display for PlaybackScroll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Off => "Off",
            Self::Page => "Page at edge",
            Self::Locked => "Lock playhead",
        })
    }
}

/// Shared physical timeline in common integer ticks; historical sample names denote time.
#[derive(Clone, Debug)]
pub struct TimelineViewport {
    total_samples: TimePosition,
    start: f64,
    visible: f64,
    playhead: TimePosition,
    lock_fraction: f64,
    selection: Option<(TimePosition, TimePosition)>,
}

impl Default for TimelineViewport {
    fn default() -> Self {
        Self::new(1)
    }
}

impl TimelineViewport {
    pub fn new(total_samples: TimePosition) -> Self {
        let total_samples = total_samples.max(1);
        Self {
            total_samples,
            start: 0.0,
            visible: total_samples as f64,
            playhead: 0,
            lock_fraction: 0.5,
            selection: None,
        }
    }

    pub fn set_total_samples(&mut self, total_samples: TimePosition) {
        *self = Self::new(total_samples);
    }

    pub fn start(&self) -> f64 {
        self.start
    }

    pub fn total_samples(&self) -> TimePosition {
        self.total_samples
    }

    pub fn visible_samples(&self) -> f64 {
        self.visible
    }

    pub fn playhead(&self) -> TimePosition {
        self.playhead
    }

    pub fn selection(&self) -> Option<(TimePosition, TimePosition)> {
        self.selection
    }

    pub fn set_selection(&mut self, selection: Option<(TimePosition, TimePosition)>) {
        self.selection = selection;
    }

    pub fn show_range(&mut self, start: TimePosition, end: TimePosition) {
        let span = end.saturating_sub(start).max(TICKS_PER_SECOND / 10) as f64;
        self.visible = (span * 3.0).min(self.total_samples as f64);
        self.start = ((start as f64 + end as f64 - self.visible) / 2.0)
            .clamp(0.0, self.total_samples as f64 - self.visible);
    }

    pub fn fit_full(&mut self) {
        self.start = 0.0;
        self.visible = self.total_samples as f64;
    }

    pub fn fit_range(&mut self, start: TimePosition, end: TimePosition) {
        let end = end.min(self.total_samples);
        if start >= end {
            return;
        }
        self.visible = ((end - start) as f64 * 1.2)
            .max(TICKS_PER_SECOND as f64)
            .min(self.total_samples as f64);
        self.start = ((start as f64 + end as f64 - self.visible) / 2.0)
            .clamp(0.0, self.total_samples as f64 - self.visible);
    }

    pub fn seek(&mut self, sample: TimePosition) {
        self.playhead = sample.min(self.total_samples.saturating_sub(1));
    }

    pub fn sample_at(&self, x: f32, width: f32) -> TimePosition {
        let fraction = (x / width.max(1.0)).clamp(0.0, 1.0) as f64;
        (self.start + fraction * self.visible)
            .round()
            .clamp(0.0, self.total_samples.saturating_sub(1) as f64) as TimePosition
    }

    pub fn x_for_sample(&self, sample: TimePosition, width: f32) -> f32 {
        (((sample as f64 - self.start) / self.visible) * width as f64) as f32
    }

    pub fn zoom_at(&mut self, cursor_x: f32, width: f32, direction: f32) {
        if direction == 0.0 {
            return;
        }
        let fraction = (cursor_x / width.max(1.0)).clamp(0.0, 1.0) as f64;
        let anchor = self.start + fraction * self.visible;
        let minimum = (TICKS_PER_SECOND as f64 / 10.0).min(self.total_samples as f64);
        self.visible = (self.visible * if direction > 0.0 { 0.8 } else { 1.25 })
            .clamp(minimum, self.total_samples as f64);
        self.start =
            (anchor - fraction * self.visible).clamp(0.0, self.total_samples as f64 - self.visible);
    }

    pub fn pan_by_pixels(&mut self, delta_x: f32, width: f32) {
        self.start = (self.start - delta_x as f64 / width.max(1.0) as f64 * self.visible)
            .clamp(0.0, self.total_samples as f64 - self.visible);
    }

    pub fn lock_playhead(&mut self) {
        self.lock_fraction = ((self.playhead as f64 - self.start) / self.visible).clamp(0.0, 1.0);
    }

    pub fn center_playhead(&mut self) {
        self.lock_fraction = 0.5;
        self.follow_playhead(PlaybackScroll::Locked);
    }

    pub fn follow_playhead(&mut self, mode: PlaybackScroll) -> bool {
        let previous = self.start;
        let playhead = self.playhead as f64;
        let start = match mode {
            PlaybackScroll::Off => self.start,
            PlaybackScroll::Locked => playhead - self.visible * self.lock_fraction,
            PlaybackScroll::Page
                if playhead < self.start || playhead >= self.start + self.visible =>
            {
                playhead
            }
            PlaybackScroll::Page => self.start,
        };
        self.start = start.clamp(0.0, self.total_samples as f64 - self.visible);
        self.start != previous
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum TimelineEvent {
    FitRange(TimePosition, TimePosition),
    Seek(TimePosition),
    Drag(TimePosition),
    DragEnd(TimePosition),
    Select(TimePosition, TimePosition),
    Zoom {
        cursor_x: f32,
        width: f32,
        direction: f32,
    },
    Pan {
        delta_x: f32,
        width: f32,
    },
}
