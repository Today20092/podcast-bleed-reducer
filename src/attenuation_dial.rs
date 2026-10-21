use crate::appearance::colors;
use iced::{
    Point, Radians, Rectangle, Renderer, Theme, keyboard, mouse,
    widget::canvas::{self, Action, Frame, Geometry, Path, Stroke, path},
};

// Keep the former slider's quick-adjustment range; the numeric field retains −120…0 dB.
const MIN_DB: f32 = -30.0;
const MAX_DB: f32 = -3.0;

fn adjusted_value(value: f32, delta: f32) -> f32 {
    let adjusted = (value + delta).clamp(MIN_DB, MAX_DB);
    if delta == 0.0 || (adjusted - value).signum() != delta.signum() {
        value
    } else {
        adjusted
    }
}

pub(crate) struct AttenuationDial<Message> {
    pub value: f32,
    pub muted: bool,
    pub keyboard_enabled: bool,
    pub on_change: fn(f32) -> Message,
    pub on_release: Message,
}

#[derive(Default)]
pub(crate) struct DialState {
    drag: Option<Drag>,
}

struct Drag {
    y: f32,
    value: f32,
    last: f32,
    changed: bool,
}

impl<Message: Clone> canvas::Program<Message> for AttenuationDial<Message> {
    type State = DialState;

