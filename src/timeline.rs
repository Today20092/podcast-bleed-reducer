#[derive(Clone, Debug)]
pub struct TimelineViewport {
    total_samples: usize,
    start: f64,
    visible: f64,
    playhead: usize,
    selection: Option<(usize, usize)>,
}

impl Default for TimelineViewport {
    fn default() -> Self {
        Self::new(1)
    }
}

impl TimelineViewport {
    pub fn new(total_samples: usize) -> Self {
        let total_samples = total_samples.max(1);
        Self {
            total_samples,
            start: 0.0,
            visible: total_samples as f64,
            playhead: 0,
            selection: None,
        }
    }

    pub fn set_total_samples(&mut self, total_samples: usize) {
        *self = Self::new(total_samples);
    }

    pub fn start(&self) -> f64 {
        self.start
    }

    pub fn total_samples(&self) -> usize {
        self.total_samples
    }

    pub fn visible_samples(&self) -> f64 {
        self.visible
    }

    pub fn playhead(&self) -> usize {
        self.playhead
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }

    pub fn set_selection(&mut self, selection: Option<(usize, usize)>) {
        self.selection = selection;
    }

    pub fn seek(&mut self, sample: usize) {
        self.playhead = sample.min(self.total_samples.saturating_sub(1));
    }

    pub fn sample_at(&self, x: f32, width: f32) -> usize {
        let fraction = (x / width.max(1.0)).clamp(0.0, 1.0) as f64;
        (self.start + fraction * self.visible)
            .round()
            .clamp(0.0, self.total_samples.saturating_sub(1) as f64) as usize
    }

    pub fn x_for_sample(&self, sample: usize, width: f32) -> f32 {
        (((sample as f64 - self.start) / self.visible) * width as f64) as f32
    }

    pub fn zoom_at(&mut self, cursor_x: f32, width: f32, direction: f32) {
        if direction == 0.0 {
            return;
        }
        let fraction = (cursor_x / width.max(1.0)).clamp(0.0, 1.0) as f64;
        let anchor = self.start + fraction * self.visible;
        let minimum = 4_800.0_f64.min(self.total_samples as f64);
        self.visible = (self.visible * if direction > 0.0 { 0.8 } else { 1.25 })
            .clamp(minimum, self.total_samples as f64);
        self.start =
            (anchor - fraction * self.visible).clamp(0.0, self.total_samples as f64 - self.visible);
    }

    pub fn pan_by_pixels(&mut self, delta_x: f32, width: f32) {
        self.start = (self.start - delta_x as f64 / width.max(1.0) as f64 * self.visible)
            .clamp(0.0, self.total_samples as f64 - self.visible);
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum TimelineEvent {
    Seek(usize),
    Drag(usize),
    DragEnd(usize),
    Select(usize, usize),
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
