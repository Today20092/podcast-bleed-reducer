use crate::appearance::colors;
use iced::{
    Point, Radians, Rectangle, Renderer, Size, Theme, mouse,
    widget::canvas::{self, Frame, Geometry, Path, Stroke, path},
};

#[derive(Clone, Copy)]
pub(crate) enum ToolbarIcon {
    Lock { locked: bool },
    Center,
    Meters { at_playhead: bool },
}

impl<Message> canvas::Program<Message> for ToolbarIcon {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let c = colors(theme);
        let mut frame = Frame::new(renderer, bounds.size());
        let stroke = Stroke::default().with_color(c.text).with_width(1.6);
        match self {
            Self::Lock { locked } => {
                let shackle = Path::new(|builder| {
                    let center = if *locked {
                        Point::new(10.0, 8.0)
                    } else {
                        Point::new(12.5, 6.0)
                    };
                    builder.arc(path::Arc {
                        center,
                        radius: 4.0,
                        start_angle: Radians(std::f32::consts::PI),
                        end_angle: Radians(std::f32::consts::TAU),
                    });
                    if !locked {
                        builder.move_to(Point::new(8.5, 6.0));
                        builder.line_to(Point::new(8.5, 9.0));
                    }
                });
                frame.stroke(&shackle, stroke);
                let body = Path::rectangle(Point::new(4.5, 9.0), Size::new(11.0, 8.0));
                frame.fill(&body, c.control);
                frame.stroke(&body, stroke);
                frame.fill(&Path::circle(Point::new(10.0, 12.5), 1.0), c.text);
            }
            Self::Center => {
                let arrows = Path::new(|builder| {
                    for (a, b) in [
                        ((1.0, 10.0), (7.0, 10.0)),
                        ((4.5, 7.5), (7.0, 10.0)),
                        ((4.5, 12.5), (7.0, 10.0)),
                        ((13.0, 10.0), (19.0, 10.0)),
                        ((15.5, 7.5), (13.0, 10.0)),
                        ((15.5, 12.5), (13.0, 10.0)),
                        ((10.0, 2.0), (10.0, 18.0)),
                    ] {
                        builder.move_to(Point::new(a.0, a.1));
                        builder.line_to(Point::new(b.0, b.1));
                    }
                });
                frame.stroke(&arrows, stroke);
            }
            Self::Meters { at_playhead } => {
                for (x, height) in [(5.0, 8.0), (9.0, 14.0), (13.0, 11.0)] {
                    frame.fill(
                        &Path::rectangle(Point::new(x, 17.0 - height), Size::new(2.0, height)),
                        c.text,
                    );
                }
                let x = if *at_playhead { 2.0 } else { 18.0 };
                frame.stroke(
                    &Path::line(Point::new(x, 2.0), Point::new(x, 18.0)),
                    stroke.with_color(c.focus),
                );
            }
        }
        vec![frame.into_geometry()]
    }
}
