use std::sync::Arc;

use chilen_backend::music_lib::Artist;
use iced::{Alignment, Element, Length};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{button, column, container, row, text};

use crate::{
    THUMBNAIL_SIZE,
    gui::{
        SPACING_SMALL, SPACING_SMALLER, font,
        icons::{self, icon_filled},
        widget::{
            cover_image::CoverImage,
            list::{BUTTON_PADDING, button_style},
            text_spacer::text_spacer,
        },
    },
};

pub struct ArtistButton<'a, Message>
where
    Message: 'a + Clone,
{
    pub theme: &'a dyn ColorScheme,
    pub artist: Arc<Artist>,
    pub play: Message,
    pub press: Message,
    pub shuffle: Message,
    pub add_to_queue: Message,
    pub highlighted: bool,
}

impl<'a, Message> From<ArtistButton<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: ArtistButton<'a, Message>) -> Self {
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
            font::bold()
        } else {
            font::regular()
        };
        let title = text(value.artist.name.clone())
            .size(font::SIZE_REGULAR)
            .color(value.theme.on_surface())
            .font(font)
            .wrapping(text::Wrapping::None);

        // TODO: Maybe an animated indicator would look better?
        let (icon_color, container_color) = match value.highlighted {
            true => (
                value.theme.on_secondary_container(),
                value.theme.secondary_container(),
            ),
            false => (
                value.theme.on_surface_variant(),
                value.theme.surface_container_high(),
            ),
        };
        let image_path = (!value.highlighted)
            .then_some(value.artist.cover.thumbnail.clone())
            .flatten();

        let cover = CoverImage {
            image_path,
            icon: *icons::ARTIST,
            icon_size: icons::SIZE_LARGE,
            icon_color,
            container_color,
            radius: f32::MAX.into(),
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
                        text(match value.artist.albums.len() {
                            0 => "No albums".to_string(),
                            1 => "1 album".to_string(),
                            _ => format!("{} albums", value.artist.albums.len()),
                        })
                        .size(font::SIZE_SMALL)
                        .color(value.theme.on_surface_variant())
                        .wrapping(text::Wrapping::None),
                        text_spacer(value.theme.on_surface_variant(), font::SIZE_SMALL),
                        text(match value.artist.tracks.len() {
                            1 => "1 track".to_string(),
                            _ => format!("{} tracks", value.artist.tracks.len()),
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
                            icon_filled(*icons::MORE_HORIZ)
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
        .style(|_, status| button_style(status, value.theme.on_surface_variant()))
        .on_press(value.press)
        .into()
    }
}
