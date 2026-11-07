use std::sync::Arc;

use chilen_backend::music_lib::{Artist, LibraryItem, is_favorite};
use iced::{Alignment, Element, Length};
use iced_m3::{theme::ColorScheme, widget::spacer};
use iced_widget::{column, responsive, row, text};

use crate::gui::{
    Chilen, SPACING_REGULAR, SPACING_SMALLER, dialog,
    event_loop::{Message, library, playback},
    font::{self, bold_text},
    icons,
    main_view::{
        self,
        top_view::{self, MAX_COVER_SIZE, MIN_COVER_SIZE, horizontal_buttons, title},
    },
    widget::{
        cover_image::CoverImage,
        list::{
            BUTTON_SPACING,
            album_button::{self, AlbumButton},
            artist_button,
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
            image_path: artist_cloned.cover.cover().ok(),
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

    let buttons = {
        let artist = artist.clone();
        let play_no_shuffle = Message::Playback(playback::Message::PlayArtistNoShuffle {
            artist: artist.clone(),
            initial_position: 0,
        });
        let shuffle = Message::Playback(playback::Message::ShuffleArtist {
            artist: artist.clone(),
            initial_position: None,
        });

        horizontal_buttons(
            state,
            Some(play_no_shuffle.clone()),
            Some(shuffle.clone()),
            move || {
                let favorite_item = artist.clone().into();
                let is_favorite = is_favorite(&favorite_item).unwrap_or_default();

                artist_button::menu(
                    state,
                    play_no_shuffle.clone(),
                    shuffle.clone(),
                    Message::Playback(playback::Message::AddArtistToQueue(artist.clone())),
                    Message::Library(library::Message::ToggleFavorite(favorite_item)),
                    is_favorite,
                )
                .into()
            },
        )
    };

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
        .enumerate()
        .map(|(index, album)| {
            let favorite_item: LibraryItem = album.clone().into();
            let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
            AlbumButton {
                state,
                album: album.clone(),
                info: vec![album_button::Info::Date],
                play: Message::Playback(playback::Message::PlayAlbumNoShuffle {
                    album: album.clone(),
                    initial_position: 0,
                }),
                press: Message::MainView(main_view::Message::TopView(top_view::Message::Navigate(
                    super::TopView::Album(album.clone()),
                ))),
                shuffle: Message::Playback(playback::Message::ShuffleAlbum {
                    album: album.clone(),
                    initial_position: None,
                }),
                add_to_queue: Message::Playback(playback::Message::AddAlbumToQueue(album.clone())),
                toggle_favorite: Message::Library(library::Message::ToggleFavorite(favorite_item)),
                toggle_menu: Message::MainView(main_view::Message::TopView(
                    top_view::Message::ToggleMenu(index),
                )),
                menu_opened: state.main_view.button_menu == Some(index),
                is_favorite,
                highlighted: highlighted_album_title
                    .map(|t| t == album.title.as_ref())
                    .unwrap_or_default(),
            }
            .into()
        })
        .collect();

    let highlighted_index = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Artist { name: a } = &p.queue_source
            && a == artist.name.as_ref()
        {
            p.real_track_index(p.position)
        } else {
            None
        }
    });
    let track_buttons = artist.tracks.iter().enumerate().map(|(index, track)| {
        let favorite_item: LibraryItem = track.clone().into();
        let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
        TrackButton {
            state,
            track: track.clone(),
            is_favorite,
            messages: track_button::Messages {
                play: Message::Playback(playback::Message::PlayArtistNoShuffle {
                    artist: artist.clone(),
                    initial_position: index,
                }),
                press: Message::Playback(playback::Message::PlayArtist {
                    artist: artist.clone(),
                    initial_position: index,
                }),
                toggle_favorite: Message::Library(library::Message::ToggleFavorite(favorite_item)),
                shuffle: Some(Message::Playback(playback::Message::ShuffleArtist {
                    artist: artist.clone(),
                    initial_position: None,
                })),
                add_to_queue: Some(Message::Playback(playback::Message::AddTrackToQueue(
                    track.clone(),
                ))),
                add_to_playlist: Message::Dialog(dialog::Message::AddTracksToPlaylist(vec![
                    track.clone(),
                ])),
                details: None,
                remove: None,
                toggle_menu: Message::MainView(main_view::Message::TopView(
                    top_view::Message::ToggleMenu(index),
                )),
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
            vibrant: state.settings.vibrant_widgets,
            menu_opened: state.main_view.button_menu == Some(index),
        }
        .into()
    });

    let has_albums = !album_buttons.is_empty();
    let albums_section = has_albums.then_some(
        column![
            spacer(state.theme.outline_variant()),
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
        spacer(state.theme.outline_variant()),
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
