use std::sync::Arc;

use chilen_backend::music_lib::Genre;
use iced::{Alignment, Element, Length};
use iced_m3::theme::ColorScheme;
use iced_widget::{column, responsive, row, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALLER, font, icons,
    main_view::top_view::{
        MAX_COVER_SIZE, MIN_COVER_SIZE, Message, horizontal_buttons, spacer, title,
    },
    widget::{
        cover_image::cover_image,
        list::{
            BUTTON_SPACING,
            album_button::{self, AlbumButton},
            artist_button::ArtistButton,
            track_button::{self, TrackButton},
        },
        text_spacer::text_spacer,
    },
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
            1.0,
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
            .spacing(SPACING_SMALLER)
            .wrap(),
        ]
        .align_x(Alignment::Start);

        row![cover, item_data]
            .align_y(Alignment::Center)
            .spacing(SPACING_REGULAR)
            .wrap()
            .into()
    })
    .height(Length::Shrink)
    .width(Length::Shrink);

    let buttons = horizontal_buttons(
        &state.theme,
        Message::PlayGenreNoShuffle {
            genre: genre.clone(),
            initial_position: None,
        },
        Message::ShuffleGenre {
            genre: genre.clone(),
            initial_position: None,
        },
        Message::Noop,
    );

    let highlighted_album_title = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Album { title } = &p.queue_source {
            Some(title)
        } else {
            None
        }
    });
    let album_buttons: Vec<_> = genre
        .albums
        .iter()
        .map(|album| {
            AlbumButton {
                state,
                album: album.clone(),
                info: vec![
                    album_button::Info::TrackCount,
                    album_button::Info::ArtistCount,
                ],
                play: Message::PlayAlbumNoShuffle {
                    album: album.clone(),
                    initial_position: None,
                },
                press: Message::Navigate(super::TopView::Album(album.clone())),
                shuffle: Message::ShuffleAlbum {
                    album: album.clone(),
                    initial_position: None,
                },
                add_to_queue: Message::AddAlbumToQueue(album.clone()),
                highlighted: highlighted_album_title
                    .map(|t| *t == album.title)
                    .unwrap_or_default(),
            }
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

    let highlighted_artist_name = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Artist { name } = &p.queue_source {
            Some(name)
        } else {
            None
        }
    });
    let artist_buttons: Vec<_> = genre
        .artists
        .iter()
        .map(|artist| {
            ArtistButton {
                theme: &state.theme,
                artist: artist.clone(),
                play: Message::PlayArtistNoShuffle {
                    artist: artist.clone(),
                    initial_position: None,
                },
                press: Message::Navigate(super::TopView::Artist(artist.clone())),
                shuffle: Message::ShuffleArtist {
                    artist: artist.clone(),
                    initial_position: None,
                },
                add_to_queue: Message::AddArtistToQueue(artist.clone()),
                highlighted: highlighted_artist_name
                    .map(|name| *name == artist.name)
                    .unwrap_or_default(),
            }
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

    let highlighted_index = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Genre { name: g } = &p.queue_source
            && g == &genre.name
        {
            p.real_track_index(p.position)
        } else {
            None
        }
    });
    let track_buttons = genre.tracks.iter().enumerate().map(|(index, track)| {
        TrackButton {
            state,
            track: track.clone(),
            messages: track_button::Messages {
                play: Message::PlayGenreNoShuffle {
                    genre: genre.clone(),
                    initial_position: Some(index),
                },
                press: Message::PlayGenre {
                    genre: genre.clone(),
                    initial_position: Some(index),
                },
                shuffle: Some(Message::ShuffleGenre {
                    genre: genre.clone(),
                    initial_position: Some(index),
                }),
                add_to_queue: Some(Message::AddTrackToQueue(track.clone())),
                add_to_playlist: None,
                details: None,
                remove: None,
            },
            info: track_button::Info::Artist,
            status: if highlighted_index
                .map(|highlighted| highlighted == index)
                .unwrap_or_default()
            {
                track_button::Status::Playing
            } else {
                track_button::Status::Idle
            },
        }
        .into()
    });

    column![
        display,
        buttons,
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
