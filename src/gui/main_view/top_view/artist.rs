use std::sync::Arc;

use chilen_backend::music_lib::Artist;
use iced::{Alignment, Element, Length};
use iced_m3::theme::ColorScheme;
use iced_widget::{column, responsive, row, text};

use crate::gui::{
    Chilen, SPACING_REGULAR, SPACING_SMALLER, font, icons,
    main_view::top_view::{
        MAX_COVER_SIZE, MIN_COVER_SIZE, Message, horizontal_buttons, spacer, title,
    },
    widget::{
        cover_image::cover_image,
        list::{BUTTON_SPACING, album_button, track_button},
        text_spacer::text_spacer,
    },
};

pub(super) fn view<'a>(state: &'a Chilen, artist: Arc<Artist>) -> Element<'a, Message> {
    let artist_cloned = artist.clone();
    let display = responsive(move |size| {
        let cover_size = (size.width.min(size.height) / 3.0).clamp(MIN_COVER_SIZE, MAX_COVER_SIZE);

        let track_count_text = if artist_cloned.tracks.len() == 1 {
            "1 track".to_string()
        } else {
            format!("{} tracks", artist_cloned.tracks.len())
        };
        let album_count_text = if artist_cloned.albums.len() == 1 {
            "1 album".to_string()
        } else {
            format!("{} albums", artist_cloned.albums.len())
        };

        let cover = cover_image(
            artist_cloned.cover.hires.clone(),
            &icons::ARTIST,
            cover_size / 4.0,
            state.theme.on_surface_variant(),
            state.theme.surface_container(),
            f32::MAX,
        )
        .width(Length::Fixed(cover_size))
        .height(Length::Fixed(cover_size));

        let item_data = column![
            text("Artist")
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_surface_variant()),
            title(&state.theme, artist_cloned.name.to_string()),
            row![
                text(album_count_text)
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface()),
                text_spacer(state.theme.on_surface_variant(), font::SIZE_LARGE),
                text(track_count_text)
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface()),
            ]
            .align_y(Alignment::Center)
            .spacing(SPACING_SMALLER),
        ];

        row![cover, item_data]
            .align_y(Alignment::Center)
            .spacing(SPACING_REGULAR)
            .into()
    })
    .height(Length::Shrink)
    .width(Length::Shrink);

    let buttons = horizontal_buttons(
        &state.theme,
        Message::PlayArtistNoShuffle {
            artist: artist.clone(),
            initial_index: 0,
        },
        Message::ShuffleArtist {
            artist: artist.clone(),
            initial_index: 0,
        },
        Message::Noop,
    );

    let album_buttons: Vec<_> = artist
        .albums
        .iter()
        .map(|a| {
            album_button::album_button(
                state,
                a.clone(),
                vec![album_button::Info::Date],
                Message::PlayAlbumNoShuffle {
                    album: a.clone(),
                    initial_index: 0,
                },
                Message::ShuffleAlbum {
                    album: a.clone(),
                    initial_index: 0,
                },
            )
            .on_press(Message::Navigate(super::TopView::Album(a.clone())))
            .into()
        })
        .collect();

    let artist_cloned = artist.clone();
    let track_buttons = artist.tracks.iter().enumerate().map(|(i, t)| {
        track_button::track_button(
            state,
            t.clone(),
            track_button::Info::Album,
            track_button::Messages {
                play: Message::PlayArtistNoShuffle {
                    artist: artist_cloned.clone(),
                    initial_index: i,
                },
                shuffle: Message::ShuffleArtist {
                    artist: artist_cloned.clone(),
                    initial_index: i,
                },
                add_to_queue: None,
                add_to_playlist: None,
                details: None,
                remove: None,
            },
        )
        .on_press(Message::PlayArtist {
            artist: artist_cloned.clone(),
            initial_index: i,
        })
        .into()
    });

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

    column![
        display,
        buttons,
        albums_section,
        spacer(&state.theme),
        {
            if has_albums {
                Some(
                    text("Tracks")
                        .font(font::font_bold())
                        .color(state.theme.on_surface())
                        .size(font::SIZE_LARGE),
                )
            } else {
                None
            }
        },
        column(track_buttons).spacing(BUTTON_SPACING)
    ]
    .width(Length::Fill)
    .spacing(SPACING_REGULAR)
    .into()
}
