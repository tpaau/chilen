use std::sync::Arc;

use chilen_backend::music_lib::Artist;
use iced::{Alignment, Length};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{Button, button, column, container, row, text};

use crate::{
    THUMBNAIL_SIZE,
    gui::{
        SPACING_SMALL, SPACING_SMALLER, font, icons,
        widget::{
            cover_image::cover_image,
            list::{BUTTON_PADDING, button_style},
            text_spacer::text_spacer,
        },
    },
};

pub fn artist_button<'a, Message: 'a + Clone>(
    theme: &'a impl ColorScheme,
    artist: Arc<Artist>,
    play_message: Message,
    shuffle_message: Message,
    highlighted: bool,
) -> Button<'a, Message> {
    let menu = iced_m3::widget::menu(
        vec![vertical_menu::Group {
            label: None,
            entries: vec![
                vertical_menu::Entry::Button {
                    icon: Some(&icons::PLAY_ARROW),
                    label: "Play",
                    supporting_text: None,
                    error: false,
                    action: vertical_menu::Action::Message(Some(play_message)),
                },
                vertical_menu::Entry::Button {
                    icon: Some(&icons::SHUFFLE),
                    label: "Shuffle",
                    supporting_text: None,
                    error: false,
                    action: vertical_menu::Action::Message(Some(shuffle_message)),
                },
            ],
        }],
        theme,
    )
    .icon_font(icons::filled());

    let font = if highlighted {
        font::font_bold()
    } else {
        font::font()
    };
    let title = text(artist.name.clone())
        .size(font::SIZE_REGULAR)
        .color(theme.on_surface())
        .font(font)
        .wrapping(text::Wrapping::None);

    // TODO: Maybe an animated indicator would look better?
    let content_color = if highlighted {
        theme.on_secondary_container()
    } else {
        theme.on_surface_variant()
    };
    let container_color = if highlighted {
        theme.secondary_container()
    } else {
        theme.surface_container_high()
    };
    let cover = cover_image(
        (!highlighted)
            .then_some(artist.cover.thumbnail.clone())
            .flatten(),
        &icons::ARTIST,
        icons::SIZE_LARGE,
        content_color,
        container_color,
        f32::MAX,
    )
    .width(Length::Fixed(THUMBNAIL_SIZE))
    .height(Length::Fixed(THUMBNAIL_SIZE));

    button(
        row![
            cover,
            container(column![
                title,
                row![
                    text(match artist.albums.len() {
                        0 => "No albums".to_string(),
                        1 => "1 album".to_string(),
                        _ => format!("{} albums", artist.albums.len()),
                    })
                    .size(font::SIZE_SMALL)
                    .color(theme.on_surface_variant())
                    .wrapping(text::Wrapping::None),
                    text_spacer(theme.on_surface_variant(), font::SIZE_SMALL),
                    text(match artist.tracks.len() {
                        1 => "1 track".to_string(),
                        _ => format!("{} tracks", artist.tracks.len()),
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
