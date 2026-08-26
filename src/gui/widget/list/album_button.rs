use std::sync::Arc;

use chilen_backend::music_lib::Album;
use iced::{Alignment, Element, Length};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{Button, button, column, container, row, text};

use crate::gui::{
    Chilen, SPACING_SMALL, SPACING_SMALLER, font,
    formatter::format_date,
    icons,
    widget::{
        cover_image::cover_image,
        list::{BUTTON_PADDING, BUTTON_ROUNDING, THUMBNAIL_SIZE, button_style},
        text_spacer::text_spacer,
    },
};

pub enum Info {
    ArtistCount,
    TrackCount,
    Date,
}

pub fn album_button<'a, Message: 'a + Clone>(
    state: &'a Chilen,
    album: Arc<Album>,
    info: Vec<Info>,
    play_message: Message,
    shuffle_message: Message,
) -> Button<'a, Message> {
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
        &state.theme,
    )
    .icon_font(icons::filled());

    let info: Vec<_> = info
        .into_iter()
        .map(|i| match i {
            Info::ArtistCount => match album.artists.len() {
                1 => "1 artist".to_string(),
                _ => format!("{} artists", album.artists.len()),
            },
            Info::TrackCount => match album.tracks.len() {
                1 => "Single".to_string(),
                _ => format!("{} tracks", album.tracks.len()),
            },
            Info::Date => match album.date {
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
                .color(state.theme.on_surface_variant())
                .wrapping(text::Wrapping::None)
                .into(),
        );
        if let Some(len) = info_len.checked_sub(1)
            && i < len
        {
            info_text.push(text_spacer(
                state.theme.on_surface_variant(),
                font::SIZE_SMALL,
            ));
        }
    }

    button(
        row![
            cover_image(
                album.cover.thumbnail.clone(),
                &icons::ALBUM,
                icons::SIZE_LARGE,
                state.theme.on_surface_variant(),
                state.theme.surface_container_high(),
                thumbnail_border_radius
            )
            .width(Length::Fixed(THUMBNAIL_SIZE))
            .height(Length::Fixed(THUMBNAIL_SIZE)),
            container(column![
                text(album.title.clone())
                    .size(font::SIZE_REGULAR)
                    .color(state.theme.on_surface())
                    .wrapping(text::Wrapping::None),
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
