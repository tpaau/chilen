use std::sync::Arc;

use chilen_backend::music_lib::state::Track;
use iced::{
    Background, Border, Length, Padding, Shadow, color,
    widget::{Button, button, column, container, row, space, text},
};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};

use crate::gui::{
    Chilen, DIM_TEXT_ALPHA, Message, ROUNDING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
    font::{SIZE_REGULAR, SIZE_SMALL},
    icons,
    widgets::THUMBNAIL_SIZE,
};

pub fn track_button<'a>(state: &'a Chilen, track: &'a Arc<Track>) -> Button<'a, Message> {
    button(
        row(vec![
            container(space())
                .style(|_| {
                    container::Style::default()
                        .background(color!(0xff0000))
                        .border(Border::default().rounded(ROUNDING_REGULAR - SPACING_SMALLER))
                })
                .width(Length::Fixed(THUMBNAIL_SIZE))
                .height(Length::Fixed(THUMBNAIL_SIZE))
                .into(),
            container(column(vec![
                text(if let Some(title) = &track.title {
                    title.clone()
                } else {
                    "Unknown".to_string()
                })
                .size(SIZE_REGULAR)
                .color(state.theme.on_surface())
                .wrapping(text::Wrapping::None)
                .into(),
                text(if let Some(artist) = &track.artist {
                    artist.clone()
                } else {
                    "Unknown".to_string()
                })
                .size(SIZE_SMALL)
                .color(state.theme.on_surface().scale_alpha(DIM_TEXT_ALPHA))
                .into(),
            ]))
            .width(Length::Fill)
            .clip(true)
            .center_y(Length::Fill)
            .into(),
            container(
                // TODO: Should be more like a button
                drop_down_menu(
                    |_| {
                        text(*icons::MORE_HORIZ)
                            .font(icons::filled())
                            .size(icons::SIZE_REGULAR)
                            .into()
                    },
                    Some(
                        iced_m3::widget::menu(
                            vec![
                                vertical_menu::Group {
                                    label: None,
                                    entries: vec![
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::PLAY_ARROW),
                                            label: "Play",
                                            supporting_text: None,
                                            error: false,
                                            action: vertical_menu::Action::Message(None),
                                        },
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::SHUFFLE),
                                            label: "Shuffle",
                                            supporting_text: None,
                                            error: false,
                                            action: vertical_menu::Action::Message(None),
                                        },
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::ADD_TO_QUEUE),
                                            label: "Add to queue",
                                            supporting_text: None,
                                            error: false,
                                            action: vertical_menu::Action::Message(None),
                                        },
                                    ],
                                },
                                vertical_menu::Group {
                                    label: None,
                                    entries: vec![vertical_menu::Entry::Button {
                                        icon: Some(&icons::UPLOAD),
                                        label: "Details",
                                        supporting_text: None,
                                        error: false,
                                        action: vertical_menu::Action::Message(None),
                                    }],
                                },
                            ],
                            &state.theme,
                        )
                        .icon_font(icons::filled())
                        .vibrant(false),
                    ),
                    iced_m3::widget::drop_down_menu::Placement::BottomRight,
                ),
            )
            .center_y(Length::Fill)
            .into(),
        ])
        .spacing(SPACING_SMALL),
    )
    .padding(Padding::new(SPACING_SMALLER as f32))
    .style(|_, status| {
        let style = button::Style {
            background: Some(Background::Color(state.theme.surface_container())),
            text_color: state.theme.on_surface(),
            border: Border::default().rounded(ROUNDING_REGULAR),
            shadow: Shadow::default(),
            snap: true,
        };

        match status {
            button::Status::Hovered => {
                style.with_background(Background::Color(state.theme.surface_container_high()))
            }
            _ => style,
        }
    })
    .on_press(Message::CloseDialog)
}
