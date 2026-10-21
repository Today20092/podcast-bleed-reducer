//! Visible-window gain drawing and point interaction; audio stays outside widgets.
use crate::{
    AutomaticEnvelope, GainConstraints, GainEnvelope, Project, TrackId, timeline::TimelineViewport,
};
use iced::{
    Point, Rectangle, Renderer, Theme, keyboard, mouse,
    widget::canvas::{self, Action},
};

#[derive(Clone, Debug)]
pub enum GainEditorEvent {
    FitRange(u64, u64),
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
    pub selected_point: Option<(u64, usize)>,
    pub on_event: Box<dyn Fn(GainEditorEvent) -> Message + 'a>,
}
#[derive(Default)]
pub(crate) struct GainCanvasState {
    appearance: std::cell::Cell<Option<crate::appearance::Appearance>>,
    geometry: canvas::Cache,
    revision: std::cell::Cell<Option<u64>>,
    dragging: bool,
    modifiers: keyboard::Modifiers,
    zoom_drag: Option<crate::waveform::ZoomDrag>,
    hovered_point: Option<(u64, usize)>,
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
        if let canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            state.modifiers = *modifiers;
            return None;
        }
        if state.zoom_drag.is_some() {
            return crate::waveform::ZoomDrag::update(
                &mut state.zoom_drag,
                event,
                self.viewport,
                bounds,
                cursor,
                |start, end| (self.on_event)(GainEditorEvent::FitRange(start, end)),
            );
        }
        match event {
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) if state.dragging => {
                state.dragging = false;
                state.hovered_point = None;
                Some(Action::publish((self.on_event)(GainEditorEvent::Cancel)).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let point = cursor.position_in(bounds)?;
                if state.modifiers.alt() {
                    state.zoom_drag = Some(crate::waveform::ZoomDrag::begin(
                        self.viewport,
                        point.x,
                        bounds.width,
                    ));
                    return Some(Action::request_redraw().and_capture());
                }
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
                state.hovered_point = None;
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
                state.hovered_point = None;
                Some(Action::publish((self.on_event)(GainEditorEvent::End)).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) if state.dragging => {
                state.dragging = false;
                state.hovered_point = None;
                Some(Action::publish((self.on_event)(GainEditorEvent::Cancel)).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let hovered = cursor.position_in(bounds).and_then(|point| {
                    hit_gain_point(
                        self.constraints,
                        self.track,
                        self.viewport,
                        GainPointer {
                            x: point.x,
                            y: point.y,
                            width: bounds.width,
                            height: bounds.height,
                        },
                    )
                });
                if hovered != state.hovered_point {
                    state.hovered_point = hovered;
                    Some(Action::request_redraw())
                } else {
                    None
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                state.hovered_point.take().map(|_| Action::request_redraw())
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
        let appearance = crate::appearance::Appearance::from_theme(theme);
        let colors = appearance.colors();
        let revision_changed = state.revision.replace(Some(self.revision)) != Some(self.revision);
        let appearance_changed = state.appearance.replace(Some(appearance)) != Some(appearance);
        if revision_changed || appearance_changed {
            state.geometry.clear();
        }
        let geometry = state.geometry.draw(renderer, bounds.size(), |frame| {
            let start = self.viewport.start().max(0.0) as u64;
            let end = (self.viewport.start() + self.viewport.visible_samples()).ceil() as u64;
            for (automatic, color, width) in [
                (true, colors.automatic_gain, 1.0),
                (false, colors.waveform, 2.0),
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
                    let bounds_path = canvas::Path::new(|builder| {
                        for bucket in buckets {
                            let x = self.viewport.x_for_sample(bucket.start, bounds.width);
                            builder.move_to(Point::new(x, gain_y(bucket.lower, bounds.height)));
                            builder.line_to(Point::new(x, gain_y(bucket.upper, bounds.height)));
                        }
                    });
                    frame.stroke(
                        &bounds_path,
                        canvas::Stroke::default()
                            .with_color(color)
                            .with_width(width),
                    );
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
                        colors.active_label
                    } else {
                        colors.inactive
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
                    colors.protection,
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
                    frame.fill(&canvas::Path::circle(position, 4.0), colors.manual_gain);
                }
            }
        });
        let mut overlay = canvas::Frame::new(renderer, bounds.size());
        crate::waveform::draw_selection_edges(
            &mut overlay,
            self.viewport,
            colors.active_label,
            self.selected_point.is_some(),
        );
        let hovered = if self.editing && !state.modifiers.alt() && cursor.is_over(bounds) {
            state.hovered_point
        } else {
            None
        };
        for (target, selected) in [(hovered, false), (self.selected_point, true)] {
            let Some((id, index)) = target else { continue };
            let Some(point) = self
                .constraints
                .manual_edits
                .iter()
                .find(|edit| edit.id == id && edit.source.track == self.track)
                .and_then(|edit| edit.points.get(index))
            else {
                continue;
            };
            let x = self.viewport.x_for_sample(point.time, bounds.width);
            if !(0.0..=bounds.width).contains(&x) {
                continue;
            }
            let position = Point::new(x, gain_y(point.gain, bounds.height));
            overlay.fill(&canvas::Path::circle(position, 6.5), colors.panel);
            overlay.fill(&canvas::Path::circle(position, 4.5), colors.manual_gain);
            overlay.stroke(
                &canvas::Path::circle(position, 6.5),
                canvas::Stroke::default()
                    .with_color(if selected {
                        colors.text
                    } else {
                        colors.manual_gain
                    })
                    .with_width(1.5),
            );
        }
        if let Some(drag) = &state.zoom_drag {
            drag.draw(&mut overlay, bounds.height, theme);
        }
        vec![geometry, overlay.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !self.editing {
            return mouse::Interaction::default();
        }
        if state.zoom_drag.is_some() || (state.modifiers.alt() && cursor.is_over(bounds)) {
            return mouse::Interaction::Crosshair;
        }
        if state.dragging {
            return mouse::Interaction::Grabbing;
        }
        if cursor.position_in(bounds).is_some_and(|point| {
            hit_gain_point(
                self.constraints,
                self.track,
                self.viewport,
                GainPointer {
                    x: point.x,
                    y: point.y,
                    width: bounds.width,
                    height: bounds.height,
                },
            )
            .is_some()
        }) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}
