use iced::{
    Animation, Element, Event, Length, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
        widget::{Operation, Tree, operation, tree},
    },
    animation::Easing,
    time::{Duration, Instant},
    window,
};

const DURATION: Duration = Duration::from_millis(120);

pub(crate) fn reveal<'a, Message: 'a, Theme: 'a, Renderer: renderer::Renderer + 'a>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    open: bool,
) -> Element<'a, Message, Theme, Renderer> {
    Element::new(Reveal {
        content: content.into(),
        open,
    })
}

struct Reveal<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    open: bool,
}

struct State {
    animation: Animation<bool>,
    at: Instant,
    expanded_size: Size,
}

impl State {
    fn new(open: bool) -> Self {
        Self {
            animation: Animation::new(open)
                .duration(DURATION)
                .easing(Easing::EaseOutCubic),
            at: Instant::now(),
            expanded_size: Size::ZERO,
        }
    }

    fn set_open(&mut self, open: bool, at: Instant) {
        if self.animation.value() != open {
            self.animation.go_mut(open, at);
            self.at = at;
        }
    }

    fn fraction(&self) -> f32 {
        self.animation.interpolate(0.0, 1.0, self.at)
    }

    fn expanded(&self) -> bool {
        self.animation.value() && !self.animation.is_animating(self.at)
    }
}

