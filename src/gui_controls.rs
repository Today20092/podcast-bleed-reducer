use crate::appearance::colors;
use iced::advanced::Renderer as _;
use iced::{
    Element, Event, Length, Padding, Rectangle, Renderer, Size, Task, Theme, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
        widget::{
            Operation, Tree,
            operation::{Focusable, Outcome, Scrollable, focusable, scrollable::AbsoluteOffset},
            tree,
        },
    },
    keyboard::{self, Key, key::Named},
    time::Instant,
    widget::{
        Tooltip,
        button::{self, Button},
    },
    window,
};
use std::time::Duration;

pub(crate) fn focus_next<Message: Send + 'static>(reverse: bool) -> Task<Message> {
    let focus = if reverse {
        iced::widget::operation::focus_previous()
    } else {
        iced::widget::operation::focus_next()
    };
    focus.chain(iced::advanced::widget::operate(RevealFocus::default()))
}

#[derive(Clone, Copy)]
struct ScrollFrame {
    index: usize,
    bounds: Rectangle,
    content: Rectangle,
    translation: Vector,
}

fn reveal_offsets(mut target: Rectangle, ancestors: &[ScrollFrame]) -> Vec<(usize, Vector)> {
    if target.width <= 0.0
        || target.height <= 0.0
        || ancestors
            .iter()
            .any(|frame| frame.bounds.width <= 0.0 || frame.bounds.height <= 0.0)
    {
        return Vec::new();
    }
    let axis =
        |start: f32, size: f32, view_start: f32, view_size: f32, current: f32, maximum: f32| {
            let visible = start - current;
            let delta = if size > view_size || visible < view_start {
                visible - view_start
            } else {
                (visible + size - view_start - view_size).max(0.0)
            };
            (current + delta).clamp(0.0, maximum.max(0.0))
        };
    let mut offsets = Vec::new();
    // Layout coordinates are unscrolled; reveal the innermost target before its parent.
    for frame in ancestors.iter().rev() {
        let translation = Vector::new(
            axis(
                target.x,
                target.width,
                frame.bounds.x,
                frame.bounds.width,
                frame.translation.x,
                frame.content.width - frame.bounds.width,
            ),
            axis(
                target.y,
                target.height,
                frame.bounds.y,
                frame.bounds.height,
                frame.translation.y,
                frame.content.height - frame.bounds.height,
            ),
        );
        if translation != frame.translation {
            offsets.push((frame.index, translation));
        }
        target.x -= translation.x;
        target.y -= translation.y;
    }
    offsets
}

#[derive(Default)]
struct RevealFocus {
    next_scroll: usize,
    pending: Option<ScrollFrame>,
    ancestors: Vec<ScrollFrame>,
    offsets: Vec<(usize, Vector)>,
}

impl<Message: Send + 'static> Operation<Message> for RevealFocus {
    fn scrollable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        self.pending = Some(ScrollFrame {
            index: self.next_scroll,
            bounds,
            content,
            translation,
        });
        self.next_scroll += 1;
    }

    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Message>)) {
        let frame = self.pending.take();
        if let Some(frame) = frame {
            self.ancestors.push(frame);
        }
        operate(self);
        if frame.is_some() {
            self.ancestors.pop();
        }
    }

    fn focusable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        bounds: Rectangle,
        state: &mut dyn Focusable,
    ) {
        if state.is_focused() {
            self.offsets = reveal_offsets(bounds, &self.ancestors);
        }
    }

    fn finish(&self) -> Outcome<Message> {
        if self.offsets.is_empty() {
            Outcome::None
        } else {
            Outcome::Chain(Box::new(ApplyReveal {
                next_scroll: 0,
                offsets: self.offsets.clone(),
            }))
        }
    }
}

struct ApplyReveal {
    next_scroll: usize,
    offsets: Vec<(usize, Vector)>,
}

