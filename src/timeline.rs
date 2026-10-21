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
    middle_pan: Option<MiddlePan>,
    middle_pan_epoch: u64,
}

#[derive(Clone, Debug)]
struct MiddlePan {
    prior_lock: Option<f64>,
    moved: bool,
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
            middle_pan: None,
            middle_pan_epoch: 0,
        }
    }

    pub fn set_total_samples(&mut self, total_samples: TimePosition) {
        let epoch = self.middle_pan_epoch.wrapping_add(1);
        *self = Self::new(total_samples);
        self.middle_pan_epoch = epoch;
    }

    pub(crate) fn extend_total_samples(&mut self, total_samples: TimePosition) {
        self.total_samples = self.total_samples.max(total_samples);
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
        let previous = self.start;
        self.start = (self.start - delta_x as f64 / width.max(1.0) as f64 * self.visible)
            .clamp(0.0, self.total_samples as f64 - self.visible);
        if let Some(pan) = &mut self.middle_pan {
            pan.moved |= self.start != previous;
        }
    }

    pub(crate) fn begin_middle_pan(&mut self, mode: PlaybackScroll) {
        if self.middle_pan.is_none() {
            self.middle_pan = Some(MiddlePan {
                prior_lock: (mode == PlaybackScroll::Locked).then_some(self.lock_fraction),
                moved: false,
            });
        }
    }

    pub(crate) fn is_middle_panning(&self) -> bool {
        self.middle_pan.is_some()
    }

    pub(crate) fn middle_pan_epoch(&self) -> u64 {
        self.middle_pan_epoch
    }

    pub(crate) fn override_middle_pan_anchor(&mut self) {
        if let Some(pan) = &mut self.middle_pan {
            pan.prior_lock = None;
        }
    }

    pub(crate) fn finish_middle_pan(&mut self, mode: PlaybackScroll) -> bool {
        let Some(pan) = self.middle_pan.take() else {
            return false;
        };
        self.middle_pan_epoch = self.middle_pan_epoch.wrapping_add(1);
        let previous = self.start;
        if mode == PlaybackScroll::Locked
            && let Some(prior) = pan.prior_lock
        {
            if pan.moved {
                let playhead = self.playhead as f64;
                if playhead < self.start {
                    self.start = playhead;
                } else if playhead > self.start + self.visible {
                    self.start = playhead - self.visible;
                }
                self.start = self
                    .start
                    .clamp(0.0, self.total_samples as f64 - self.visible);
                self.lock_playhead();
                return self.start != previous;
            } else {
                self.lock_fraction = prior;
            }
        }
        self.follow_playhead(mode);
        self.start != previous
    }

    pub(crate) fn cancel_middle_pan(&mut self, mode: PlaybackScroll) -> bool {
        let Some(pan) = self.middle_pan.take() else {
            return false;
        };
        self.middle_pan_epoch = self.middle_pan_epoch.wrapping_add(1);
        if mode == PlaybackScroll::Locked
            && let Some(prior) = pan.prior_lock
        {
            self.lock_fraction = prior;
        }
        self.follow_playhead(mode)
    }

    pub fn lock_playhead(&mut self) {
        self.lock_fraction = ((self.playhead as f64 - self.start) / self.visible).clamp(0.0, 1.0);
    }

    pub fn center_playhead(&mut self) {
        self.override_middle_pan_anchor();
        self.lock_fraction = 0.5;
        self.start = (self.playhead as f64 - self.visible * self.lock_fraction)
            .clamp(0.0, self.total_samples as f64 - self.visible);
    }

    pub fn follow_playhead(&mut self, mode: PlaybackScroll) -> bool {
        if self.middle_pan.is_some() {
            return false;
        }
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
    PanStart,
    PanEnd,
    PanCancel,
}

#[cfg(test)]
mod middle_pan_tests {
    use super::*;

    fn locked() -> TimelineViewport {
        let mut viewport = TimelineViewport::new(100 * TICKS_PER_SECOND);
        viewport.fit_range(30 * TICKS_PER_SECOND, 50 * TICKS_PER_SECOND);
        viewport.seek(40 * TICKS_PER_SECOND);
        viewport.lock_playhead();
        viewport
    }

