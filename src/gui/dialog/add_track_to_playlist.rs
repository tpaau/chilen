use std::sync::Arc;

use chilen_backend::music_lib::{Playlist, Track};
use iced::{Alignment, Element, Length};
use iced_m3::{
    theme::ColorScheme,
    widget::{dialog, spacer},
};
use iced_widget::{button, column, container, row, scrollable, text};

use crate::{
    THUMBNAIL_SIZE,
    gui::{
        Chilen, Message, ROUNDING_LARGE, SPACING_SMALL,
        font::{self, font_bold},
        icons,
        widget::{
            cover_image::CoverImage,
            list::{BUTTON_PADDING, BUTTON_SPACING},
        },
    },
};

fn playlist_choice<'a>(
    theme: &'a impl ColorScheme,
    track: Arc<Track>,
    playlist: &'a Arc<Playlist>,
) -> Element<'a, Message> {
    button(
        row![
            CoverImage {
                image_path: playlist.cover.thumbnail.clone(),
                icon: *icons::PLAYLIST_PLAY,
                icon_size: icons::SIZE_LARGE,
                icon_color: theme.on_surface_variant(),
                container_color: theme.surface_container(),
                radius: ROUNDING_LARGE.into(),
                opacity: 1.0,
                width: THUMBNAIL_SIZE.into(),
                height: THUMBNAIL_SIZE.into()
            },
            text(&playlist.name)
                .color(theme.on_surface())
                .font(font_bold())
                .wrapping(text::Wrapping::None)
                .size(font::SIZE_LARGE)
        ]
        .align_y(Alignment::Center)
        .spacing(BUTTON_SPACING),
    )
    .padding(BUTTON_PADDING)
    .width(Length::Fill)
    .on_press(Message::AddTrackToPlaylist {
        track,
        playlist: playlist.name.clone(),
    })
    .style(|_, status| crate::gui::widget::list::button_style(status, theme.on_surface()))
    .into()
}

pub(super) fn view<'a>(state: &'a Chilen, track: Arc<Track>) -> Element<'a, Message> {
    let choices: Element<'_, Message> = match &state.library {
        Some(lib) => column(
            lib.playlists
                .iter()
                .map(|p| playlist_choice(&state.theme, track.clone(), p)),
        )
        .into(),
        None => text("no library!!").into(),
    };
    let content = column![
        spacer(&state.theme),
        container(
            scrollable(choices).style(|_, status| iced_m3::style::scrollable(status, &state.theme)),
        )
        .max_height(400.0)
    ]
    .spacing(SPACING_SMALL);

    let buttons = vec![iced_m3::widget::dialog::Button {
        on_press: Some(Message::CloseDialog),
        label: "Cancel".to_string(),
        style: iced_m3::widget::button::Style::Outlined,
    }];
    dialog(&state.theme, content, buttons)
        .title_font(font::font_bold())
        .title("Add to playlist")
        .icon_font(icons::filled())
        .icon(*icons::PLAYLIST_ADD)
        .into()
}