    fn update(
        &self,
        state: &mut DialState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let point = cursor.position_in(bounds)?;
                state.drag = Some(Drag {
                    y: point.y,
                    value: self.value,
                    last: self.value,
                    changed: false,
                });
                Some(Action::request_redraw().and_capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let drag = state.drag.as_mut()?;
                let y = cursor.position()?.y - bounds.y;
                let delta = ((drag.y - y) / 4.0).round();
                // Clicking a value entered outside the dial range must not change it.
                let value = adjusted_value(drag.value, delta);
                if value == drag.last {
                    return Some(Action::capture());
                }
                drag.last = value;
                drag.changed = true;
                Some(Action::publish((self.on_change)(value)).and_capture())
            }
            canvas::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                if self.keyboard_enabled && cursor.is_over(bounds) && state.drag.is_none() =>
            {
                let delta = match key {
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => 1.0,
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => -1.0,
                    _ => return None,
                };
                let value = adjusted_value(self.value, delta);
                Some(if value == self.value {
                    Action::capture()
                } else {
                    Action::publish((self.on_change)(value)).and_capture()
                })
            }
            canvas::Event::Mouse(
                mouse::Event::ButtonReleased(mouse::Button::Left) | mouse::Event::CursorLeft,
            )
            | canvas::Event::Window(iced::window::Event::Unfocused) => {
                let drag = state.drag.take()?;
                Some(if drag.changed {
                    Action::publish(self.on_release.clone()).and_capture()
                } else {
                    Action::request_redraw().and_capture()
                })
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &DialState,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let c = colors(theme);
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        let radius = bounds.width.min(bounds.height) / 2.0 - 6.0;
        let start = std::f32::consts::PI * 0.75;
        let sweep = std::f32::consts::PI * 1.5;
        let fraction = ((self.value - MIN_DB) / (MAX_DB - MIN_DB)).clamp(0.0, 1.0);
        let arc = |end| {
            Path::new(|builder| {
                builder.arc(path::Arc {
                    center,
                    radius,
                    start_angle: Radians(start),
                    end_angle: Radians(end),
                });
            })
        };
        frame.stroke(
            &arc(start + sweep),
            Stroke::default().with_color(c.panel_border).with_width(4.0),
        );
        if fraction > 0.0 && !self.muted {
            frame.stroke(
                &arc(start + sweep * fraction),
                Stroke::default().with_color(c.action).with_width(4.0),
            );
        }
        let knob = Path::circle(center, radius - 8.0);
        frame.fill(
            &knob,
            if state.drag.is_some() {
                c.pressed
            } else if cursor.is_over(bounds) {
                c.hover
            } else {
                c.control
            },
        );
        frame.stroke(
            &knob,
            Stroke::default().with_color(c.border).with_width(1.0),
        );
        let angle = start + sweep * fraction;
        let point = |r: f32| Point::new(center.x + angle.cos() * r, center.y + angle.sin() * r);
        frame.stroke(
            &Path::line(point(radius - 22.0), point(radius - 13.0)),
            Stroke::default()
                .with_color(if self.muted { c.muted } else { c.text })
                .with_width(3.0),
        );
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &DialState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.drag.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canvas::Program;

    #[derive(Clone, Debug, PartialEq)]
    enum Message {
        Change(f32),
        Release,
    }

    #[test]
    fn drag_clamps_and_releases_once_even_outside_or_unfocused() {
        let dial = AttenuationDial {
            value: -18.0,
            muted: false,
            keyboard_enabled: true,
            on_change: Message::Change,
            on_release: Message::Release,
        };
        let bounds = Rectangle::new(Point::new(200.0, 100.0), iced::Size::new(88.0, 88.0));
        let mut state = DialState::default();
        let press = canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let release = canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
        let update = |state: &mut DialState, event, y| {
            dial.update(
                state,
                &event,
                bounds,
                mouse::Cursor::Available(Point::new(244.0, y)),
            )
            .map(|a| a.into_inner())
        };
        assert!(update(&mut state, press.clone(), 90.0).is_none());
        assert!(
            update(&mut state, press.clone(), 144.0)
                .unwrap()
                .0
                .is_none()
        );
        for (y, expected) in [(124.0, -13.0), (-100.0, MAX_DB), (500.0, MIN_DB)] {
            let action = update(
                &mut state,
                canvas::Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(244.0, y),
                }),
                y,
            )
            .unwrap();
            assert_eq!(action.0, Some(Message::Change(expected)));
            assert_eq!(action.2, iced::event::Status::Captured);
        }
        assert_eq!(
            update(&mut state, release.clone(), 500.0).unwrap().0,
            Some(Message::Release)
        );
        assert!(update(&mut state, release.clone(), 500.0).is_none());
        update(&mut state, press.clone(), 144.0);
        assert!(update(&mut state, release, 144.0).unwrap().0.is_none());
        update(&mut state, press, 144.0);
        update(
            &mut state,
            canvas::Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(244.0, 120.0),
            }),
            120.0,
        );
        assert_eq!(
            update(
                &mut state,
                canvas::Event::Window(iced::window::Event::Unfocused),
                120.0
            )
            .unwrap()
            .0,
            Some(Message::Release)
        );
        assert!(state.drag.is_none());
    }

    #[test]
    fn click_preserves_extended_typed_values_and_mute() {
        for value in [-120.0, 0.0] {
            assert_eq!(
                adjusted_value(value, if value == 0.0 { 1.0 } else { -1.0 }),
                value
            );
            let dial = AttenuationDial {
                value,
                muted: true,
                keyboard_enabled: true,
                on_change: Message::Change,
                on_release: Message::Release,
            };
            let mut state = DialState::default();
            let bounds = Rectangle::with_size(iced::Size::new(88.0, 88.0));
            let cursor = mouse::Cursor::Available(Point::new(44.0, 44.0));
            for event in [
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                canvas::Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(44.0, 44.0),
                }),
                canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            ] {
                assert!(
                    dial.update(&mut state, &event, bounds, cursor)
                        .unwrap()
                        .into_inner()
                        .0
                        .is_none()
                );
            }
        }
    }

    #[test]
    fn hover_keys_adjust_without_leaking_or_stealing_text_input() {
        let mut dial = AttenuationDial {
            value: -18.0,
            muted: false,
            keyboard_enabled: true,
            on_change: Message::Change,
            on_release: Message::Release,
        };
        let bounds = Rectangle::with_size(iced::Size::new(88.0, 88.0));
        let cursor = mouse::Cursor::Available(Point::new(44.0, 44.0));
        let key = keyboard::Key::Named(keyboard::key::Named::ArrowUp);
        let event = canvas::Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::ArrowUp),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
            text: None,
            repeat: false,
        });
        let mut state = DialState::default();
        let (message, _, status) = dial
            .update(&mut state, &event, bounds, cursor)
            .unwrap()
            .into_inner();
        assert_eq!(message, Some(Message::Change(-17.0)));
        assert_eq!(status, iced::event::Status::Captured);
        assert!(
            dial.update(&mut state, &event, bounds, mouse::Cursor::Unavailable)
                .is_none()
        );
        dial.keyboard_enabled = false;
        assert!(dial.update(&mut state, &event, bounds, cursor).is_none());
    }
}
