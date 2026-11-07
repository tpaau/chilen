use std::ops::Not;

use chilen_backend::music_lib::{
    LibraryItem, is_favorite,
    sorting::{sort_albums, sort_artists, sort_genres, sort_playlists, sort_tracks},
};
use chilen_widget::virtual_list::VirtualList;
use iced::{Element, Length};
use iced_core::text::IntoFragment;
use iced_m3::{theme::ColorScheme, widget::spacer};
use iced_widget::{center, column, space};

use crate::gui::{
    Chilen, SPACING_LARGE, SPACING_REGULAR, dialog,
    event_loop::{Message, library, playback},
    font::bold_text,
    icons,
    main_view::{
        self, NO_FAVORITES_LABEL, NavTab, TOOLBAR_OFFSET,
        top_view::{self, TopView},
    },
    widget::{
        icon_shape::IconShape,
        list::{
            BUTTON_HEIGHT, BUTTON_SPACING,
            album_button::{self, AlbumButton},
            artist_button::ArtistButton,
            genre_button::GenreButton,
            playlist_button::{self, PlaylistButton},
            track_button::{self, TrackButton},
        },
    },
};

pub const ALL_FAVORITE_TRACKS_QUEUE_NAME: &str = "Favorite tracks";

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

        sort_playlists(&mut playlists);
        sort_tracks(&mut tracks);
        sort_albums(&mut albums);
        sort_artists(&mut artists);
        sort_genres(&mut genres);

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
                    let favorite_item: LibraryItem = playlist.clone().into();
                    let is_favorite = is_favorite(&favorite_item).unwrap_or_default();

                    PlaylistButton {
                        state,
                        playlist,
                        press: Message::MainView(main_view::Message::TopView(
                            top_view::Message::Navigate(TopView::Playlist(playlist.clone())),
                        )),
                        toggle_menu: Message::MainView(main_view::Message::ToggleMenu(index)),
                        messages: playlist_button::Messages {
                            play: playlist
                                .tracks
                                .is_empty()
                                .not()
                                .then_some(Message::Playback(playback::Message::PlayPlaylist {
                                    playlist: playlist.clone(),
                                    initial_position: index,
                                })),
                            shuffle: playlist
                                .tracks
                                .is_empty()
                                .not()
                                .then_some(Message::Playback(playback::Message::ShufflePlaylist {
                                    playlist: playlist.clone(),
                                    initial_position: None,
                                })),
                            add_to_queue: playlist.tracks.is_empty().not().then_some(
                                Message::Playback(playback::Message::AddPlaylistToQueue(
                                    playlist.clone(),
                                )),
                            ),
                            export: playlist.tracks.is_empty().not().then_some(Message::Library(
                                library::Message::ExportPlaylist(playlist.clone()),
                            )),
                            toggle_favorite: Message::Library(library::Message::ToggleFavorite(
                                favorite_item,
                            )),
                            rename: Message::Dialog(dialog::Message::RenamePlaylist {
                                playlist: playlist.clone(),
                                name: playlist.name.clone().into_string(),
                            }),
                            delete: Message::Dialog(dialog::Message::DeletePlaylist(
                                playlist.clone(),
                            )),
                        },
                        highlighted: highlighted_name
                            .map(|name| name.as_str() == playlist.name.as_ref())
                            .unwrap_or_default(),
                        is_favorite,
                        menu_opened: state.main_view.button_menu == Some(index),
                    }
                    .into()
                }),
                delegate_height: BUTTON_HEIGHT,
                visibilities: state.main_view.visible.as_deref().unwrap_or(&[]),
                list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
                on_show: Box::new(|i| Message::MainView(main_view::Message::ButtonPoppedIn(i))),
                on_hide: Box::new(|i| Message::MainView(main_view::Message::ButtonPoppedOut(i))),
            }
        });

        let tracks = tracks.is_empty().not().then_some({
            // FIX: Won't work that
            let highlighted_index = state.player_state.as_ref().and_then(|p| {
                if let chilen_backend::playback::QueueSource::Custom { label } = &p.queue_source
                    && label == ALL_FAVORITE_TRACKS_QUEUE_NAME
                {
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
                            play: Message::Playback(
                                playback::Message::PlayFavoriteTracksNoShuffle {
                                    initial_position: index,
                                },
                            ),
                            press: Message::Playback(playback::Message::PlayFavoriteTracks {
                                initial_position: index,
                            }),
                            toggle_favorite: Message::Library(library::Message::ToggleFavorite(
                                favorite_item,
                            )),
                            shuffle: Some(Message::Playback(
                                playback::Message::ShuffleFavoriteTracks {
                                    initial_position: Some(index),
                                },
                            )),
                            add_to_queue: Some(Message::Playback(
                                playback::Message::AddTrackToQueue(track.clone()),
                            )),
                            add_to_playlist: Message::Dialog(dialog::Message::AddTracksToPlaylist(
                                vec![track.clone()],
                            )),
                            details: None,
                            remove: None,
                            toggle_menu: Message::MainView(main_view::Message::ToggleMenu(
                                index + track_offset,
                            )),
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
                        menu_opened: state.main_view.button_menu == Some(index + track_offset),
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
                on_show: Box::new(move |index| {
                    Message::MainView(main_view::Message::ButtonPoppedIn(track_offset + index))
                }),
                on_hide: Box::new(move |index| {
                    Message::MainView(main_view::Message::ButtonPoppedOut(track_offset + index))
                }),
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
                model: albums.into_iter().enumerate(),
                delegate: Box::new(move |(index, album)| {
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
                        play: Message::Playback(playback::Message::PlayAlbum {
                            album: album.clone(),
                            initial_position: 0,
                        }),
                        press: Message::MainView(main_view::Message::TopView(
                            top_view::Message::Navigate(TopView::Album(album.clone())),
                        )),
                        shuffle: Message::Playback(playback::Message::ShuffleAlbum {
                            album: album.clone(),
                            initial_position: None,
                        }),
                        add_to_queue: Message::Playback(playback::Message::AddAlbumToQueue(
                            album.clone(),
                        )),
                        toggle_favorite: Message::Library(library::Message::ToggleFavorite(
                            favorite_item,
                        )),
                        toggle_menu: Message::MainView(main_view::Message::ToggleMenu(
                            index + album_offset,
                        )),
                        menu_opened: state.main_view.button_menu == Some(index + album_offset),
                        is_favorite,
                        highlighted,
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
                on_show: Box::new(move |index| {
                    Message::MainView(main_view::Message::ButtonPoppedIn(album_offset + index))
                }),
                on_hide: Box::new(move |index| {
                    Message::MainView(main_view::Message::ButtonPoppedOut(album_offset + index))
                }),
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
                model: artists.into_iter().enumerate(),
                delegate: Box::new(move |(index, artist)| {
                    let highlighted = highlighted_artist_name
                        .map(|name| name == artist.name.as_ref())
                        .unwrap_or_default();
                    let favorite_item: LibraryItem = artist.clone().into();
                    let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                    ArtistButton {
                        state,
                        artist: artist.clone(),
                        play: Message::Playback(playback::Message::PlayArtist {
                            artist: artist.clone(),
                            initial_position: 0,
                        }),
                        press: Message::MainView(main_view::Message::TopView(
                            top_view::Message::Navigate(TopView::Artist(artist.clone())),
                        )),
                        shuffle: Message::Playback(playback::Message::ShuffleArtist {
                            artist: artist.clone(),
                            initial_position: None,
                        }),
                        add_to_queue: Message::Playback(playback::Message::AddArtistToQueue(
                            artist.clone(),
                        )),
                        toggle_favorite: Message::Library(library::Message::ToggleFavorite(
                            favorite_item,
                        )),
                        toggle_menu: Message::MainView(main_view::Message::ToggleMenu(
                            index + artist_offset,
                        )),
                        is_favorite,
                        highlighted,
                        vibrant: state.settings.vibrant_widgets,
                        menu_opened: state.main_view.button_menu == Some(index + artist_offset),
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
                on_show: Box::new(move |index| {
                    Message::MainView(main_view::Message::ButtonPoppedIn(artist_offset + index))
                }),
                on_hide: Box::new(move |index| {
                    Message::MainView(main_view::Message::ButtonPoppedOut(artist_offset + index))
                }),
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
                model: genres.into_iter().enumerate(),
                delegate: Box::new(move |(index, genre)| {
                    let highlighted = highlighted_genre_name
                        .map(|name| name == genre.name.as_ref())
                        .unwrap_or_default();
                    let favorite_item: LibraryItem = genre.clone().into();
                    let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                    GenreButton {
                        state,
                        genre: genre.clone(),
                        play: Message::Playback(playback::Message::PlayGenre {
                            genre: genre.clone(),
                            initial_position: 0,
                        }),
                        press: Message::MainView(main_view::Message::TopView(
                            top_view::Message::Navigate(TopView::Genre(genre.clone())),
                        )),
                        shuffle: Message::Playback(playback::Message::ShuffleGenre {
                            genre: genre.clone(),
                            initial_position: None,
                        }),
                        add_to_queue: Message::Playback(playback::Message::AddGenreToQueue(
                            genre.clone(),
                        )),
                        toggle_favorite: Message::Library(library::Message::ToggleFavorite(
                            favorite_item,
                        )),
                        toggle_menu: Message::MainView(main_view::Message::ToggleMenu(
                            index + genre_offset,
                        )),
                        is_favorite,
                        highlighted,
                        menu_opened: state.main_view.button_menu == Some(index + genre_offset),
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
                on_show: Box::new(move |index| {
                    Message::MainView(main_view::Message::ButtonPoppedIn(genre_offset + index))
                }),
                on_hide: Box::new(move |index| {
                    Message::MainView(main_view::Message::ButtonPoppedOut(genre_offset + index))
                }),
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
