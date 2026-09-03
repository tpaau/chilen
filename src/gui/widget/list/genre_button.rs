use std::sync::Arc;

use chilen_backend::music_lib::Genre;
use iced::{Alignment, Element, Length};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{button, column, container, row, text};

use crate::{
    THUMBNAIL_SIZE,
    gui::{
        SPACING_SMALL, SPACING_SMALLER, font, icons,
        widget::{
            cover_image::CoverImage,
            list::{BUTTON_PADDING, BUTTON_ROUNDING, button_style},
            text_spacer::text_spacer,
        },
    },
};

pub struct GenreButton<'a, Message>
where
    Message: 'a + Clone,
{
    pub theme: &'a dyn ColorScheme,
    pub genre: Arc<Genre>,
    pub play: Message,
    pub press: Message,
    pub shuffle: Message,
    pub add_to_queue: Message,
    pub highlighted: bool,
}

impl<'a, Message> From<GenreButton<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: GenreButton<'a, Message>) -> Self {
        let thumbnail_border_radius = BUTTON_ROUNDING - BUTTON_PADDING;

        let menu = iced_m3::widget::menu(
            vec![vertical_menu::Group {
                label: None,
                entries: vec![
                    vertical_menu::Entry::Button {
                        icon: Some(&icons::PLAY_ARROW),
                        label: "Play",
                        supporting_text: None,
                        error: false,
                        action: vertical_menu::Action::Message(Some(value.play)),
                    },
                    vertical_menu::Entry::Button {
                        icon: Some(&icons::SHUFFLE),
                        label: "Shuffle",
                        supporting_text: None,
                        error: false,
                        action: vertical_menu::Action::Message(Some(value.shuffle)),
                    },
                    vertical_menu::Entry::Button {
                        icon: Some(&icons::ADD_TO_QUEUE),
                        label: "Add to queue",
                        supporting_text: None,
                        error: false,
                        action: vertical_menu::Action::Message(Some(value.add_to_queue)),
                    },
                ],
            }],
            value.theme,
        )
        .icon_font(icons::filled());

        let font = if value.highlighted {
            font::font_bold()
        } else {
            font::font()
        };
        let title = text(value.genre.name.clone())
            .size(font::SIZE_REGULAR)
            .color(value.theme.on_surface())
            .font(font)
            .wrapping(text::Wrapping::None);

        // TODO: Maybe an animated indicator would look better?
        let icon_color = if value.highlighted {
            value.theme.on_secondary_container()
        } else {
            value.theme.on_surface_variant()
        };
        let container_color = if value.highlighted {
            value.theme.secondary_container()
        } else {
            value.theme.surface_container_high()
        };
        let image_path = (!value.highlighted)
            .then_some(value.genre.cover.thumbnail.clone())
            .flatten();
        let cover = CoverImage {
            image_path,
            icon: *icons::GENRES,
            icon_size: icons::SIZE_LARGE,
            icon_color,
            container_color,
            radius: thumbnail_border_radius.into(),
            opacity: 1.0,
            width: THUMBNAIL_SIZE.into(),
            height: THUMBNAIL_SIZE.into(),
        };

        button(
            row![
                cover,
                container(column![
                    title,
                    row![
                        text(match value.genre.artists.len() {
                            0 => "Unknown artist".to_string(),
                            1 => "1 artist".to_string(),
                            _ => format!("{} artists", value.genre.artists.len()),
                        })
                        .size(font::SIZE_SMALL)
                        .color(value.theme.on_surface_variant())
                        .wrapping(text::Wrapping::None),
                        text_spacer(value.theme.on_surface_variant(), font::SIZE_SMALL),
                        text(match value.genre.tracks.len() {
                            1 => "1 track".to_string(),
                            _ => format!("{} tracks", value.genre.albums.len()),
                        })
                        .size(font::SIZE_SMALL)
                        .color(value.theme.on_surface_variant())
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
                                .color(value.theme.on_surface())
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
        .on_press(value.press)
        .style(|_, status| button_style(status, value.theme.on_surface_variant()))
        .into()
    }
}
