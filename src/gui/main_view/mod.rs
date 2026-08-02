mod albums;
mod artists;
mod genres;
mod tracks;

use std::sync::Arc;

use chilen_backend::music_lib::state::Track;
use iced::{Border, Color, Element, Length, Padding};
use iced_m3::{HOVER_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY, theme::ColorScheme};
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

pub struct State {
    pub view: View,
}

fn button_style(
    status: iced_widget::button::Status,
    theme: &impl ColorScheme,
) -> iced_widget::button::Style {
    let content_color = theme.on_surface_variant();
    iced_widget::button::Style {
        background: Some(iced::Background::Color(match status {
            iced_widget::button::Status::Active => Color::TRANSPARENT,
            iced_widget::button::Status::Hovered => {
                content_color.scale_alpha(HOVER_STATE_LAYER_OPACITY)
            }
            iced_widget::button::Status::Pressed => {
                content_color.scale_alpha(PRESSED_STATE_LAYER_OPACITY)
            }
            iced_widget::button::Status::Disabled => {
                unreachable!("There should be no inactive buttons in the main view")
            }
        })),
        text_color: content_color,
        border: Border::default().rounded(ROUNDING_REGULAR),
        ..Default::default()
    }
}

pub const THUMBNAIL_SIZE: f32 = 64.0;

pub fn view(state: &Chilen) -> Element<'_, Message> {
    container(column![
        // TODO: Custom ordering
        {
            let index = match state.main_view.view {
                View::Tracks => 0,
                View::Albums => 1,
                View::Artists => 2,
                View::Genres => 3,
            };
            iced_m3::widget::navbar::<_, iced::Theme, iced::Renderer>(
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
            .focused_index(index)
            .icon_font_active(icons::filled())
            .icon_font_inactive(icons::outlined())
        },
        {
            let content = match state.main_view.view {
                View::Tracks => tracks::view(state),
                View::Albums => albums::view(state),
                View::Artists => artists::view(state),
                View::Genres => genres::view(state),
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
