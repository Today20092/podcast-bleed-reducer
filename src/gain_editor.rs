//! Visible-window gain drawing and point interaction; audio stays outside widgets.
use crate::{
    AutomaticEnvelope, GainConstraints, GainEnvelope, Project, TrackId, timeline::TimelineViewport,
};
use iced::{
    Color, Point, Rectangle, Renderer, Theme, mouse,
    widget::canvas::{self, Action},
};

#[derive(Clone, Debug)]
pub enum GainEditorEvent {
    Begin(u64, usize),
    Move(u64, f32),
    End,
    Cancel,
}

#[derive(Clone, Copy, Debug)]
pub struct GainPointer {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn hit_gain_point(
    constraints: &GainConstraints,
    track: TrackId,
    viewport: &TimelineViewport,
    pointer: GainPointer,
) -> Option<(u64, usize)> {
    let start = viewport.start().max(0.0) as u64;
    let end = (viewport.start() + viewport.visible_samples()).ceil() as u64;
    constraints
        .manual_edits
        .iter()
        .filter(|edit| edit.source.track == track)
        .find_map(|edit| {
            let first = edit.points.partition_point(|point| point.time < start);
            let last = edit.points.partition_point(|point| point.time <= end);
            edit.points[first..last]
                .iter()
                .enumerate()
                .find_map(|(offset, point)| {
                    let x = viewport.x_for_sample(point.time, pointer.width);
                    let y = gain_y(point.gain, pointer.height);
                    ((pointer.x - x).hypot(pointer.y - y) <= 9.0)
                        .then_some((edit.id, first + offset))
                })
        })
}
fn gain_y(gain: f32, height: f32) -> f32 {
    8.0 + (1.0 - gain) * (height - 16.0).max(1.0)
}

pub(crate) struct GainCanvas<'a, Message> {
    pub revision: u64,
    pub project: &'a Project,
    pub constraints: &'a GainConstraints,
    pub track: TrackId,
    pub viewport: &'a TimelineViewport,
    pub automatic: Option<&'a AutomaticEnvelope>,
    pub effective: Option<&'a GainEnvelope>,
    pub editing: bool,
    pub on_event: Box<dyn Fn(GainEditorEvent) -> Message + 'a>,
}
#[derive(Default)]
pub(crate) struct GainCanvasState {
    geometry: canvas::Cache,
    revision: std::cell::Cell<Option<u64>>,
    dragging: bool,
}
impl<Message: 'static> canvas::Program<Message> for GainCanvas<'_, Message> {
    type State = GainCanvasState;
    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if !self.editing {
            return None;
        }
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let point = cursor.position_in(bounds)?;
                let hit = hit_gain_point(
                    self.constraints,
                    self.track,
                    self.viewport,
                    GainPointer {
                        x: point.x,
                        y: point.y,
                        width: bounds.width,
                        height: bounds.height,
                    },
                )?;
                state.dragging = true;
                Some(
                    Action::publish((self.on_event)(GainEditorEvent::Begin(hit.0, hit.1)))
                        .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                let point = cursor.position_from(bounds.position())?;
                let time =
                    crate::editor_drawing::visible_boundary(self.viewport, point.x, bounds.width);
                let gain =
                    (1.0 - (point.y - 8.0) / (bounds.height - 16.0).max(1.0)).clamp(0.0, 1.0);
                Some(
                    Action::publish((self.on_event)(GainEditorEvent::Move(time, gain)))
                        .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging =>
            {
                state.dragging = false;
                Some(Action::publish((self.on_event)(GainEditorEvent::End)).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) if state.dragging => {
                state.dragging = false;
                Some(Action::publish((self.on_event)(GainEditorEvent::Cancel)).and_capture())
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
            let start = self.viewport.start().max(0.0) as u64;
            let end = (self.viewport.start() + self.viewport.visible_samples()).ceil() as u64;
            for (automatic, color, width) in [
                (true, Color::from_rgb8(125, 145, 165), 1.0),
                (false, Color::from_rgb8(80, 220, 200), 2.0),
            ] {
                let samples = crate::editor_drawing::sample_visible_curve(
                    self.viewport,
                    bounds.width,
                    |time| {
                        if automatic {
                            self.automatic.map_or(1.0, |e| e.gain_at_time(time))
                        } else {
                            self.effective.map_or(1.0, |e| e.gain_at_time(time))
                        }
                    },
                );
                let path = canvas::Path::new(|builder| {
                    for (column, sample) in samples.iter().enumerate() {
                        let point = Point::new(
                            self.viewport.x_for_sample(sample.time, bounds.width),
                            gain_y(sample.gain, bounds.height),
                        );
                        if column == 0 {
                            builder.move_to(point);
                        } else {
                            builder.line_to(point);
                        }
                    }
                });
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_color(color)
                        .with_width(width),
                );
                let buckets = if automatic {
                    self.automatic.map(|envelope| {
                        crate::visible_automatic_buckets(self.viewport, bounds.width, envelope)
                    })
                } else {
                    self.effective.map(|envelope| {
                        crate::visible_gain_buckets(self.viewport, bounds.width, envelope)
                    })
                };
                if let Some(buckets) = buckets {
                    for bucket in buckets {
                        let x = self.viewport.x_for_sample(bucket.start, bounds.width);
                        frame.stroke(
                            &canvas::Path::line(
                                Point::new(x, gain_y(bucket.lower, bounds.height)),
                                Point::new(x, gain_y(bucket.upper, bounds.height)),
                            ),
                            canvas::Stroke::default()
                                .with_color(color)
                                .with_width(width),
                        );
                    }
                }
            }
            for label in self
                .project
                .reference_labels()
                .iter()
                .filter(|label| label.range.start < end && label.range.end > start)
            {
                let left = self
                    .viewport
                    .x_for_sample(label.range.start.max(start), bounds.width);
                let right = self
                    .viewport
                    .x_for_sample(label.range.end.min(end), bounds.width);
                let active = self
                    .project
                    .sources
                    .iter()
                    .find(|s| s.track_id == self.track)
                    .is_some_and(|s| label.active_speakers.contains(&s.speaker_id));
                frame.fill_rectangle(
                    Point::new(left, bounds.height - 5.0),
                    iced::Size::new((right - left).max(1.0), 5.0),
                    if active {
                        Color::from_rgb8(90, 135, 255)
                    } else {
                        Color::from_rgb8(100, 105, 125)
                    },
                );
            }
            for protection in self.constraints.protections.iter().filter(|p| {
                p.sources.iter().any(|s| s.track == self.track)
                    && p.range.start < end
                    && p.range.end > start
            }) {
                let left = self
                    .viewport
                    .x_for_sample(protection.range.start.max(start), bounds.width);
                let right = self
                    .viewport
                    .x_for_sample(protection.range.end.min(end), bounds.width);
                frame.fill_rectangle(
                    Point::new(left, 3.0),
                    iced::Size::new((right - left).max(1.0), 5.0),
                    Color::from_rgb8(220, 175, 65),
                );
            }
            for edit in self
                .constraints
                .manual_edits
                .iter()
                .filter(|edit| edit.source.track == self.track)
            {
                let first = edit
                    .points
                    .partition_point(|p| p.time < start)
                    .saturating_sub(1);
                let last = edit
                    .points
                    .partition_point(|p| p.time <= end)
                    .saturating_add(1)
                    .min(edit.points.len());
                for point in &edit.points[first..last] {
                    if point.time < start || point.time > end {
                        continue;
                    }
                    let position = Point::new(
                        self.viewport.x_for_sample(point.time, bounds.width),
                        gain_y(point.gain, bounds.height),
                    );
                    frame.fill(
                        &canvas::Path::circle(position, 4.0),
                        Color::from_rgb8(235, 150, 245),
                    );
                }
            }
        });
        vec![geometry]
    }
}
