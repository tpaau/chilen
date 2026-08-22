use std::sync::Arc;

use chilen_backend::music_lib::Album;
use iced::{Alignment, Length};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{Button, button, column, container, row, text};

use crate::gui::{
    SPACING_SMALL, SPACING_SMALLER, font, icons,
    widget::{
        cover_image::cover_image,
        list::{BUTTON_PADDING, BUTTON_ROUNDING, THUMBNAIL_SIZE, button_style},
        text_spacer::text_spacer,
    },
};

pub fn album_button<'a, Message: 'a + Clone>(
    theme: &'a impl ColorScheme,
    album: Arc<Album>,
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
                        icon: Some(&icons::SHUFFLE),
                        label: "Shuffle",
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
        theme,
    )
    .icon_font(icons::filled());

    button(
        row![
            cover_image(
                album.cover.thumbnail.clone(),
                &icons::ALBUM,
                icons::SIZE_LARGE,
                theme.on_surface_variant(),
                theme.surface_container_high(),
                thumbnail_border_radius
            )
            .width(Length::Fixed(THUMBNAIL_SIZE))
            .height(Length::Fixed(THUMBNAIL_SIZE)),
            container(column![
                text(album.title.clone())
                    .size(font::SIZE_REGULAR)
                    .color(theme.on_surface())
                    .wrapping(text::Wrapping::None),
                row![
                    text(match album.tracks.len() {
                        1 => "Single".to_string(),
                        _ => format!("{} tracks", album.tracks.len()),
                    })
                    .size(font::SIZE_SMALL)
                    .color(theme.on_surface_variant())
                    .wrapping(text::Wrapping::None),
                    text_spacer(theme.on_surface_variant(), font::SIZE_SMALL),
                    text(match album.artists.len() {
                        1 => "1 artist".to_string(),
                        _ => format!("{} artists", album.artists.len()),
                    })
                    .size(font::SIZE_SMALL)
                    .color(theme.on_surface_variant())
                    .wrapping(text::Wrapping::None),
                ]
                .align_y(Alignment::Center)
                .spacing(SPACING_SMALLER),
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
    .style(|_, status| button_style(status, theme.on_surface_variant()))
}
