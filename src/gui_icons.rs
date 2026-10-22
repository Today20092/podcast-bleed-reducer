use crate::appearance::colors;
use iced::{
    Color, Element, Theme,
    widget::{Space, container, stack, svg},
};

#[derive(Clone, Copy)]
pub(crate) enum ToolbarIcon {
    Waveform(crate::waveform::WaveformDisplay),
    CatchUp,
    Lock { locked: bool },
    Center,
    Meters { at_playhead: bool },
    Previous,
    Play,
    Pause,
    Stop,
}

impl ToolbarIcon {
    pub(crate) fn view<Message: 'static>(self, tint: Option<Color>) -> Element<'static, Message> {
        let bytes: &'static [u8] = match self {
            Self::Waveform(crate::waveform::WaveformDisplay::Original) => {
                include_bytes!("../assets/ui/waveform-original.svg")
            }
            Self::Waveform(crate::waveform::WaveformDisplay::Processed) => {
                include_bytes!("../assets/ui/waveform-processed.svg")
            }
            Self::Waveform(crate::waveform::WaveformDisplay::Both) => {
                include_bytes!("../assets/ui/waveform-both.svg")
            }
            Self::CatchUp => include_bytes!("../assets/lucide/arrow-right-to-line.svg"),
            Self::Lock { locked: true } => include_bytes!("../assets/lucide/lock-keyhole.svg"),
            Self::Lock { locked: false } => {
                include_bytes!("../assets/lucide/lock-keyhole-open.svg")
            }
            Self::Center => include_bytes!("../assets/lucide/fold-horizontal.svg"),
            Self::Meters { .. } => include_bytes!("../assets/lucide/audio-lines.svg"),
            Self::Previous => include_bytes!("../assets/lucide/skip-back.svg"),
            Self::Play => include_bytes!("../assets/lucide/play.svg"),
            Self::Pause => include_bytes!("../assets/lucide/pause.svg"),
            Self::Stop => include_bytes!("../assets/lucide/square.svg"),
        };
        let icon = svg(svg::Handle::from_memory(bytes))
            .width(20)
            .height(20)
            .style(move |theme: &Theme, _| svg::Style {
                color: Some(tint.unwrap_or_else(|| colors(theme).text)),
            });
        if let Self::Meters { at_playhead } = self {
            let marker =
                container(Space::new().width(1).height(16)).style(|theme| container::Style {
                    background: Some(colors(theme).focus.into()),
                    ..container::Style::default()
                });
            let marker = container(marker).width(24).center_y(20);
            let marker = if at_playhead {
                marker.align_left(24)
            } else {
                marker.align_right(24)
            };
            // Keep the existing placement cue separate from the upstream artwork.
            stack![container(icon).center_x(24), marker].into()
        } else {
            icon.into()
        }
    }
}
