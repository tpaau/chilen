use std::sync::Arc;

use chilen_backend::music_lib::state::Genre;
use iced::{Alignment, Element, Length};
use iced_m3::theme::ColorScheme;
use iced_widget::{column, container, responsive, row, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALLER, font, icons,
    main_view::top_view::{
        MAX_COVER_SIZE, MIN_COVER_SIZE, Message, horizontal_buttons, spacer, title, unwind_button,
    },
    widget::{self, cover_image::cover_image, list::BUTTON_SPACING, text_spacer::text_spacer},
};

pub(super) fn view<'a>(state: &'a Chilen, genre: Arc<Genre>) -> Element<'a, Message> {
    let genre_cloned = genre.clone();
    let display = responsive(move |size| {
        let cover_size = (size.width.min(size.height) / 3.0).clamp(MIN_COVER_SIZE, MAX_COVER_SIZE);

        let track_count_text = if genre_cloned.tracks.len() == 1 {
            "1 track".to_string()
        } else {
            format!("{} tracks", genre_cloned.tracks.len())
        };

        let artist_count_text = if genre_cloned.artists.is_empty() {
            None
        } else if genre_cloned.artists.len() == 1 {
            Some("1 artist".to_string())
        } else {
            Some(format!("{} artists", genre_cloned.artists.len()))
        };

        let album_count_text = if genre_cloned.albums.is_empty() {
            None
        } else if genre_cloned.albums.len() == 1 {
            Some("1 album".to_string())
        } else {
            Some(format!("{} albums", genre_cloned.albums.len()))
        };

        let cover = cover_image(
            genre_cloned.cover.hires.clone(),
            &icons::ALBUM,
            cover_size / 4.0,
            state.theme.on_surface_variant(),
            state.theme.surface_container(),
            ROUNDING_LARGE,
        )
        .width(cover_size)
        .height(cover_size);

        let item_data = column![
            text("Genre")
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_surface_variant()),
            title(&state.theme, genre_cloned.name.clone()),
            row![
                text(track_count_text)
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface()),
                artist_count_text
                    .as_ref()
                    .map(|_| text_spacer(state.theme.on_surface_variant(), font::SIZE_LARGE)),
                artist_count_text.map(|a| text(a)
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface())),
                album_count_text
                    .as_ref()
                    .map(|_| text_spacer(state.theme.on_surface_variant(), font::SIZE_LARGE)),
                album_count_text.map(|a| text(a)
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface())),
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

    let album_buttons: Vec<_> = genre
        .albums
        .iter()
        .map(|a| {
            widget::list::album_button::album_button(&state.theme, a.clone())
                .on_press(Message::Navigate(super::TopView::Album(a.clone())))
                .into()
        })
        .collect();

    let has_albums = !album_buttons.is_empty();
    let albums_section = has_albums.then_some(
        column![
            spacer(&state.theme),
            text("Albums")
                .font(font::font_bold())
                .color(state.theme.on_surface())
                .size(font::SIZE_LARGE),
            column(album_buttons).spacing(BUTTON_SPACING),
        ]
        .spacing(SPACING_REGULAR),
    );

    let artist_buttons: Vec<_> = genre
        .artists
        .iter()
        .map(|a| {
            widget::list::artist_button::artist_button(&state.theme, a.clone())
                .on_press(Message::Navigate(super::TopView::Artist(a.clone())))
                .into()
        })
        .collect();

    let has_artists = !artist_buttons.is_empty();
    let artist_section = has_artists.then_some(
        column![
            spacer(&state.theme),
            text("Artists")
                .font(font::font_bold())
                .color(state.theme.on_surface())
                .size(font::SIZE_LARGE),
            column(artist_buttons).spacing(BUTTON_SPACING)
        ]
        .spacing(SPACING_REGULAR),
    );

    let track_buttons = genre.tracks.iter().map(|t| {
        widget::list::track_button::track_button(state, t.clone())
            .on_press(Message::Noop)
            .into()
    });

    column![
        row![
            container(unwind_button(&state.theme)).width(Length::Fixed(50.0)),
            column![display, buttons].spacing(SPACING_REGULAR)
        ]
        .spacing(SPACING_REGULAR),
        artist_section,
        albums_section,
        if has_albums || has_artists {
            Some(
                column![
                    spacer(&state.theme),
                    text("Tracks")
                        .font(font::font_bold())
                        .color(state.theme.on_surface())
                        .size(font::SIZE_LARGE),
                ]
                .spacing(SPACING_REGULAR),
            )
        } else {
            None
        },
        column(track_buttons).spacing(BUTTON_SPACING)
    ]
    .width(Length::Fill)
    .spacing(SPACING_REGULAR)
    .into()
}
