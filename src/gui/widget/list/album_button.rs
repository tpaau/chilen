use std::sync::Arc;

use chilen_backend::music_lib::Album;
use iced::{Alignment, Element, Length};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{button, column, container, row, text};

use crate::gui::{
    Chilen, SPACING_SMALL, SPACING_SMALLER, font,
    formatter::format_date,
    icons::{self, icon_filled},
    widget::{
        cover_image::CoverImage,
        list::{BUTTON_PADDING, BUTTON_ROUNDING, THUMBNAIL_SIZE, button_style},
        text_spacer::text_spacer,
    },
};

pub enum Info {
    ArtistCount,
    TrackCount,
    Date,
}

pub struct AlbumButton<'a, Message> {
    pub state: &'a Chilen,
    pub album: Arc<Album>,
    pub info: Vec<Info>,
    pub play: Message,
    pub press: Message,
    pub shuffle: Message,
    pub add_to_queue: Message,
    pub highlighted: bool,
}

impl<'a, Message> From<AlbumButton<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: AlbumButton<'a, Message>) -> Self {
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
            &value.state.theme,
        )
        .icon_font(icons::filled());

        let info: Vec<_> = value
            .info
            .into_iter()
            .map(|i| match i {
                Info::ArtistCount => match value.album.artists.len() {
                    1 => "1 artist".to_string(),
                    _ => format!("{} artists", value.album.artists.len()),
                },
                Info::TrackCount => match value.album.tracks.len() {
                    1 => "Single".to_string(),
                    _ => format!("{} tracks", value.album.tracks.len()),
                },
                Info::Date => match value.album.date {
                    Some(date) => format_date(date),
                    None => "Unknown date".to_string(),
                },
            })
            .collect();

        let info_len = info.len();
        let mut info_text: Vec<Element<'_, Message>> = Vec::with_capacity(2 * info_len - 1);
        for (i, info) in info.into_iter().enumerate() {
            info_text.push(
                text(info)
                    .size(font::SIZE_SMALL)
                    .color(value.state.theme.on_surface_variant())
                    .wrapping(text::Wrapping::None)
                    .into(),
            );
            if let Some(len) = info_len.checked_sub(1)
                && i < len
            {
                info_text.push(text_spacer(
                    value.state.theme.on_surface_variant(),
                    font::SIZE_SMALL,
                ));
            }
        }

        let font = if value.highlighted {
            font::bold()
        } else {
            font::regular()
        };
        let title = text(value.album.title.clone())
            .size(font::SIZE_REGULAR)
            .color(value.state.theme.on_surface())
            .font(font)
            .wrapping(text::Wrapping::None);

        // TODO: Maybe an animated indicator would look better?
        let (icon_color, container_color) = match value.highlighted {
            true => (
                value.state.theme.on_secondary_container(),
                value.state.theme.secondary_container(),
            ),
            false => (
                value.state.theme.on_surface_variant(),
                value.state.theme.surface_container_high(),
            ),
        };
        let image_path = (!value.highlighted)
            .then_some(value.album.cover.thumbnail.clone())
            .flatten();

        let cover = CoverImage {
            image_path,
            icon: *icons::ALBUM,
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
                    row(info_text)
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
                                .color(value.state.theme.on_surface())
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
        .style(|_, status| button_style(status, value.state.theme.on_surface_variant()))
        .on_press(value.press)
        .into()
    }
}
