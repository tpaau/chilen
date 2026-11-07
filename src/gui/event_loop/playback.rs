use std::sync::Arc;

use chilen_backend::{
    music_lib::{
        Album, Artist, Genre, LibraryItem, MusicLibrary, Playlist, Track, sorting::sort_tracks,
    },
    playback::{Queue, ShuffleState},
};
use iced::Task;
use log::error;

use crate::gui::{Chilen, event_loop, persistent_state};

#[derive(Clone)]
pub enum Message {
    /// Plays all tracks in the music library with the current shuffle state.
    PlayAllTracks { initial_position: usize },
    /// Enables shuffling and plays all tracks in the music library.
    ShuffleAllTracks {
        /// The initial position in the queue. Will be random if let at `None`.
        initial_position: Option<usize>,
    },
    /// Disables shuffling and plays all tracks in the music library.
    PlayAllTracksNoShuffle { initial_position: usize },
    /// Plays favorite tracks in the music library with the current shuffle state.
    PlayFavoriteTracks { initial_position: usize },
    /// Enables shuffling and plays favorite tracks in the music library.
    ShuffleFavoriteTracks {
        /// The initial position in the queue. Will be random if let at `None`.
        initial_position: Option<usize>,
    },
    /// Disables shuffling and plays favorite tracks in the music library.
    PlayFavoriteTracksNoShuffle { initial_position: usize },
    /// Plays tracks in the given album with the current shuffle state.
    PlayAlbum {
        album: Arc<Album>,
        initial_position: usize,
    },
    /// Enables shuffling and plays tracks in the given album.
    ShuffleAlbum {
        album: Arc<Album>,
        /// The initial position in the queue. Will be random if let at `None`.
        initial_position: Option<usize>,
    },
    /// Disables shuffling and plays tracks in the given album.
    PlayAlbumNoShuffle {
        album: Arc<Album>,
        initial_position: usize,
    },
    /// Plays tracks of the given artist with the current shuffle state.
    PlayArtist {
        artist: Arc<Artist>,
        initial_position: usize,
    },
    /// Enables shuffling and plays tracks of the given artist.
    ShuffleArtist {
        artist: Arc<Artist>,
        /// The initial position in the queue. Will be random if let at `None`.
        initial_position: Option<usize>,
    },
    /// Disables shuffling and plays tracks of the given artist.
    PlayArtistNoShuffle {
        artist: Arc<Artist>,
        initial_position: usize,
    },
    /// Plays tracks with the given genre with the current shuffle state.
    PlayGenre {
        genre: Arc<Genre>,
        initial_position: usize,
    },
    /// Enables shuffling and plays tracks with the given genre.
    ShuffleGenre {
        genre: Arc<Genre>,
        /// The initial position in the queue. Will be random if let at `None`.
        initial_position: Option<usize>,
    },
    /// Disables shuffling and plays tracks with the given genre.
    PlayGenreNoShuffle {
        genre: Arc<Genre>,
        initial_position: usize,
    },
    /// Plays tracks in the given playlist with the current shuffle state.
    PlayPlaylist {
        playlist: Arc<Playlist>,
        initial_position: usize,
    },
    /// Enables shuffling and plays tracks in the given playlist.
    ShufflePlaylist {
        playlist: Arc<Playlist>,
        /// The initial position in the queue. Will be random if let at `None`.
        initial_position: Option<usize>,
    },
    /// Disables shuffling and plays tracks in the given playlist.
    PlayPlaylistNoShuffle {
        playlist: Arc<Playlist>,
        initial_position: usize,
    },
    /// Sets the queue to a single track.
    PlayTrack(Arc<Track>),
    /// Appends a track to the queue.
    AddTrackToQueue(Arc<Track>),
    /// Appends all tracks in the given album to the queue.
    AddAlbumToQueue(Arc<Album>),
    /// Appends all tracks of the given artist to the queue.
    AddArtistToQueue(Arc<Artist>),
    /// Appends all tracks with the given genre to the queue.
    AddGenreToQueue(Arc<Genre>),
    /// Appends all tracks in the given playlists to the queue.
    AddPlaylistToQueue(Arc<Playlist>),
}

pub const ALL_FAVORITE_TRACKS_QUEUE_NAME: &str = "Favorite tracks";
const UNLOADED_ERROR_MESSAGE: &str = "Cannot play the track, the library is not loaded";

fn get_favorite_tracks(lib: &MusicLibrary) -> Vec<Arc<Track>> {
    let mut tracks: Vec<_> = lib
        .favorites
        .iter()
        .filter_map(|f| {
            if let LibraryItem::Track(t) = f {
                return Some(t.clone());
            }
            None
        })
        .collect();
    sort_tracks(&mut tracks);
    tracks
}