impl<Message> Operation<Message> for ApplyReveal {
    fn scrollable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        _bounds: Rectangle,
        _content: Rectangle,
        _translation: Vector,
        state: &mut dyn Scrollable,
    ) {
        if let Some((_, offset)) = self
            .offsets
            .iter()
            .find(|(index, _)| *index == self.next_scroll)
        {
            state.scroll_to(AbsoluteOffset {
                x: Some(offset.x),
                y: Some(offset.y),
            });
        }
        self.next_scroll += 1;
    }

    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Message>)) {
        operate(self);
    }
}

pub(crate) fn button<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
) -> ControlButton<'a, Message> {
    ControlButton {
        inner: Button::new(content).padding([7, 10]),
        on_press: None,
        keyboard_enabled: true,
        selected: false,
        hover_outline: true,
        id: None,
    }
}

pub(crate) struct ControlButton<'a, Message> {
    inner: Button<'a, Message>,
    on_press: Option<Message>,
    keyboard_enabled: bool,
    selected: bool,
    hover_outline: bool,
    id: Option<iced::advanced::widget::Id>,
}

impl<'a, Message: Clone + 'a> ControlButton<'a, Message> {
    pub fn id(mut self, id: impl Into<iced::advanced::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.inner = self.inner.height(height);
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(padding);
        self
    }

    pub fn style(mut self, style: impl Fn(&Theme, button::Status) -> button::Style + 'a) -> Self {
        self.inner = self.inner.style(style);
        self
    }

    pub fn on_press(self, action: impl crate::button_action::ButtonAction<Message>) -> Self {
        self.on_press_maybe(Some(action))
    }

    pub fn on_press_maybe(
        mut self,
        action: Option<impl crate::button_action::ButtonAction<Message>>,
    ) -> Self {
        let message = action.map(crate::button_action::message);
        self.inner = self.inner.on_press_maybe(message.clone());
        self.on_press = message;
        self
    }

    pub fn keyboard_enabled(mut self, enabled: bool) -> Self {
        self.keyboard_enabled = enabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn hover_outline(mut self, enabled: bool) -> Self {
        self.hover_outline = enabled;
        self
    }
}

#[derive(Default)]
struct ButtonState {
    focused: bool,
    key_down: Option<Named>,
    hover: Hover,
    selection: Hover,
}

impl Focusable for ButtonState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.key_down = None;
    }
}

#[derive(Debug, PartialEq)]
enum KeyAction {
    Ignore,
    Capture,
    Activate,
}

impl ButtonState {
    fn key_action(&mut self, event: &Event, enabled: bool) -> KeyAction {
        if !enabled {
            self.unfocus();
            return KeyAction::Ignore;
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(iced::touch::Event::FingerPressed { .. }) => {
                self.unfocus();
                KeyAction::Ignore
            }
            Event::Window(window::Event::Unfocused) => {
                self.key_down = None;
                KeyAction::Ignore
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key @ (Named::Enter | Named::Space)),
                modifiers,
                repeat,
                ..
            }) if self.focused && !modifiers.control() && !modifiers.alt() && !modifiers.logo() => {
                if !repeat && self.key_down.is_none() {
                    self.key_down = Some(*key);
                }
                KeyAction::Capture
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: Key::Named(key @ (Named::Enter | Named::Space)),
                ..
            }) if self.focused && self.key_down == Some(*key) => {
                self.key_down = None;
                KeyAction::Activate
            }
            _ => KeyAction::Ignore,
        }
    }
}

const HOVER_DURATION: Duration = Duration::from_millis(90);

struct Hover {
    from: f32,
    to: f32,
    changed_at: Instant,
}

impl Default for Hover {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: 0.0,
            changed_at: Instant::now(),
        }
    }
}

