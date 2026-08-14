use std::sync::Arc;

use chilen_backend::music_lib::state::Track;
use iced::Length;
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{Button, button, column, container, row, text};

use crate::gui::{
    Chilen, SPACING_SMALL, THUMBNAIL_SIZE, font, icons,
    widget::{
        cover_image::cover_image,
        list::{BUTTON_PADDING, BUTTON_ROUNDING, button_style},
    },
};

pub fn track_button<'a, Message: 'a + Clone>(
    state: &'a Chilen,
    track: Arc<Track>,
) -> Button<'a, Message> {
    let thumbnail_border_radius = BUTTON_ROUNDING - BUTTON_PADDING;

    let menu = iced_m3::widget::menu(
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
    .icon_font(icons::filled());

    button(
        row![
            cover_image(
                track.cover.thumbnail.clone(),
                &icons::MUSIC_NOTE,
                icons::SIZE_LARGE,
                state.theme.on_surface_variant(),
                state.theme.surface_container_high(),
                thumbnail_border_radius
            )
            .width(Length::Fixed(THUMBNAIL_SIZE))
            .height(Length::Fixed(THUMBNAIL_SIZE)),
            container(column![
                text(if let Some(title) = track.title.clone() {
                    title
                } else {
                    "Unknown".to_string()
                })
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_surface())
                .wrapping(text::Wrapping::None),
                text(
                    track
                        .artists
                        .as_ref()
                        .map(|a| a.join(&state.settings.value_separator))
                        .unwrap_or("Unknown".to_string())
                )
                .size(font::SIZE_SMALL)
                .color(state.theme.on_surface_variant())
                .wrapping(text::Wrapping::None),
            ])
            .width(Length::Fill)
            .clip(true)
            .center_y(Length::Fill),
            container(
                // TODO: Should be more like a button
                drop_down_menu(
                    |_| {
                        text(*icons::MORE_HORIZ)
                            .font(icons::filled())
                            .size(icons::SIZE_REGULAR)
                            .into()
                    },
                    Some(menu),
                    iced_m3::widget::drop_down_menu::Placement::BottomRight,
                ),
            )
            .center_y(Length::Fill),
        ]
        .spacing(SPACING_SMALL),
    )
    .padding(BUTTON_PADDING)
    .style(|_, status| button_style(status, state.theme.on_surface_variant()))
}
