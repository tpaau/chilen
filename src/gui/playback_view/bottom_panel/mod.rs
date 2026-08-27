use iced::{Border, Element, Length, padding};
use iced_m3::theme::ColorScheme;
use iced_widget::{column, container};

use crate::gui::{Chilen, ROUNDING_LARGER, SPACING_REGULAR, icons, playback_view::Message};

mod lyrics;
mod queue;

const PANEL_ROUNDING: f32 = ROUNDING_LARGER;
const PANEL_PADDING: f32 = SPACING_REGULAR;

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let custom_navbar = iced_m3::widget::navbar::<Message, iced::Theme, iced::Renderer>(
        vec![
            iced_m3::widget::navbar::Item {
                icon: &icons::QUEUE_MUSIC,
                label: "Queue",
                message: Message::OpenQueue,
            },
            iced_m3::widget::navbar::Item {
                icon: &icons::LYRICS,
                label: "Lyrics",
                message: Message::OpenLyrics,
            },
        ],
        &state.theme,
    )
    .focused_index(match state.playback_view.tab {
        super::Tab::Queue => 0,
        super::Tab::Lyrics => 1,
    })
    .icon_font_active(icons::filled())
    .icon_font_inactive(icons::outlined());

    let content = match state.playback_view.tab {
        super::Tab::Lyrics => lyrics::view(state),
        super::Tab::Queue => queue::view(state),
    };

    container(
        container(
            column![custom_navbar, content]
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .padding(padding::horizontal(PANEL_PADDING).bottom(PANEL_PADDING)),
    )
    .style(|_| {
        iced_widget::container::Style::default()
            .background(state.theme.surface())
            .border(Border::default().rounded(PANEL_ROUNDING))
    })
    .into()
}
