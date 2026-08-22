use std::sync::Arc;

use chilen_backend::music_lib::Playlist;
use iced::{Alignment, Element, Length};
use iced_m3::theme::ColorScheme;
use iced_widget::{column, container, responsive, row, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALLER, font, icons,
    main_view::top_view::{
        MAX_COVER_SIZE, MIN_COVER_SIZE, Message, format_duration, horizontal_buttons, spacer, title,
    },
    widget::{self, cover_image::cover_image, list::BUTTON_SPACING, text_spacer::text_spacer},
};

pub(super) fn view<'a>(state: &'a Chilen, playlist: Arc<Playlist>) -> Element<'a, Message> {
    let playlist_cloned = playlist.clone();
    let display = responsive(move |size| {
        let has_tracks = !playlist_cloned.tracks.is_empty();
        let track_count_text = if !has_tracks {
            "No tracks".to_string()
        } else if playlist_cloned.tracks.len() == 1 {
            "1 track".to_string()
        } else {
            format!("{} tracks", playlist_cloned.tracks.len())
        };
        let cover_size = (size.width.min(size.height) / 3.0).clamp(MIN_COVER_SIZE, MAX_COVER_SIZE);

        let cover = cover_image(
            playlist_cloned.cover.hires.clone(),
            &icons::PLAYLIST_PLAY,
            cover_size / 4.0,
            state.theme.on_surface_variant(),
            state.theme.surface_container(),
            ROUNDING_LARGE,
        )
        .width(cover_size)
        .height(cover_size);

        let item_data = column![
            text("Playlist")
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_surface_variant()),
            title(&state.theme, playlist_cloned.name.clone()),
            row![
                text(track_count_text)
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface()),
                has_tracks.then_some(text_spacer(
                    state.theme.on_surface_variant(),
                    font::SIZE_LARGE
                )),
                has_tracks.then_some(
                    text(format_duration(playlist_cloned.duration))
                        .size(font::SIZE_LARGE)
                        .color(state.theme.on_surface())
                )
            ]
            .align_y(Alignment::Center)
            .spacing(SPACING_SMALLER),
        ]
        .align_x(Alignment::Start);

        row![cover, item_data]
            .align_y(Alignment::Center)
            .spacing(SPACING_REGULAR)
            .into()
    })
    .height(Length::Shrink)
    .width(Length::Shrink);

    let buttons = horizontal_buttons(&state.theme, Message::Noop, Message::Noop, Message::Noop);

    let track_buttons: Vec<_> = playlist
        .tracks
        .iter()
        .map(|t| {
            widget::list::track_button::track_button(state, t.clone())
                .on_press(Message::Noop)
                .into()
        })
        .collect();

    let tracks_section: Element<'_, Message> = if track_buttons.is_empty() {
        container(
            text("No tracks yet!")
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_surface_variant()),
        )
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
    } else {
        column(track_buttons).spacing(BUTTON_SPACING).into()
    };

    column![display, buttons, spacer(&state.theme), tracks_section]
        .width(Length::Fill)
        .spacing(SPACING_REGULAR)
        .into()
}
