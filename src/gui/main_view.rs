use iced::{Border, Element, Length, Padding};
use iced_m3::theme::ColorScheme;
use iced_widget::{center, column, container};

use crate::gui::{Chilen, Message, ROUNDING_REGULAR, SPACING_SMALL, icons};

#[derive(Debug, Clone, Default)]
pub enum View {
    #[default]
    Tracks,
    Albums,
    Artists,
    Genres,
}

pub fn view(state: &Chilen) -> Element<'_, Message> {
    container(column![
        // TODO: Custom ordering
        {
            let index = match state.main_view {
                View::Tracks => 0,
                View::Albums => 1,
                View::Artists => 2,
                View::Genres => 3,
            };
            iced_m3::widget::navbar::Navbar::new(
                vec![
                    iced_m3::widget::navbar::Item {
                        icon: &icons::MUSIC_NOTE,
                        label: "Tracks",
                        message: Message::ChangeMainView(View::Tracks),
                    },
                    iced_m3::widget::navbar::Item {
                        icon: &icons::ALBUM,
                        label: "Albums",
                        message: Message::ChangeMainView(View::Albums),
                    },
                    iced_m3::widget::navbar::Item {
                        icon: &icons::ARTIST,
                        label: "Artists",
                        message: Message::ChangeMainView(View::Artists),
                    },
                    iced_m3::widget::navbar::Item {
                        icon: &icons::GENRES,
                        label: "Genres",
                        message: Message::ChangeMainView(View::Genres),
                    },
                ],
                &state.theme,
            )
            .button_width(110.0)
            .focused_index(index)
            .icon_font_active(icons::filled())
            .icon_font_inactive(icons::outlined())
        },
        {
            let content = match state.main_view {
                View::Tracks => "Track view",
                View::Albums => "Album view",
                View::Artists => "Artist view",
                View::Genres => "Genre view",
            };
            center(content)
        }
    ])
    .style(|_| {
        container::Style::default()
            .background(state.theme.background())
            .border(Border::default().rounded(ROUNDING_REGULAR))
    })
    .padding(Padding::new(SPACING_SMALL as f32))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