impl<Message, Theme, Renderer: renderer::Renderer> Widget<Message, Theme, Renderer>
    for Reveal<'_, Message, Theme, Renderer>
{
    fn size(&self) -> Size<Length> {
        Size::new(self.content.as_widget().size().width, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.open))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.state
            .downcast_mut::<State>()
            .set_open(self.open, Instant::now());
        tree.children[0].diff(self.content.as_widget());
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits.loose());
        if !self.open {
            // Retain scroll state while removing focus from newly hidden fields.
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                Layout::new(&child),
                renderer,
                &mut operation::focusable::unfocus::<()>(),
            );
        }
        let size = child.size();
        let state = tree.state.downcast_mut::<State>();
        if self.open {
            state.expanded_size = size;
        }
        layout::Node::with_children(
            Size::new(
                size.width
                    .max(state.expanded_size.width)
                    .min(limits.max().width),
                (state.expanded_size.height * state.fraction()).min(limits.max().height),
            ),
            vec![child],
        )
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
        if let Some(clipped) = layout.bounds().intersection(viewport) {
            let cursor = if self.open && cursor.is_over(clipped) {
                cursor
            } else {
                mouse::Cursor::Unavailable
            };
            renderer.with_layer(clipped, |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout.children().next().unwrap(),
                    cursor,
                    &clipped,
                );
            });
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
        let state = tree.state.downcast_mut::<State>();
        if let Event::Window(window::Event::RedrawRequested(at)) = event {
            let previous = state.fraction();
            state.at = *at;
            if state.fraction() != previous {
                shell.invalidate_layout();
            }
            let bounds = layout.bounds();
            // A just-opening disclosure has zero height but still needs its first frame.
            let visible = Rectangle {
                height: bounds.height.max(1.0),
                ..bounds
            }
            .intersects(viewport);
            if state.animation.is_animating(*at) && visible {
                shell.request_redraw();
            }
        }
        if self.open
            && let Some(clipped) = layout.bounds().intersection(viewport)
        {
            if matches!(event, Event::Touch(
                iced::touch::Event::FingerPressed { position, .. }
                | iced::touch::Event::FingerMoved { position, .. }
            ) if !clipped.contains(*position))
            {
                return;
            }
            let cursor = if cursor.is_over(clipped) {
                cursor
            } else {
                mouse::Cursor::Unavailable
            };
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                &clipped,
            );
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if tree.state.downcast_ref::<State>().expanded() {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
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
        if self.open
            && let Some(clipped) = layout.bounds().intersection(viewport)
        {
            let cursor = if cursor.is_over(clipped) {
                cursor
            } else {
                mouse::Cursor::Unavailable
            };
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor,
                &clipped,
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        if tree.state.downcast_ref::<State>().expanded() {
            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                viewport,
                translation,
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::widget::operation::Focusable;

    struct Probe;

    struct ProbeState {
        focused: bool,
        scroll_position: u32,
        updates: u32,
    }

    impl Focusable for ProbeState {
        fn is_focused(&self) -> bool {
            self.focused
        }
        fn focus(&mut self) {
            self.focused = true;
        }
        fn unfocus(&mut self) {
            self.focused = false;
        }
    }

    impl Widget<(), (), ()> for Probe {
        fn size(&self) -> Size<Length> {
            Size::new(Length::Fixed(200.0), Length::Fixed(100.0))
        }
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<ProbeState>()
        }
        fn state(&self) -> tree::State {
            tree::State::new(ProbeState {
                focused: true,
                scroll_position: 42,
                updates: 0,
            })
        }
        fn layout(&mut self, _: &mut Tree, _: &(), _: &layout::Limits) -> layout::Node {
            layout::Node::new(Size::new(200.0, 100.0))
        }
        fn draw(
            &self,
            _: &Tree,
            _: &mut (),
            _: &(),
            _: &renderer::Style,
            _: Layout<'_>,
            _: mouse::Cursor,
            _: &Rectangle,
        ) {
        }
        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            _: &(),
            operation: &mut dyn Operation,
        ) {
            operation.focusable(
                None,
                layout.bounds(),
                tree.state.downcast_mut::<ProbeState>(),
            );
        }
        fn update(
            &mut self,
            tree: &mut Tree,
            _: &Event,
            _: Layout<'_>,
            _: mouse::Cursor,
            _: &(),
            _: &mut dyn Clipboard,
            shell: &mut Shell<'_, ()>,
            _: &Rectangle,
        ) {
            tree.state.downcast_mut::<ProbeState>().updates += 1;
            shell.publish(());
            shell.capture_event();
        }
    }

    #[test]
    fn transition_is_bounded_finite_and_reversible() {
        let mut state = State::new(false);
        let start = Instant::now();
        state.set_open(true, start);
        assert!(!state.expanded());
        for elapsed in [0, 20, 60, 120, 240] {
            state.at = start + Duration::from_millis(elapsed);
            assert!((0.0..=1.0).contains(&state.fraction()));
        }
        assert_eq!(state.fraction(), 1.0);
        assert!(state.expanded());
        state.set_open(false, start + Duration::from_millis(240));
        assert!(!state.expanded());
        state.at = start + Duration::from_millis(300);
        assert!(state.fraction() > 0.0 && state.fraction() < 1.0);
        state.set_open(true, start + Duration::from_millis(300));
        state.at = start + Duration::from_millis(540);
        assert_eq!(state.fraction(), 1.0);
        assert!(!state.animation.is_animating(state.at));
    }

    #[test]
    fn collapsing_gates_input_and_focus_preserves_child_and_stops_redraws() {
        let mut reveal = Reveal {
            content: Element::new(Probe),
            open: true,
        };
        let mut tree = Tree::new(&reveal as &dyn Widget<(), (), ()>);
        let limits = layout::Limits::new(Size::ZERO, Size::new(300.0, 500.0));
        let viewport = Rectangle::with_size(Size::new(300.0, 500.0));
        assert_eq!(reveal.layout(&mut tree, &(), &limits).size().height, 100.0);
        let start = Instant::now();
        reveal.open = false;
        tree.state.downcast_mut::<State>().set_open(false, start);
        for (elapsed, animate) in [(60, true), (240, false)] {
            let node = reveal.layout(&mut tree, &(), &limits);
            let mut messages = vec![];
            let mut shell = Shell::new(&mut messages);
            reveal.update(
                &mut tree,
                &Event::Window(window::Event::RedrawRequested(
                    start + Duration::from_millis(elapsed),
                )),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &(),
                &mut iced::advanced::clipboard::Null,
                &mut shell,
                &viewport,
            );
            assert!(shell.is_layout_invalid());
            assert_eq!(
                shell.redraw_request(),
                if animate {
                    window::RedrawRequest::NextFrame
                } else {
                    window::RedrawRequest::Wait
                }
            );
            assert!(shell.is_empty());
            let mut operation = operation::focusable::count();
            reveal.operate(
                &mut tree,
                Layout::new(&node),
                &(),
                &mut operation::black_box(&mut operation),
            );
            assert!(
                matches!(operation.finish(), operation::Outcome::Some(count) if count.total == 0)
            );
        }
        let node = reveal.layout(&mut tree, &(), &limits);
        assert_eq!(node.size().height, 0.0);
        let child = tree.children[0].state.downcast_ref::<ProbeState>();
        assert!(!child.focused);
        assert_eq!(child.scroll_position, 42);
        assert_eq!(child.updates, 0);
        reveal.open = true;
        tree.state
            .downcast_mut::<State>()
            .set_open(true, start + Duration::from_millis(240));
        tree.state.downcast_mut::<State>().at = start + Duration::from_millis(480);
        let node = reveal.layout(&mut tree, &(), &limits);
        assert_eq!(node.size().height, 100.0);
        let mut operation = operation::focusable::count();
        reveal.operate(
            &mut tree,
            Layout::new(&node),
            &(),
            &mut operation::black_box(&mut operation),
        );
        assert!(matches!(operation.finish(), operation::Outcome::Some(count) if count.total == 1));
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        reveal.update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Layout::new(&node),
            mouse::Cursor::Available(iced::Point::new(10.0, 10.0)),
            &(),
            &mut iced::advanced::clipboard::Null,
            &mut shell,
            &viewport,
        );
        assert!(shell.is_event_captured());
        assert_eq!(
            tree.children[0].state.downcast_ref::<ProbeState>().updates,
            1
        );
        assert_eq!(
            tree.children[0]
                .state
                .downcast_ref::<ProbeState>()
                .scroll_position,
            42
        );
    }

    #[test]
    fn opening_outside_viewport_does_not_schedule_frames_or_reach_child() {
        let mut reveal = Reveal {
            content: Element::new(Probe),
            open: false,
        };
        let mut tree = Tree::new(&reveal as &dyn Widget<(), (), ()>);
        let start = Instant::now();
        reveal.open = true;
        tree.state.downcast_mut::<State>().set_open(true, start);
        let node = reveal.layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(300.0, 500.0)),
        );
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        reveal.update(
            &mut tree,
            &Event::Window(window::Event::RedrawRequested(
                start + Duration::from_millis(60),
            )),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &(),
            &mut iced::advanced::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(Size::ZERO),
        );
        assert_eq!(shell.redraw_request(), window::RedrawRequest::Wait);
        assert!(shell.is_empty());
        assert_eq!(
            tree.children[0].state.downcast_ref::<ProbeState>().updates,
            0
        );
    }

    #[test]
    fn empty_closed_child_retains_measured_space_for_collapse() {
        let mut reveal = Reveal {
            content: Element::new(Probe),
            open: true,
        };
        let mut tree = Tree::new(&reveal as &dyn Widget<(), (), ()>);
        let limits = layout::Limits::new(Size::ZERO, Size::new(300.0, 500.0));
        assert_eq!(
            reveal.layout(&mut tree, &(), &limits).size(),
            Size::new(200.0, 100.0)
        );
        reveal.open = false;
        reveal.content = Element::new(iced::widget::Space::new());
        reveal.diff(&mut tree);
        let node = reveal.layout(&mut tree, &(), &limits);
        assert_eq!(node.size(), Size::new(200.0, 100.0));
        let at = tree.state.downcast_ref::<State>().at + Duration::from_millis(60);
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        reveal.update(
            &mut tree,
            &Event::Window(window::Event::RedrawRequested(at)),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &(),
            &mut iced::advanced::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(Size::new(300.0, 500.0)),
        );
        assert_eq!(shell.redraw_request(), window::RedrawRequest::NextFrame);
        let height = reveal.layout(&mut tree, &(), &limits).size().height;
        assert!(height > 0.0 && height < 100.0);
    }

    #[test]
    fn opening_accepts_revealed_touch_immediately_and_ignores_hidden_touch() {
        let mut reveal = Reveal {
            content: Element::new(Probe),
            open: false,
        };
        let mut tree = Tree::new(&reveal as &dyn Widget<(), (), ()>);
        let start = Instant::now();
        reveal.open = true;
        let state = tree.state.downcast_mut::<State>();
        state.set_open(true, start);
        state.at = start + Duration::from_millis(60);
        let node = reveal.layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(300.0, 500.0)),
        );
        assert!(node.size().height > 10.0 && node.size().height < 99.0);
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        for (y, captured) in [(99.0, false), (10.0, true)] {
            reveal.update(
                &mut tree,
                &Event::Touch(iced::touch::Event::FingerPressed {
                    id: iced::touch::Finger(0),
                    position: iced::Point::new(10.0, y),
                }),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &(),
                &mut iced::advanced::clipboard::Null,
                &mut shell,
                &Rectangle::with_size(Size::new(300.0, 500.0)),
            );
            assert_eq!(shell.is_event_captured(), captured);
        }
        assert_eq!(
            tree.children[0].state.downcast_ref::<ProbeState>().updates,
            1
        );
    }
}