impl Hover {
    fn value(&self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.changed_at).as_secs_f32();
        let t = (elapsed / HOVER_DURATION.as_secs_f32()).min(1.0);
        self.from + (self.to - self.from) * (1.0 - (1.0 - t).powi(3))
    }

    fn set(&mut self, hovered: bool, now: Instant) -> bool {
        let target = if hovered { 1.0 } else { 0.0 };
        if self.to == target {
            return false;
        }
        self.from = self.value(now);
        self.to = target;
        self.changed_at = now;
        true
    }

    fn running(&self, now: Instant) -> bool {
        self.from != self.to && now.saturating_duration_since(self.changed_at) < HOVER_DURATION
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for ControlButton<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ButtonState>()
    }

    fn state(&self) -> tree::State {
        let value = if self.selected { 1.0 } else { 0.0 };
        tree::State::new(ButtonState {
            selection: Hover {
                from: value,
                to: value,
                changed_at: Instant::now(),
            },
            ..ButtonState::default()
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(
            &self.inner as &dyn Widget<Message, Theme, Renderer>,
        )]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.inner as &dyn Widget<Message, Theme, Renderer>);
        if self.on_press.is_none() || !self.keyboard_enabled {
            tree.state.downcast_mut::<ButtonState>().unfocus();
        }
    }

    fn size(&self) -> Size<Length> {
        self.inner.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.inner.layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.on_press.is_some() && self.keyboard_enabled {
            operation.focusable(
                self.id.as_ref(),
                layout.bounds(),
                tree.state.downcast_mut::<ButtonState>(),
            );
        }
        self.inner
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<ButtonState>();
        let enabled = self.on_press.is_some();
        let over = cursor
            .position()
            .is_some_and(|p| layout.bounds().contains(p) && viewport.contains(p));
        if !shell.is_event_captured()
            || matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(_))
                    | Event::Touch(iced::touch::Event::FingerPressed { .. })
            )
        {
            match state.key_action(event, enabled && self.keyboard_enabled) {
                KeyAction::Activate => {
                    shell.publish(self.on_press.as_ref().unwrap().clone());
                    shell.capture_event();
                    shell.request_redraw();
                }
                KeyAction::Capture => {
                    shell.capture_event();
                    shell.request_redraw();
                }
                KeyAction::Ignore => {}
            }
        }
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => Instant::now(),
        };
        if state.hover.set(over && enabled && self.hover_outline, now) || state.hover.running(now) {
            shell.request_redraw();
        }
        if state.selection.set(self.selected, now) || state.selection.running(now) {
            shell.request_redraw();
        }
        self.inner.update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.inner.draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        let state = tree.state.downcast_ref::<ButtonState>();
        let focused = state.focused && self.keyboard_enabled && self.on_press.is_some();
        let hover = if self.hover_outline {
            state.hover.value(Instant::now())
        } else {
            0.0
        };
        let selection = state.selection.value(Instant::now());
        if focused || hover > 0.0 || selection > 0.0 {
            let Some(clipped) = layout.bounds().intersection(viewport) else {
                return;
            };
            let mut color = colors(theme).focus;
            color.a = if focused { 1.0 } else { hover * 0.4 };
            renderer.with_layer(clipped, |renderer| {
                if selection > 0.0 {
                    let bounds = layout.bounds();
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: bounds.x + 8.0,
                                y: bounds.y + bounds.height - 3.0,
                                width: (bounds.width - 16.0).max(0.0) * selection,
                                height: 2.0,
                            },
                            border: iced::Border {
                                radius: 1.0.into(),
                                ..iced::Border::default()
                            },
                            ..renderer::Quad::default()
                        },
                        colors(theme).focus,
                    );
                }
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds().shrink(0.5),
                        border: iced::Border {
                            color,
                            width: if state.key_down.is_some() {
                                3.0
                            } else if focused {
                                2.0
                            } else {
                                1.0
                            },
                            radius: 6.0.into(),
                        },
                        ..renderer::Quad::default()
                    },
                    iced::Color::TRANSPARENT,
                )
            });
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.inner
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.inner.overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'a> From<ControlButton<'a, Message>> for Element<'a, Message> {
    fn from(button: ControlButton<'a, Message>) -> Self {
        Element::new(button)
    }
}

pub(crate) fn focus_scope<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    active: bool,
) -> Element<'a, Message> {
    Element::new(FocusScope {
        content: content.into(),
        active,
    })
}

