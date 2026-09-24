use iced::{Border, Element, Length, padding};
use iced_core::text::IntoFragment;
use iced_m3::{theme::ColorScheme, widget::hybrid_icon::Icon};
use iced_widget::{column, container};

use crate::gui::{Chilen, ROUNDING_LARGER, SPACING_REGULAR, font, icons, playback_view::Message};

mod lyrics;
mod queue;

const PANEL_ROUNDING: f32 = ROUNDING_LARGER;
const PANEL_PADDING: f32 = SPACING_REGULAR;

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let navbar = iced_m3::widget::navbar::<Message, iced::Theme, iced::Renderer>(
        vec![
            iced_m3::widget::navbar::Item {
                icon_active: Icon::Text {
                    text: icons::QUEUE_MUSIC.into_fragment(),
                    font: Some(icons::filled()),
                },
                icon_inactive: Icon::Text {
                    text: icons::QUEUE_MUSIC.into_fragment(),
                    font: Some(icons::outlined()),
                },
                label: "Queue".into(),
                message: Message::OpenQueue,
            },
            iced_m3::widget::navbar::Item {
                icon_active: Icon::Text {
                    text: icons::LYRICS.into_fragment(),
                    font: Some(icons::filled()),
                },
                icon_inactive: Icon::Text {
                    text: icons::LYRICS.into_fragment(),
                    font: Some(icons::outlined()),
                },
                label: "Lyrics".into(),
                message: Message::OpenLyrics,
            },
        ],
        font::regular(),
        &state.theme,
    )
    .focused_index(match state.playback_view.tab {
        super::Tab::Queue => 0,
        super::Tab::Lyrics => 1,
    });

    let content = match state.playback_view.tab {
        super::Tab::Lyrics => lyrics::view(state, PANEL_PADDING / 2.0),
        super::Tab::Queue => queue::view(state, PANEL_PADDING / 2.0),
    };

    container(
        container(
            column![navbar, content]
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .padding(padding::horizontal(PANEL_PADDING / 2.0).bottom(PANEL_PADDING / 2.0)),
    )
    .style(|_| {
        iced_widget::container::Style::default()
            .background(state.theme.surface())
            .border(Border::default().rounded(PANEL_ROUNDING))
    })
    .into()
}
