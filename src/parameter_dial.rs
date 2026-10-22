use crate::appearance::colors;
use iced::{
    Color, Point, Radians, Rectangle, Renderer, Theme,
    advanced::mouse::{Click, click},
    keyboard, mouse,
    widget::canvas::{
        self, Action, Frame, Geometry, Gradient, LineCap, Path, Stroke, gradient, path,
    },
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialKind {
    Attenuation,
    Open,
    Close,
    Gap,
}

impl DialKind {
    fn range(self) -> (f32, f32) {
        match self {
            Self::Attenuation => (-30.0, -3.0),
            Self::Open => (1.0, 1000.0),
            Self::Close => (1.0, 2000.0),
            Self::Gap => (10.0, 5000.0),
        }
    }

    fn units_per_pixel(self) -> f32 {
        if self == Self::Attenuation {
            0.25
        } else {
            let (min, max) = self.range();
            (max - min) / 180.0
        }
    }

    fn wheel_step(self) -> f32 {
        match self {
            Self::Attenuation => 1.0,
            Self::Open => 10.0,
            Self::Close => 20.0,
            Self::Gap => 50.0,
        }
    }

    fn adjusted(self, value: f32, delta: f32) -> f32 {
        let (min, max) = self.range();
        let adjusted = (value + delta).clamp(min, max);
        if delta == 0.0 || (adjusted - value).signum() != delta.signum() {
            value
        } else {
            adjusted
        }
    }
}

fn quantize(value: f32, fine: bool) -> f32 {
    let scale = if fine { 10.0 } else { 1.0 };
    (value * scale).round() / scale
}

pub(crate) struct ParameterDial<Message> {
    pub kind: DialKind,
    pub value: f32,
    pub muted: bool,
    pub enabled: bool,
    pub keyboard_enabled: bool,
    pub on_change: fn(f32) -> Message,
    pub on_release: Message,
    pub on_reset: Message,
    pub on_edit: Message,
}

#[derive(Default)]
pub(crate) struct DialState {
    motion: Option<DialMotion>,
    drag: Option<Drag>,
    modifiers: keyboard::Modifiers,
    last_click: Option<Click>,
    wheel_remainder: f32,
}

struct DialMotion {
    from: f32,
    target: f32,
    started: Instant,
}

impl DialMotion {
    const DURATION: Duration = Duration::from_millis(110);

    fn value(&self, now: Instant) -> f32 {
        let t = (now.saturating_duration_since(self.started).as_secs_f32()
            / Self::DURATION.as_secs_f32())
        .min(1.0);
        self.from + (self.target - self.from) * (1.0 - (1.0 - t).powi(3))
    }

    fn retarget(&mut self, target: f32, now: Instant) {
        if self.target != target {
            self.from = self.value(now);
            self.target = target;
            self.started = now;
        }
    }

    fn active(&self, now: Instant) -> bool {
        self.from != self.target && now.saturating_duration_since(self.started) < Self::DURATION
    }
}

struct Drag {
    start_y: f32,
    last_y: f32,
    raw: f32,
    last: f32,
    started: bool,
    changed: bool,
}

impl DialState {
    fn modifiers(&mut self, modifiers: keyboard::Modifiers) {
        if self.modifiers.shift() != modifiers.shift() {
            if let Some(drag) = &mut self.drag {
                // Fine adjustment starts at the displayed value, never an old rounding remainder.
                drag.raw = drag.last;
            }
            self.wheel_remainder = 0.0;
        }
        self.modifiers = modifiers;
    }
}

impl<Message: Clone> canvas::Program<Message> for ParameterDial<Message> {
    type State = DialState;

    fn update(
        &self,
        state: &mut DialState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if let canvas::Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            let motion = state.motion.get_or_insert(DialMotion {
                from: self.value,
                target: self.value,
                started: *now,
            });
            motion.retarget(self.value, *now);
            return motion.active(*now).then(Action::request_redraw);
        }
        if let canvas::Event::Keyboard(
            keyboard::Event::ModifiersChanged(modifiers)
            | keyboard::Event::KeyPressed { modifiers, .. },
        ) = event
        {
            state.modifiers(*modifiers);
        }
        if !self.enabled {
            state.last_click = None;
            state.wheel_remainder = 0.0;
            return state.drag.take().map(|drag| {
                if drag.changed {
                    Action::publish(self.on_release.clone()).and_capture()
                } else {
                    Action::request_redraw()
                }
            });
        }
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let point = cursor.position_in(bounds)?;
                if state.modifiers.command() {
                    state.drag = None;
                    state.last_click = None;
                    return Some(Action::publish(self.on_reset.clone()).and_capture());
                }
                let click = Click::new(point, mouse::Button::Left, state.last_click);
                state.last_click = Some(click);
                if click.kind() == click::Kind::Double {
                    state.drag = None;
                    state.last_click = None;
                    return Some(Action::publish(self.on_edit.clone()).and_capture());
                }
                state.drag = Some(Drag {
                    start_y: point.y,
                    last_y: point.y,
                    raw: self.value,
                    last: self.value,
                    started: false,
                    changed: false,
                });
                Some(Action::request_redraw().and_capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let drag = state.drag.as_mut()?;
                let y = cursor.position()?.y - bounds.y;
                if !drag.started {
                    if self.value != drag.last {
                        drag.raw = self.value;
                        drag.last = self.value;
                    }
                    if (y - drag.start_y).abs() < 3.0 {
                        return Some(Action::capture());
                    }
                    drag.started = true;
                    state.last_click = None;
                }
                let delta = drag.last_y - y;
                drag.last_y = y;
                if delta == 0.0 {
                    return Some(Action::capture());
                }
                let fine = state.modifiers.shift();
                let adjusted = self.kind.adjusted(
                    drag.raw,
                    delta * self.kind.units_per_pixel() * if fine { 0.1 } else { 1.0 },
                );
                if adjusted == drag.raw {
                    return Some(Action::capture());
                }
                drag.raw = adjusted;
                let value = quantize(drag.raw, fine);
                if value == drag.last {
                    return Some(Action::capture());
                }
                drag.last = value;
                drag.changed = true;
                Some(Action::publish((self.on_change)(value)).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta })
                if self.keyboard_enabled && cursor.is_over(bounds) =>
            {
                if state.drag.is_some() {
                    return Some(Action::capture());
                }
                let lines = match *delta {
                    mouse::ScrollDelta::Lines { y, .. } => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 40.0,
                };
                if !lines.is_finite() || lines == 0.0 {
                    return None;
                }
                state.last_click = None;
                state.wheel_remainder += lines;
                let steps = state.wheel_remainder.trunc();
                state.wheel_remainder -= steps;
                let fine = state.modifiers.shift();
                let delta = steps * self.kind.wheel_step() * if fine { 0.1 } else { 1.0 };
                let adjusted = self.kind.adjusted(self.value, delta);
                let value = if adjusted == self.value {
                    self.value
                } else {
                    quantize(adjusted, fine)
                };
                Some(if value == self.value {
                    Action::capture()
                } else {
                    Action::publish((self.on_change)(value)).and_capture()
                })
            }
            canvas::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if self.keyboard_enabled
                    && cursor.is_over(bounds)
                    && !modifiers.control()
                    && !modifiers.alt()
                    && !modifiers.logo() =>
            {
                let delta = match key {
                    keyboard::Key::Named(
                        keyboard::key::Named::ArrowUp | keyboard::key::Named::ArrowRight,
                    ) => 1.0,
                    keyboard::Key::Named(
                        keyboard::key::Named::ArrowDown | keyboard::key::Named::ArrowLeft,
                    ) => -1.0,
                    _ => return None,
                };
                state.last_click = None;
                if state.drag.is_some() {
                    return Some(Action::capture());
                }
                let fine = modifiers.shift();
                let adjusted = self
                    .kind
                    .adjusted(self.value, delta * if fine { 0.1 } else { 1.0 });
                let value = if adjusted == self.value {
                    self.value
                } else {
                    quantize(adjusted, fine)
                };
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
                if matches!(
                    event,
                    canvas::Event::Mouse(mouse::Event::CursorLeft) | canvas::Event::Window(_)
                ) {
                    state.last_click = None;
                    state.wheel_remainder = 0.0;
                    if matches!(event, canvas::Event::Window(_)) {
                        state.modifiers = keyboard::Modifiers::default();
                    }
                }
                if !cursor.is_over(bounds) {
                    state.last_click = None;
                }
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
        let diameter = bounds.width.min(bounds.height);
        if diameter < 32.0 {
            return vec![frame.into_geometry()];
        }
        let center = frame.center();
        let radius = diameter / 2.0 - 7.0;
        let face_radius = radius - 6.0;
        let primary = self.kind == DialKind::Attenuation;
        let width = if primary { 4.0 } else { 3.0 };
        let start = std::f32::consts::PI * 0.75;
        let sweep = std::f32::consts::PI * 1.5;
        let (min, max) = self.kind.range();
        let displayed = state
            .motion
            .as_ref()
            .map_or(self.value, |motion| motion.value(Instant::now()));
        let fraction = ((displayed - min) / (max - min)).clamp(0.0, 1.0);
        let over = self.enabled && cursor.is_over(bounds);
        let dragging = self.enabled && state.drag.is_some();
        let accent = if !self.enabled {
            c.disabled
        } else if self.muted {
            c.muted
        } else if dragging {
            c.focus
        } else if over {
            c.action_hover
        } else {
            c.action
        };
        let radial =
            |angle: f32, r: f32| Point::new(center.x + angle.cos() * r, center.y + angle.sin() * r);
        let arc = |radius, end| {
            Path::new(|builder| {
                builder.arc(path::Arc {
                    center,
                    radius,
                    start_angle: Radians(start),
                    end_angle: Radians(end),
                })
            })
        };
        let stroke = |color, width| {
            Stroke::default()
                .with_color(color)
                .with_width(width)
                .with_line_cap(LineCap::Round)
        };
        frame.stroke(&arc(radius, start + sweep), stroke(c.panel_border, width));
        if fraction > 0.0 && self.enabled && !self.muted {
            frame.stroke(
                &arc(radius, start + sweep * fraction),
                stroke(accent, width),
            );
        }
        for angle in [start, start + sweep] {
            frame.fill(
                &Path::circle(radial(angle, radius + 4.5), if primary { 1.5 } else { 1.0 }),
                c.border,
            );
        }
        frame.fill(
            &Path::circle(Point::new(center.x, center.y + 1.5), face_radius + 1.0),
            Color {
                a: 0.18,
                ..Color::BLACK
            },
        );
        frame.fill(&Path::circle(center, face_radius + 0.5), c.panel_border);
        let top = if !self.enabled {
            c.panel
        } else if dragging {
            c.pressed
        } else if over {
            c.hover
        } else {
            c.control
        };
        let face = Path::circle(center, face_radius);
        frame.fill(
            &face,
            Gradient::Linear(
                gradient::Linear::new(
                    Point::new(center.x, center.y - face_radius),
                    Point::new(center.x, center.y + face_radius),
                )
                .add_stop(0.0, top)
                .add_stop(1.0, c.surface),
            ),
        );
        frame.stroke(
            &face,
            stroke(
                if over || dragging { accent } else { c.border },
                if dragging { 1.5 } else { 1.0 },
            ),
        );
        let angle = start + sweep * fraction;
        frame.stroke(
            &Path::line(
                radial(angle, face_radius * 0.45),
                radial(angle, face_radius * 0.83),
            ),
            stroke(
                if self.enabled && !self.muted {
                    c.text
                } else {
                    c.disabled
                },
                if primary { 3.0 } else { 2.0 },
            ),
        );
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &DialState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !self.enabled {
            mouse::Interaction::default()
        } else if state.drag.is_some() {
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

    #[test]
    fn motion_retargets_continuously_and_settles_without_idle_redraws() {
        let now = Instant::now();
        let mut motion = DialMotion {
            from: -18.0,
            target: -18.0,
            started: now,
        };
        assert!(!motion.active(now));
        motion.retarget(-12.0, now);
        assert_eq!(motion.value(now), -18.0);
        let middle = now + Duration::from_millis(55);
        let displayed = motion.value(middle);
        assert!(displayed > -18.0 && displayed < -12.0);
        motion.retarget(-20.0, middle);
        assert_eq!(motion.value(middle), displayed);
        let end = middle + DialMotion::DURATION;
        assert_eq!(motion.value(end), -20.0);
        assert!(!motion.active(end));
    }

    #[test]
    fn redraw_animation_never_publishes_audio_changes() {
        let mut dial = dial(DialKind::Attenuation, -18.0);
        let mut state = DialState::default();
        let now = Instant::now();
        assert!(
            event(
                &dial,
                &mut state,
                canvas::Event::Window(iced::window::Event::RedrawRequested(now)),
                148.0
            )
            .is_none()
        );
        dial.value = -12.0;
        let action = event(
            &dial,
            &mut state,
            canvas::Event::Window(iced::window::Event::RedrawRequested(now)),
            148.0,
        )
        .unwrap();
        assert!(action.0.is_none());
        assert_eq!(state.motion.as_ref().unwrap().value(now), -18.0);
        assert!(
            event(
                &dial,
                &mut state,
                canvas::Event::Window(iced::window::Event::RedrawRequested(
                    now + DialMotion::DURATION
                )),
                148.0
            )
            .is_none()
        );
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Message {
        Change(f32),
        Release,
        Reset,
        Edit,
    }

    fn dial(kind: DialKind, value: f32) -> ParameterDial<Message> {
        ParameterDial {
            kind,
            value,
            muted: false,
            enabled: true,
            keyboard_enabled: true,
            on_change: Message::Change,
            on_release: Message::Release,
            on_reset: Message::Reset,
            on_edit: Message::Edit,
        }
    }

    fn event(
        dial: &ParameterDial<Message>,
        state: &mut DialState,
        event: canvas::Event,
        y: f32,
    ) -> Option<(Option<Message>, iced::event::Status)> {
        dial.update(
            state,
            &event,
            Rectangle::new(Point::new(200.0, 100.0), iced::Size::new(96.0, 96.0)),
            mouse::Cursor::Available(Point::new(248.0, y)),
        )
        .map(|action| {
            let (message, _, status) = action.into_inner();
            (message, status)
        })
    }

    fn press() -> canvas::Event {
        canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
    }
    fn release() -> canvas::Event {
        canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
    }
    fn moved(y: f32) -> canvas::Event {
        canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(248.0, y),
        })
    }
    fn modifiers(modifiers: keyboard::Modifiers) -> canvas::Event {
        canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers))
    }
    fn key(key: keyboard::key::Named, modifiers: keyboard::Modifiers) -> canvas::Event {
        canvas::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            modified_key: keyboard::Key::Named(key),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::ArrowUp),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        })
    }
    fn changed(result: Option<(Option<Message>, iced::event::Status)>) -> f32 {
        let (Some(Message::Change(value)), status) = result.unwrap() else {
            panic!("Expected value change");
        };
        assert_eq!(status, iced::event::Status::Captured);
        value
    }

    #[test]
    fn ordinary_click_and_jitter_preserve_extended_typed_values_and_mute_for_every_kind() {
        for (kind, values) in [
            (DialKind::Attenuation, [-120.0, 0.0]),
            (DialKind::Open, [0.0, 60000.0]),
            (DialKind::Close, [0.0, 60000.0]),
            (DialKind::Gap, [0.0, 60000.0]),
        ] {
            for value in values {
                let mut dial = dial(kind, value);
                dial.muted = true;
                let mut state = DialState::default();
                assert!(event(&dial, &mut state, press(), 90.0).is_none());
                for (input, y) in [(press(), 148.0), (moved(147.0), 147.0), (release(), 147.0)] {
                    assert!(event(&dial, &mut state, input, y).unwrap().0.is_none());
                }
                assert!(state.drag.is_none());
                assert_eq!(dial.value, value);
                assert!(dial.muted);
            }
        }
    }

    #[test]
    fn all_dial_quick_ranges_clamp_real_drag_and_complete_release_outside_once() {
        for kind in [
            DialKind::Attenuation,
            DialKind::Open,
            DialKind::Close,
            DialKind::Gap,
        ] {
            let (min, max) = kind.range();
            let dial = dial(kind, (min + max) / 2.0);
            let mut state = DialState::default();
            event(&dial, &mut state, press(), 148.0);
            assert_eq!(
                changed(event(&dial, &mut state, moved(-3000.0), -3000.0)),
                max
            );
            assert_eq!(
                changed(event(&dial, &mut state, moved(3000.0), 3000.0)),
                min
            );
            assert_eq!(
                event(&dial, &mut state, release(), 3000.0).unwrap().0,
                Some(Message::Release)
            );
            assert!(event(&dial, &mut state, release(), 3000.0).is_none());
        }
    }

    #[test]
    fn shift_changes_mid_drag_rebase_without_value_jumps() {
        let dial = dial(DialKind::Attenuation, -18.0);
        let mut state = DialState::default();
        event(&dial, &mut state, press(), 148.0);
        assert_eq!(
            changed(event(&dial, &mut state, moved(139.0), 139.0)),
            -16.0
        );
        event(
            &dial,
            &mut state,
            modifiers(keyboard::Modifiers::SHIFT),
            139.0,
        );
        assert!(
            event(&dial, &mut state, moved(139.0), 139.0)
                .unwrap()
                .0
                .is_none()
        );
        assert!((changed(event(&dial, &mut state, moved(135.0), 135.0)) - -15.9).abs() < 0.001);
        event(
            &dial,
            &mut state,
            modifiers(keyboard::Modifiers::NONE),
            135.0,
        );
        assert!(
            event(&dial, &mut state, moved(135.0), 135.0)
                .unwrap()
                .0
                .is_none()
        );
        assert_eq!(
            event(&dial, &mut state, release(), 135.0).unwrap().0,
            Some(Message::Release)
        );
    }

    #[test]
    fn wheel_is_hover_only_accumulates_pixels_and_respects_fine_and_text_ownership() {
        let mut dial = dial(DialKind::Attenuation, -18.0);
        let mut state = DialState::default();
        let wheel = |delta| canvas::Event::Mouse(mouse::Event::WheelScrolled { delta });
        let line = wheel(mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 });
        assert!(event(&dial, &mut state, line.clone(), 90.0).is_none());
        dial.keyboard_enabled = false;
        assert!(event(&dial, &mut state, line.clone(), 148.0).is_none());
        dial.keyboard_enabled = true;
        dial.enabled = false;
        assert!(event(&dial, &mut state, line.clone(), 148.0).is_none());
        dial.enabled = true;
        let pixel = wheel(mouse::ScrollDelta::Pixels { x: 0.0, y: 20.0 });
        assert!(
            event(&dial, &mut state, pixel.clone(), 148.0)
                .unwrap()
                .0
                .is_none()
        );
        dial.value = changed(event(&dial, &mut state, pixel, 148.0));
        assert_eq!(dial.value, -17.0);
        event(
            &dial,
            &mut state,
            modifiers(keyboard::Modifiers::SHIFT),
            148.0,
        );
        assert!((changed(event(&dial, &mut state, line, 148.0)) - -16.9).abs() < 0.001);
        assert!(
            event(
                &dial,
                &mut state,
                wheel(mouse::ScrollDelta::Lines { x: 1.0, y: 0.0 }),
                148.0
            )
            .is_none()
        );
        assert!(
            event(
                &dial,
                &mut state,
                wheel(mouse::ScrollDelta::Lines {
                    x: 0.0,
                    y: f32::NAN
                }),
                148.0
            )
            .is_none()
        );
    }

    #[test]
    fn command_click_resets_and_double_click_edits_without_starting_a_drag() {
        for kind in [
            DialKind::Attenuation,
            DialKind::Open,
            DialKind::Close,
            DialKind::Gap,
        ] {
            let dial = dial(kind, kind.range().0);
            let mut state = DialState::default();
            event(
                &dial,
                &mut state,
                modifiers(keyboard::Modifiers::COMMAND),
                148.0,
            );
            assert_eq!(
                event(&dial, &mut state, press(), 148.0).unwrap().0,
                Some(Message::Reset)
            );
            assert!(event(&dial, &mut state, release(), 148.0).is_none());
            event(
                &dial,
                &mut state,
                modifiers(keyboard::Modifiers::NONE),
                148.0,
            );
            event(&dial, &mut state, press(), 148.0);
            event(&dial, &mut state, release(), 148.0);
            assert_eq!(
                event(&dial, &mut state, press(), 148.0).unwrap().0,
                Some(Message::Edit)
            );
            assert!(state.drag.is_none());
            assert!(event(&dial, &mut state, release(), 148.0).is_none());
        }
    }

    #[test]
    fn dragging_clears_double_click_history_and_unfocus_finishes_only_once() {
        let dial = dial(DialKind::Attenuation, -18.0);
        let mut state = DialState::default();
        event(&dial, &mut state, press(), 148.0);
        changed(event(&dial, &mut state, moved(144.0), 144.0));
        event(&dial, &mut state, release(), 144.0);
        assert!(
            event(&dial, &mut state, press(), 144.0)
                .unwrap()
                .0
                .is_none()
        );
        changed(event(&dial, &mut state, moved(140.0), 140.0));
        let unfocused = canvas::Event::Window(iced::window::Event::Unfocused);
        assert_eq!(
            event(&dial, &mut state, unfocused.clone(), 140.0)
                .unwrap()
                .0,
            Some(Message::Release)
        );
        assert!(event(&dial, &mut state, unfocused, 140.0).is_none());
        assert!(event(&dial, &mut state, release(), 140.0).is_none());
    }

    #[test]
    fn arrows_are_precise_and_do_not_steal_text_or_modified_application_shortcuts() {
        for kind in [
            DialKind::Attenuation,
            DialKind::Open,
            DialKind::Close,
            DialKind::Gap,
        ] {
            let mut dial = dial(
                kind,
                if kind == DialKind::Attenuation {
                    -18.0
                } else {
                    50.0
                },
            );
            let mut state = DialState::default();
            let up = key(keyboard::key::Named::ArrowUp, keyboard::Modifiers::NONE);
            assert!(event(&dial, &mut state, up.clone(), 90.0).is_none());
            dial.keyboard_enabled = false;
            assert!(event(&dial, &mut state, up.clone(), 148.0).is_none());
            dial.keyboard_enabled = true;
            assert_eq!(
                changed(event(&dial, &mut state, up, 148.0)),
                dial.value + 1.0
            );
            let down = key(keyboard::key::Named::ArrowDown, keyboard::Modifiers::SHIFT);
            assert!(
                (changed(event(&dial, &mut state, down, 148.0)) - (dial.value - 0.1)).abs() < 0.001
            );
            assert!(
                event(
                    &dial,
                    &mut state,
                    key(keyboard::key::Named::ArrowUp, keyboard::Modifiers::COMMAND),
                    148.0
                )
                .is_none()
            );
            dial.enabled = false;
            assert!(
                event(
                    &dial,
                    &mut state,
                    key(keyboard::key::Named::ArrowUp, keyboard::Modifiers::NONE),
                    148.0
                )
                .is_none()
            );
        }
    }

    #[test]
    fn extended_fractional_values_do_not_snap_in_the_opposite_direction() {
        for kind in [
            DialKind::Attenuation,
            DialKind::Open,
            DialKind::Close,
            DialKind::Gap,
        ] {
            let (min, max) = kind.range();
            for (value, direction) in [(min - 0.5, -1.0), (max + 0.5, 1.0)] {
                let dial = dial(kind, value);
                let mut state = DialState::default();
                let arrow = if direction > 0.0 {
                    keyboard::key::Named::ArrowUp
                } else {
                    keyboard::key::Named::ArrowDown
                };
                assert!(
                    event(
                        &dial,
                        &mut state,
                        key(arrow, keyboard::Modifiers::NONE),
                        148.0
                    )
                    .unwrap()
                    .0
                    .is_none()
                );
                let wheel = canvas::Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines {
                        x: 0.0,
                        y: direction,
                    },
                });
                assert!(event(&dial, &mut state, wheel, 148.0).unwrap().0.is_none());
                event(&dial, &mut state, press(), 148.0);
                let y = 148.0 - direction * 4.0;
                assert!(event(&dial, &mut state, moved(y), y).unwrap().0.is_none());
                assert!(event(&dial, &mut state, release(), y).unwrap().0.is_none());
            }
        }
    }

    #[test]
    fn disabling_finishes_a_changed_drag_once_and_blocks_reset_edit_and_wheel() {
        let mut dial = dial(DialKind::Attenuation, -18.0);
        let mut state = DialState::default();
        event(&dial, &mut state, press(), 148.0);
        changed(event(&dial, &mut state, moved(140.0), 140.0));
        dial.enabled = false;
        assert_eq!(
            event(&dial, &mut state, moved(136.0), 136.0).unwrap().0,
            Some(Message::Release)
        );
        for input in [
            release(),
            press(),
            press(),
            modifiers(keyboard::Modifiers::COMMAND),
            press(),
            canvas::Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
            }),
        ] {
            assert!(event(&dial, &mut state, input, 148.0).is_none());
        }
        assert!(state.drag.is_none());
    }

    #[test]
    fn drag_starts_from_a_numeric_value_committed_on_pointer_blur() {
        let mut dial = dial(DialKind::Attenuation, -18.0);
        let mut state = DialState::default();
        event(&dial, &mut state, press(), 148.0);
        dial.value = -12.0;
        assert_eq!(
            changed(event(&dial, &mut state, moved(144.0), 144.0)),
            -11.0
        );
    }
}