pub fn update(state: &mut Chilen, message: Message) -> Task<event_loop::Message> {
    match message {
        Message::PlayAllTracks { initial_position } => match &state.library {
            Some(lib) => {
                let _ = chilen_backend::playback::play_new_queue(
                    Queue::AllTracks(lib.tracks.clone()),
                    Some(initial_position),
                );
            }
            None => error!("{UNLOADED_ERROR_MESSAGE}"),
        },
        Message::ShuffleAllTracks { initial_position } => match &state.library {
            Some(lib) => {
                let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
                let _ = chilen_backend::playback::play_new_queue(
                    Queue::AllTracks(lib.tracks.clone()),
                    initial_position,
                );
            }
            None => error!("{UNLOADED_ERROR_MESSAGE}"),
        },
        Message::PlayAllTracksNoShuffle { initial_position } => match &state.library {
            Some(lib) => {
                let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
                let _ = chilen_backend::playback::play_new_queue(
                    Queue::AllTracks(lib.tracks.clone()),
                    Some(initial_position),
                );
            }
            None => error!("{UNLOADED_ERROR_MESSAGE}"),
        },
        Message::PlayFavoriteTracks { initial_position } => match &state.library {
            Some(lib) => {
                let _ = chilen_backend::playback::play_new_queue(
                    Queue::Custom {
                        label: ALL_FAVORITE_TRACKS_QUEUE_NAME.to_string(),
                        tracks: get_favorite_tracks(lib),
                    },
                    Some(initial_position),
                );
            }
            None => error!("{UNLOADED_ERROR_MESSAGE}"),
        },
        Message::ShuffleFavoriteTracks { initial_position } => match &state.library {
            Some(lib) => {
                let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
                let _ = chilen_backend::playback::play_new_queue(
                    Queue::Custom {
                        label: ALL_FAVORITE_TRACKS_QUEUE_NAME.to_string(),
                        tracks: get_favorite_tracks(lib),
                    },
                    initial_position,
                );
            }
            None => error!("{UNLOADED_ERROR_MESSAGE}"),
        },
        Message::PlayFavoriteTracksNoShuffle { initial_position } => match &state.library {
            Some(lib) => {
                let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
                let _ = chilen_backend::playback::play_new_queue(
                    Queue::Custom {
                        label: ALL_FAVORITE_TRACKS_QUEUE_NAME.to_string(),
                        tracks: get_favorite_tracks(lib),
                    },
                    Some(initial_position),
                );
            }
            None => error!("{UNLOADED_ERROR_MESSAGE}"),
        },
        Message::PlayAlbum {
            album,
            initial_position,
        } => {
            let _ = chilen_backend::playback::play_new_queue(album.clone(), Some(initial_position));
            persistent_state::push_recent(state, album);
        }
        Message::ShuffleAlbum {
            album,
            initial_position,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
            let _ = chilen_backend::playback::play_new_queue(album.clone(), initial_position);
            persistent_state::push_recent(state, album);
        }
        Message::PlayAlbumNoShuffle {
            album,
            initial_position,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
            let _ = chilen_backend::playback::play_new_queue(album.clone(), Some(initial_position));
            persistent_state::push_recent(state, album);
        }
        Message::PlayArtist {
            artist,
            initial_position,
        } => {
            let _ =
                chilen_backend::playback::play_new_queue(artist.clone(), Some(initial_position));
            persistent_state::push_recent(state, artist);
        }
        Message::ShuffleArtist {
            artist,
            initial_position,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
            let _ = chilen_backend::playback::play_new_queue(artist.clone(), initial_position);
            persistent_state::push_recent(state, artist);
        }
        Message::PlayArtistNoShuffle {
            artist,
            initial_position,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
            let _ =
                chilen_backend::playback::play_new_queue(artist.clone(), Some(initial_position));
            persistent_state::push_recent(state, artist);
        }
        Message::PlayGenre {
            genre,
            initial_position,
        } => {
            let _ = chilen_backend::playback::play_new_queue(genre.clone(), Some(initial_position));
            persistent_state::push_recent(state, genre);
        }
        Message::ShuffleGenre {
            genre,
            initial_position,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
            let _ = chilen_backend::playback::play_new_queue(genre.clone(), initial_position);
            persistent_state::push_recent(state, genre);
        }
        Message::PlayGenreNoShuffle {
            genre,
            initial_position,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
            let _ = chilen_backend::playback::play_new_queue(genre.clone(), Some(initial_position));
            persistent_state::push_recent(state, genre);
        }
        Message::PlayPlaylist {
            playlist,
            initial_position,
        } => {
            let _ =
                chilen_backend::playback::play_new_queue(playlist.clone(), Some(initial_position));
            persistent_state::push_recent(state, playlist);
        }
        Message::ShufflePlaylist {
            playlist,
            initial_position,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
            let _ = chilen_backend::playback::play_new_queue(playlist.clone(), initial_position);
            persistent_state::push_recent(state, playlist);
        }
        Message::PlayPlaylistNoShuffle {
            playlist,
            initial_position,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
            let _ =
                chilen_backend::playback::play_new_queue(playlist.clone(), Some(initial_position));
            persistent_state::push_recent(state, playlist);
        }
        Message::PlayTrack(track) => {
            let _ = chilen_backend::playback::play_new_queue(
                Queue::Custom {
                    label: track
                        .title
                        .clone()
                        .map(|t| t.into_string())
                        .unwrap_or("Track".to_string()),
                    tracks: vec![track],
                },
                None,
            );
        }
        Message::AddTrackToQueue(track) => {
            let _ = chilen_backend::playback::append_to_queue(&mut vec![track]);
        }
        Message::AddAlbumToQueue(album) => {
            let _ = chilen_backend::playback::append_to_queue(&mut album.tracks.to_vec());
        }
        Message::AddArtistToQueue(artist) => {
            let _ = chilen_backend::playback::append_to_queue(&mut artist.tracks.to_vec());
        }
        Message::AddGenreToQueue(genre) => {
            let _ = chilen_backend::playback::append_to_queue(&mut genre.tracks.to_vec());
        }
        Message::AddPlaylistToQueue(playlist) => {
            let _ = chilen_backend::playback::append_to_queue(&mut playlist.tracks.to_vec());
        }
    }
    Task::none()
}
