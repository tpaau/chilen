use std::sync::Arc;

use chilen_backend::music_lib::Artist;
use iced::{Alignment, Element, Length};
use iced_m3::{theme::ColorScheme, widget::spacer};
use iced_widget::{column, responsive, row, text};

use crate::gui::{
    Chilen, SPACING_REGULAR, SPACING_SMALLER,
    font::{self, bold_text},
    icons,
    main_view::top_view::{MAX_COVER_SIZE, MIN_COVER_SIZE, Message, horizontal_buttons, title},
    widget::{
        cover_image::CoverImage,
        list::{
            BUTTON_SPACING,
            album_button::{self, AlbumButton},
            track_button::{self, TrackButton},
        },
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

        let cover = CoverImage {
            image_path: artist_cloned.cover.hires.clone(),
            icon: *icons::ARTIST,
            icon_size: cover_size / 4.0,
            icon_color: state.theme.on_surface_variant(),
            container_color: state.theme.surface_container(),
            radius: f32::MAX.into(),
            opacity: 1.0,
            width: cover_size.into(),
            height: cover_size.into(),
        };

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
            .spacing(SPACING_SMALLER)
            .wrap(),
        ];

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
        Some(Message::PlayArtistNoShuffle {
            artist: artist.clone(),
            initial_position: None,
        }),
        Some(Message::ShuffleArtist {
            artist: artist.clone(),
            initial_position: None,
        }),
        Message::Noop,
    );

    let highlighted_album_title = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Album { title } = &p.queue_source {
            Some(title)
        } else {
            None
        }
    });
    let album_buttons: Vec<_> = artist
        .albums
        .iter()
        .map(|album| {
            AlbumButton {
                state,
                album: album.clone(),
                info: vec![album_button::Info::Date],
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

    let highlighted_index = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Artist { name: a } = &p.queue_source
            && a == &artist.name
        {
            p.real_track_index(p.position)
        } else {
            None
        }
    });
    let track_buttons = artist.tracks.iter().enumerate().map(|(index, track)| {
        TrackButton {
            state,
            track: track.clone(),
            messages: track_button::Messages {
                play: Message::PlayArtistNoShuffle {
                    artist: artist.clone(),
                    initial_position: Some(index),
                },
                press: Message::PlayArtist {
                    artist: artist.clone(),
                    initial_position: Some(index),
                },
                shuffle: Some(Message::ShuffleArtist {
                    artist: artist.clone(),
                    initial_position: Some(index),
                }),
                add_to_queue: Some(Message::AddTrackToQueue(track.clone())),
                add_to_playlist: Message::AddTrackToPlaylist(track.clone()),
                details: None,
                remove: None,
            },
            info: track_button::Info::Album,
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

    let has_albums = !album_buttons.is_empty();
    let albums_section = has_albums.then_some(
        column![
            spacer(&state.theme),
            bold_text("Albums")
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
                    bold_text("Tracks")
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