    #[test]
    fn locked_drag_reanchors_from_release_playhead_and_survives_resize() {
        let mut viewport = locked();
        viewport.set_selection(Some((TICKS_PER_SECOND, 2 * TICKS_PER_SECOND)));
        viewport.begin_middle_pan(PlaybackScroll::Locked);
        viewport.pan_by_pixels(-25.0, 100.0);
        let panned = viewport.start();
        viewport.seek(42 * TICKS_PER_SECOND);
        assert!(!viewport.follow_playhead(PlaybackScroll::Locked));
        assert_eq!(viewport.start(), panned);
        assert!(!viewport.finish_middle_pan(PlaybackScroll::Locked));
        assert!(!viewport.is_middle_panning());
        let release_fraction = viewport.x_for_sample(viewport.playhead(), 400.0) / 400.0;
        assert!((release_fraction - 1.0 / 3.0).abs() < 0.0001);
        assert_eq!(
            viewport.selection(),
            Some((TICKS_PER_SECOND, 2 * TICKS_PER_SECOND))
        );
        viewport.seek(43 * TICKS_PER_SECOND);
        viewport.follow_playhead(PlaybackScroll::Locked);
        assert!(
            (viewport.x_for_sample(viewport.playhead(), 1000.0) / 1000.0 - release_fraction).abs()
                < 0.0001
        );
        let start = viewport.start();
        assert!(!viewport.finish_middle_pan(PlaybackScroll::Locked));
        assert_eq!(viewport.start(), start);
    }

    #[test]
    fn middle_click_and_cancel_restore_prior_anchor_without_seeking() {
        for moved in [false, true] {
            let mut viewport = locked();
            let anchor = viewport.lock_fraction;
            viewport.begin_middle_pan(PlaybackScroll::Locked);
            if moved {
                viewport.pan_by_pixels(-30.0, 100.0);
            }
            viewport.seek(42 * TICKS_PER_SECOND);
            if moved {
                viewport.cancel_middle_pan(PlaybackScroll::Locked);
            } else {
                viewport.finish_middle_pan(PlaybackScroll::Locked);
            }
            assert_eq!(viewport.lock_fraction, anchor);
            assert_eq!(viewport.playhead(), 42 * TICKS_PER_SECOND);
            assert_eq!(viewport.x_for_sample(viewport.playhead(), 1000.0), 500.0);
            assert!(!viewport.is_middle_panning());
        }
    }

    #[test]
    fn offscreen_release_reveals_nearest_edge_and_respects_bounds() {
        for (delta, edge) in [(-500.0, 0.0), (500.0, 1.0)] {
            let mut viewport = locked();
            viewport.begin_middle_pan(PlaybackScroll::Locked);
            viewport.pan_by_pixels(delta, 100.0);
            assert!(viewport.finish_middle_pan(PlaybackScroll::Locked));
            assert!(
                (viewport.x_for_sample(viewport.playhead(), 100.0) / 100.0 - edge).abs() < 0.0001
            );
            assert_eq!(viewport.playhead(), 40 * TICKS_PER_SECOND);
            viewport.seek(99 * TICKS_PER_SECOND);
            viewport.follow_playhead(PlaybackScroll::Locked);
            assert!(
                viewport.start() + viewport.visible_samples() <= viewport.total_samples() as f64
            );
        }
        let mut short = TimelineViewport::new(0);
        short.begin_middle_pan(PlaybackScroll::Locked);
        short.pan_by_pixels(-30.0, 100.0);
        short.finish_middle_pan(PlaybackScroll::Locked);
        assert_eq!(short.start(), 0.0);
    }

    #[test]
    fn explicit_mode_or_center_changes_win_over_automatic_reanchor() {
        for cancel in [false, true] {
            let mut viewport = locked();
            viewport.begin_middle_pan(PlaybackScroll::Locked);
            viewport.pan_by_pixels(-25.0, 100.0);
            viewport.override_middle_pan_anchor();
            let start = viewport.start();
            if cancel {
                viewport.cancel_middle_pan(PlaybackScroll::Off);
            } else {
                viewport.finish_middle_pan(PlaybackScroll::Off);
            }
            assert_eq!(viewport.start(), start);
            assert!(!viewport.is_middle_panning());
        }
        let mut viewport = locked();
        viewport.begin_middle_pan(PlaybackScroll::Off);
        viewport.pan_by_pixels(-25.0, 100.0);
        viewport.center_playhead();
        viewport.pan_by_pixels(10.0, 100.0);
        viewport.finish_middle_pan(PlaybackScroll::Locked);
        assert_eq!(viewport.x_for_sample(viewport.playhead(), 100.0), 50.0);
    }
}
