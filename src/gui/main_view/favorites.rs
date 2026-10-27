use std::ops::Not;

use chilen_backend::music_lib::{LibraryItem, is_favorite};
use chilen_widget::virtual_list::VirtualList;
use iced::{Element, Length};
use iced_core::text::IntoFragment;
use iced_m3::{theme::ColorScheme, widget::spacer};
use iced_widget::{center, column, space};

use crate::gui::{
    Chilen, SPACING_LARGE, SPACING_REGULAR,
    font::bold_text,
    icons,
    main_view::{
        Message, NO_FAVORITES_LABEL, NavTab, TOOLBAR_OFFSET,
        top_view::{self, TopView},
    },
    widget::{
        icon_shape::IconShape,
        list::{
            BUTTON_HEIGHT, BUTTON_SPACING,
            album_button::{self, AlbumButton},
            artist_button::ArtistButton,
            genre_button::GenreButton,
            playlist_button::playlist_button,
            track_button::{self, TrackButton},
        },
    },
};

fn section<'a>(
    title: impl IntoFragment<'a>,
    theme: &(impl ColorScheme + ?Sized),
    content: Element<'a, Message>,
    show_spacer: bool,
) -> Element<'a, Message> {
    column![
        show_spacer.then_some(spacer(theme.outline_variant())),
        bold_text(title).color(theme.on_surface()).size(24),
        content,
    ]
    .spacing(SPACING_REGULAR)
    .into()
}

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    if let Some(lib) = &state.library
        && !lib.favorites.is_empty()
    {
        let mut playlists = Vec::new();
        let mut tracks = Vec::new();
        let mut albums = Vec::new();
        let mut artists = Vec::new();
        let mut genres = Vec::new();

        for item in &lib.favorites {
            match item {
                LibraryItem::Playlist(p) => playlists.push(p),
                LibraryItem::Track(t) => tracks.push(t),
                LibraryItem::Album(a) => albums.push(a),
                LibraryItem::Artist(a) => artists.push(a),
                LibraryItem::Genre(g) => genres.push(g),
            }
        }

        let track_offset = playlists.len();
        let album_offset = track_offset + tracks.len();
        let artist_offset = album_offset + albums.len();
        let genre_offset = artist_offset + artists.len();

        let playlists = playlists.is_empty().not().then_some({
            let highlighted_name = state.player_state.as_ref().and_then(|p| {
                if let chilen_backend::playback::QueueSource::Playlist { name } = &p.queue_source {
                    Some(name)
                } else {
                    None
                }
            });

            VirtualList {
                model: playlists.into_iter().enumerate(),
                delegate: Box::new(move |(index, playlist)| {
                    playlist_button(
                        state,
                        playlist,
                        index,
                        highlighted_name
                            .map(|name| name.as_str() == playlist.name.as_ref())
                            .unwrap_or_default(),
                        state.settings.vibrant_widgets,
                    )
                }),
                delegate_height: BUTTON_HEIGHT,
                visibilities: state.main_view.visible.as_deref().unwrap_or(&[]),
                list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
                on_show: Box::new(Message::ButtonPoppedIn),
                on_hide: Box::new(Message::ButtonPoppedOut),
            }
        });

        let tracks = tracks.is_empty().not().then_some({
            // FIX: Won't work that
            let highlighted_index = state.player_state.as_ref().and_then(|p| {
                if p.queue_source == chilen_backend::playback::QueueSource::AllTracks {
                    p.real_track_index(p.position)
                } else {
                    None
                }
            });

            VirtualList {
                model: tracks.into_iter().enumerate(),
                delegate: Box::new(move |(index, track)| {
                    let favorite_item: LibraryItem = track.clone().into();
                    let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                    TrackButton {
                        state,
                        track: track.clone(),
                        is_favorite,
                        messages: track_button::Messages {
                            play: Message::PlayTracksNoShuffle {
                                initial_position: index,
                            },
                            press: Message::PlayTracks {
                                initial_position: index,
                            },
                            toggle_favorite: Message::ToggleFavorite(favorite_item),
                            shuffle: Some(Message::ShuffleTracks {
                                initial_position: Some(index),
                            }),
                            add_to_queue: Some(Message::AddTrackToQueue(track.clone())),
                            add_to_playlist: Message::AddTracksToPlaylist(vec![track.hash_self()]),
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
                        vibrant: state.settings.vibrant_widgets,
                    }
                    .into()
                }),
                delegate_height: BUTTON_HEIGHT,
                visibilities: state
                    .main_view
                    .visible
                    .as_deref()
                    .unwrap_or(&[])
                    .get(track_offset..)
                    .unwrap_or(&[]),
                list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
                on_show: Box::new(move |index| Message::ButtonPoppedIn(track_offset + index)),
                on_hide: Box::new(move |index| Message::ButtonPoppedOut(track_offset + index)),
            }
        });

        let albums = albums.is_empty().not().then_some({
            let highlighted_album_title = state.player_state.as_ref().and_then(|p| {
                if let chilen_backend::playback::QueueSource::Album { title } = &p.queue_source {
                    Some(title)
                } else {
                    None
                }
            });

            VirtualList {
                model: albums,
                delegate: Box::new(move |album| {
                    let highlighted = highlighted_album_title
                        .map(|t| t == album.title.as_ref())
                        .unwrap_or_default();
                    let favorite_item: LibraryItem = album.clone().into();
                    let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                    AlbumButton {
                        state,
                        album: album.clone(),
                        info: vec![
                            album_button::Info::TrackCount,
                            album_button::Info::ArtistCount,
                        ],
                        play: Message::PlayAlbum(album.clone()),
                        press: Message::TopView(top_view::Message::Navigate(TopView::Album(
                            album.clone(),
                        ))),
                        shuffle: Message::ShuffleAlbum(album.clone()),
                        add_to_queue: Message::AddAlbumToQueue(album.clone()),
                        toggle_favorite: Message::ToggleFavorite(favorite_item),
                        is_favorite,
                        highlighted,
                        vibrant: state.settings.vibrant_widgets,
                    }
                    .into()
                }),
                delegate_height: BUTTON_HEIGHT,
                visibilities: state
                    .main_view
                    .visible
                    .as_deref()
                    .unwrap_or(&[])
                    .get(album_offset..)
                    .unwrap_or(&[]),
                list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
                on_show: Box::new(move |index| Message::ButtonPoppedIn(album_offset + index)),
                on_hide: Box::new(move |index| Message::ButtonPoppedOut(album_offset + index)),
            }
        });

        let artists = artists.is_empty().not().then_some({
            let highlighted_artist_name = state.player_state.as_ref().and_then(|p| {
                if let chilen_backend::playback::QueueSource::Artist { name } = &p.queue_source {
                    Some(name)
                } else {
                    None
                }
            });

            VirtualList {
                model: artists,
                delegate: Box::new(move |artist| {
                    let highlighted = highlighted_artist_name
                        .map(|name| name == artist.name.as_ref())
                        .unwrap_or_default();
                    let favorite_item: LibraryItem = artist.clone().into();
                    let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                    ArtistButton {
                        theme: &state.theme,
                        artist: artist.clone(),
                        play: Message::PlayArtist(artist.clone()),
                        press: Message::TopView(top_view::Message::Navigate(TopView::Artist(
                            artist.clone(),
                        ))),
                        shuffle: Message::ShuffleArtist(artist.clone()),
                        add_to_queue: Message::AddArtistToQueue(artist.clone()),
                        toggle_favorite: Message::ToggleFavorite(favorite_item),
                        is_favorite,
                        highlighted,
                        vibrant: state.settings.vibrant_widgets,
                    }
                    .into()
                }),
                delegate_height: BUTTON_HEIGHT,
                visibilities: state
                    .main_view
                    .visible
                    .as_deref()
                    .unwrap_or(&[])
                    .get(artist_offset..)
                    .unwrap_or(&[]),
                list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
                on_show: Box::new(move |index| Message::ButtonPoppedIn(artist_offset + index)),
                on_hide: Box::new(move |index| Message::ButtonPoppedOut(artist_offset + index)),
            }
        });

        let genres = genres.is_empty().not().then_some({
            let highlighted_genre_name = state.player_state.as_ref().and_then(|p| {
                if let chilen_backend::playback::QueueSource::Genre { name } = &p.queue_source {
                    Some(name)
                } else {
                    None
                }
            });

            VirtualList {
                model: genres,
                delegate: Box::new(move |genre| {
                    let highlighted = highlighted_genre_name
                        .map(|name| name == genre.name.as_ref())
                        .unwrap_or_default();
                    let favorite_item: LibraryItem = genre.clone().into();
                    let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                    GenreButton {
                        theme: &state.theme,
                        genre: genre.clone(),
                        play: Message::PlayGenre(genre.clone()),
                        press: Message::TopView(top_view::Message::Navigate(TopView::Genre(
                            genre.clone(),
                        ))),
                        shuffle: Message::ShuffleGenre(genre.clone()),
                        add_to_queue: Message::AddGenreToQueue(genre.clone()),
                        toggle_favorite: Message::ToggleFavorite(favorite_item),
                        is_favorite,
                        highlighted,
                        vibrant: state.settings.vibrant_widgets,
                    }
                    .into()
                }),
                delegate_height: BUTTON_HEIGHT,
                visibilities: state
                    .main_view
                    .visible
                    .as_deref()
                    .unwrap_or(&[])
                    .get(genre_offset..)
                    .unwrap_or(&[]),
                list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
                on_show: Box::new(move |index| Message::ButtonPoppedIn(genre_offset + index)),
                on_hide: Box::new(move |index| Message::ButtonPoppedOut(genre_offset + index)),
            }
        });

        let track_spacer = playlists.is_some();
        let album_spacer = track_spacer || tracks.is_some();
        let artist_spacer = album_spacer || albums.is_some();
        let genre_spacer = artist_spacer || artists.is_some();
        let content = column![
            playlists.map(|c| section("Playlists", &state.theme, c.into(), false)),
            tracks.map(|c| section("Tracks", &state.theme, c.into(), track_spacer)),
            albums.map(|c| section("Albums", &state.theme, c.into(), album_spacer)),
            artists.map(|c| section("Artists", &state.theme, c.into(), artist_spacer)),
            genres.map(|c| section("Genres", &state.theme, c.into(), genre_spacer))
        ]
        .spacing(SPACING_LARGE);

        iced_widget::scrollable(column![content, space().height(*TOOLBAR_OFFSET)])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
            .into()
    } else {
        center(
            IconShape::new(
                NavTab::Favorites.shape(),
                state.theme.surface_container_high(),
                icons::MUSIC_NOTE.into_fragment(),
                state.theme.on_surface_variant(),
            )
            .label(NO_FAVORITES_LABEL, state.theme.on_surface_variant()),
        )
        .into()
    }
}