struct FocusScope<'a, Message> {
    content: Element<'a, Message>,
    active: bool,
}

struct ScopeState {
    active: bool,
}

pub(crate) fn shortcut_hint<'a, Message: 'a>(
    hint: Tooltip<'a, Message>,
    generation: u64,
) -> Element<'a, Message> {
    Element::new(ShortcutHint { hint, generation })
}

struct ShortcutHint<'a, Message> {
    hint: Tooltip<'a, Message>,
    generation: u64,
}

struct HintGeneration(u64);

impl<Message> Widget<Message, Theme, Renderer> for ShortcutHint<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<HintGeneration>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(HintGeneration(self.generation))
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(
            &self.hint as &dyn Widget<Message, Theme, Renderer>,
        )]
    }
    fn diff(&self, tree: &mut Tree) {
        let generation = tree.state.downcast_mut::<HintGeneration>();
        if generation.0 != self.generation {
            // Dismiss only the hint timer/open state; keep the control's focus and press state.
            tree.children[0].state = self.hint.state();
            generation.0 = self.generation;
        }
        self.hint.diff(&mut tree.children[0]);
    }
    fn size(&self) -> Size<Length> {
        self.hint.size()
    }
    fn size_hint(&self) -> Size<Length> {
        self.hint.size_hint()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.hint.layout(&mut tree.children[0], renderer, limits)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.hint
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.hint.update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.hint.draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.hint
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.hint.overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<Message> Widget<Message, Theme, Renderer> for FocusScope<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ScopeState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(ScopeState { active: true })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.active {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        } else {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut focusable::unfocus(),
            );
        }
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<ScopeState>();
        if state.active && !self.active {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut focusable::unfocus(),
            );
        }
        state.active = self.active;
        if self.active || !matches!(event, Event::Keyboard(_)) {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        if !self.active && matches!(event, Event::Mouse(_) | Event::Touch(_)) {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut focusable::unfocus(),
            );
        }
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    impl crate::button_action::ButtonAction<()> for () {
        fn into_message(self) {}
    }
    use super::*;

    fn key_event(key: Named, pressed: bool, repeat: bool) -> Event {
        let key = Key::Named(key);
        if pressed {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::default(),
                text: None,
                repeat,
            })
        } else {
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: key.clone(),
                modified_key: key,
                physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::default(),
            })
        }
    }

    #[test]
    fn focused_button_captures_activation_once_and_cancels_on_blur_or_disable() {
        for key in [Named::Enter, Named::Space] {
            let mut state = ButtonState::default();
            assert_eq!(
                state.key_action(&key_event(key, true, false), true),
                KeyAction::Ignore
            );
            state.focus();
            assert_eq!(
                state.key_action(&key_event(key, true, false), true),
                KeyAction::Capture
            );
            assert_eq!(
                state.key_action(&key_event(key, true, true), true),
                KeyAction::Capture
            );
            assert_eq!(
                state.key_action(&key_event(key, false, false), true),
                KeyAction::Activate
            );
            assert_eq!(
                state.key_action(&key_event(key, false, false), true),
                KeyAction::Ignore
            );
            state.key_action(&key_event(key, true, false), true);
            state.unfocus();
            assert_eq!(
                state.key_action(&key_event(key, false, false), true),
                KeyAction::Ignore
            );
            state.focus();
            state.key_action(&key_event(key, true, false), true);
            state.key_action(&Event::Window(window::Event::Unfocused), true);
            assert_eq!(
                state.key_action(&key_event(key, false, false), true),
                KeyAction::Ignore
            );
            state.focus();
            assert_eq!(
                state.key_action(&key_event(key, true, false), false),
                KeyAction::Ignore
            );
            assert!(!state.is_focused());
        }
    }

    #[test]
    fn hover_transition_is_continuous_and_stops_requesting_frames() {
        let now = Instant::now();
        let mut hover = Hover::default();
        assert!(!hover.running(now));
        assert!(hover.set(true, now));
        let middle = now + HOVER_DURATION / 2;
        let before = hover.value(middle);
        assert!(before > 0.0 && before < 1.0);
        assert!(hover.set(false, middle));
        assert_eq!(hover.value(middle), before);
        let end = middle + HOVER_DURATION;
        assert_eq!(hover.value(end), 0.0);
        assert!(!hover.running(end));
        assert!(!hover.set(false, end));
    }

    #[test]
    fn pointer_use_clears_keyboard_focus_and_preserves_global_space() {
        let mut state = ButtonState::default();
        state.focus();
        assert_eq!(
            state.key_action(&key_event(Named::Space, true, false), true),
            KeyAction::Capture
        );
        assert_eq!(
            state.key_action(
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                true
            ),
            KeyAction::Ignore
        );
        assert!(!state.is_focused());
        assert_eq!(
            state.key_action(&key_event(Named::Space, false, false), true),
            KeyAction::Ignore
        );
        assert_eq!(
            state.key_action(&key_event(Named::Space, true, false), true),
            KeyAction::Ignore
        );
    }

    #[test]
    fn initial_selected_indicator_is_settled_and_deselection_uses_bounded_transition() {
        let button = button::<()>("Selected").selected(true);
        let mut tree_state = button.state();
        let state = tree_state.downcast_mut::<ButtonState>();
        let now = Instant::now();
        assert_eq!(state.selection.value(now), 1.0);
        assert!(!state.selection.running(now));
        assert!(state.selection.set(false, now));
        let end = now + HOVER_DURATION;
        assert_eq!(state.selection.value(end), 0.0);
        assert!(!state.selection.running(end));
    }

    #[test]
    fn dismissing_shortcut_hint_preserves_focused_armed_and_native_button_state() {
        let hint = |generation| ShortcutHint {
            hint: iced::widget::tooltip(
                button("Play").hover_outline(false).on_press(()),
                "Play · Space",
                iced::widget::tooltip::Position::Bottom,
            )
            .delay(Duration::from_millis(600)),
            generation,
        };
        let initial = hint(0);
        let mut tree = Tree::new(&initial as &dyn Widget<(), Theme, Renderer>);
        let control = &mut tree.children[0].children[0];
        let state = control.state.downcast_mut::<ButtonState>();
        state.focus();
        assert_eq!(
            state.key_action(&key_event(Named::Space, true, false), true),
            KeyAction::Capture
        );
        let state_address = |state: &tree::State| match state {
            tree::State::Some(state) => state.as_ref() as *const dyn std::any::Any as *const (),
            tree::State::None => panic!("Expected native button state"),
        };
        let native_button_state = state_address(&control.children[0].state);
        struct PreviousHintTimer;
        tree.children[0].state = tree::State::new(PreviousHintTimer);
        hint(1).diff(&mut tree);
        assert!(
            matches!(&tree.children[0].state, tree::State::Some(state) if !state.is::<PreviousHintTimer>())
        );
        let control = &mut tree.children[0].children[0];
        assert_eq!(
            state_address(&control.children[0].state),
            native_button_state
        );
        let state = control.state.downcast_mut::<ButtonState>();
        assert!(state.is_focused());
        assert_eq!(
            state.key_action(&key_event(Named::Space, false, false), true),
            KeyAction::Activate
        );
    }

    #[test]
    fn focused_reveal_preserves_visible_position_and_moves_only_as_needed() {
        let frame = ScrollFrame {
            index: 0,
            bounds: Rectangle {
                x: 20.0,
                y: 100.0,
                width: 200.0,
                height: 100.0,
            },
            content: Rectangle::with_size(Size::new(200.0, 500.0)),
            translation: Vector::new(0.0, 50.0),
        };
        let target = |y, height| Rectangle {
            x: 30.0,
            y,
            width: 100.0,
            height,
        };
        assert!(reveal_offsets(target(160.0, 30.0), &[frame]).is_empty());
        assert_eq!(
            reveal_offsets(target(120.0, 30.0), &[frame]),
            vec![(0, Vector::new(0.0, 20.0))]
        );
        assert_eq!(
            reveal_offsets(target(300.0, 30.0), &[frame]),
            vec![(0, Vector::new(0.0, 130.0))]
        );
        assert!(reveal_offsets(target(300.0, 0.0), &[frame]).is_empty());
        assert!(
            reveal_offsets(
                target(300.0, 30.0),
                &[ScrollFrame {
                    bounds: Rectangle::with_size(Size::ZERO),
                    ..frame
                }]
            )
            .is_empty()
        );
    }

    #[test]
    fn nested_reveal_accounts_for_inner_scroll_before_outer_scroll() {
        let outer = ScrollFrame {
            index: 0,
            bounds: Rectangle {
                x: 0.0,
                y: 100.0,
                width: 200.0,
                height: 150.0,
            },
            content: Rectangle::with_size(Size::new(200.0, 600.0)),
            translation: Vector::new(0.0, 50.0),
        };
        let inner = ScrollFrame {
            index: 2,
            bounds: Rectangle {
                x: 0.0,
                y: 260.0,
                width: 200.0,
                height: 100.0,
            },
            content: Rectangle::with_size(Size::new(200.0, 400.0)),
            translation: Vector::new(0.0, 20.0),
        };
        let target = Rectangle {
            x: 10.0,
            y: 500.0,
            width: 100.0,
            height: 30.0,
        };
        assert_eq!(
            reveal_offsets(target, &[outer, inner]),
            vec![(2, Vector::new(0.0, 170.0)), (0, Vector::new(0.0, 110.0))]
        );
        assert!(
            reveal_offsets(
                target,
                &[
                    ScrollFrame {
                        translation: Vector::new(0.0, 110.0),
                        ..outer
                    },
                    ScrollFrame {
                        translation: Vector::new(0.0, 170.0),
                        ..inner
                    }
                ]
            )
            .is_empty()
        );
    }

    #[test]
    fn reveal_operation_targets_only_focused_ancestors_without_scroll_ids() {
        struct Scroll(Vector);
        impl Scrollable for Scroll {
            fn snap_to(
                &mut self,
                _: iced::advanced::widget::operation::scrollable::RelativeOffset<Option<f32>>,
            ) {
                panic!("Reveal must use absolute offsets");
            }
            fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>) {
                self.0 = Vector::new(offset.x.unwrap_or(self.0.x), offset.y.unwrap_or(self.0.y));
            }
            fn scroll_by(&mut self, _: AbsoluteOffset, _: Rectangle, _: Rectangle) {
                panic!("Reveal must use absolute offsets");
            }
        }
        let mut outer = Scroll(Vector::new(0.0, 50.0));
        let mut sibling = Scroll(Vector::new(0.0, 10.0));
        let mut inner = Scroll(Vector::new(0.0, 20.0));
        let mut focused = ButtonState::default();
        focused.focus();
        let bounds = |y, height| Rectangle {
            x: 0.0,
            y,
            width: 200.0,
            height,
        };
        let content = Rectangle::with_size(Size::new(200.0, 600.0));
        let mut operation: Box<dyn Operation<()>> = Box::new(RevealFocus::default());
        for pass in 0..2 {
            operation.scrollable(None, bounds(100.0, 150.0), content, outer.0, &mut outer);
            operation.traverse(&mut |operation| {
                operation.scrollable(None, bounds(160.0, 100.0), content, sibling.0, &mut sibling);
                operation.traverse(&mut |_| {});
                operation.scrollable(None, bounds(260.0, 100.0), content, inner.0, &mut inner);
                operation.traverse(&mut |operation| {
                    operation.focusable(None, bounds(500.0, 30.0), &mut focused);
                });
            });
            if pass == 0 {
                let Outcome::Chain(next) = operation.finish() else {
                    panic!("Expected reveal pass");
                };
                operation = next;
            }
        }
        assert_eq!(outer.0, Vector::new(0.0, 110.0));
        assert_eq!(inner.0, Vector::new(0.0, 170.0));
        assert_eq!(sibling.0, Vector::new(0.0, 10.0));
    }
}
