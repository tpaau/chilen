//! Sometimes more than one view needs to perform the same kind of action (eg. all main_view,
//! top_view and playback_view can display tracks and can add them to playlists), so this module
//! exists so that common functionality can be implemented here once instead of being copy-pasted
//! between modules.
use std::sync::Arc;

use chilen_backend::{
    music_lib::{Album, Artist, Genre, Playlist, Track},
    playback::{Queue, ShuffleState},
};
use log::error;

use crate::gui::Chilen;

/// Plays all tracks in the music library with the current shuffle state.
pub fn play_tracks(state: &Chilen, initial_position: usize) {
    match &state.library {
        Some(lib) => {
            let _ = chilen_backend::playback::play_new_queue(
                Queue::AllTracks(lib.tracks.clone()),
                Some(initial_position),
            );
        }
        None => error!("Cannot play the track, the library is not loaded"),
    }
}

/// Disables shuffling and plays all tracks in the music library.
pub fn play_tracks_no_shuffle(state: &Chilen, initial_position: Option<usize>) {
    match &state.library {
        Some(lib) => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
            let _ = chilen_backend::playback::play_new_queue(
                Queue::AllTracks(lib.tracks.clone()),
                initial_position,
            );
        }
        None => error!("Cannot play the track, the library is not loaded"),
    }
}

/// Enables shuffling and plays all tracks in the music library.
pub fn shuffle_tracks(state: &Chilen, initial_position: Option<usize>) {
    match &state.library {
        Some(lib) => {
            let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
            let _ = chilen_backend::playback::play_new_queue(
                Queue::AllTracks(lib.tracks.clone()),
                initial_position,
            );
        }
        None => error!("Cannot play the track, the library is not loaded"),
    }
}

/// Plays tracks in the given playlist with the current shuffle state.
pub fn play_playlist(playlist: Arc<Playlist>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::play_new_queue(Queue::Playlist(playlist), initial_position);
}

/// Disables shuffling and plays tracks in the given playlist.
pub fn play_playlist_no_shuffle(playlist: Arc<Playlist>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
    let _ = chilen_backend::playback::play_new_queue(Queue::Playlist(playlist), initial_position);
}

/// Enables shuffling and plays tracks in the given playlist.
pub fn shuffle_playlist(playlist: Arc<Playlist>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
    let _ = chilen_backend::playback::play_new_queue(Queue::Playlist(playlist), initial_position);
}

/// Plays tracks in the given album with the current shuffle state.
pub fn play_album(album: Arc<Album>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::play_new_queue(Queue::Album(album), initial_position);
}

/// Disables shuffling and plays tracks in the given album.
pub fn play_album_no_shuffle(album: Arc<Album>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
    let _ = chilen_backend::playback::play_new_queue(Queue::Album(album), initial_position);
}

/// Enables shuffling and plays tracks in the given album.
pub fn shuffle_album(album: Arc<Album>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
    let _ = chilen_backend::playback::play_new_queue(Queue::Album(album), initial_position);
}

/// Plays tracks of the given artist with the current shuffle state.
pub fn play_artist(artist: Arc<Artist>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::play_new_queue(Queue::Artist(artist), initial_position);
}

/// Disables shuffling and plays tracks of the given artist.
pub fn play_artist_no_shuffle(artist: Arc<Artist>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
    let _ = chilen_backend::playback::play_new_queue(Queue::Artist(artist), initial_position);
}

/// Enables shuffling and plays tracks of the given artist.
pub fn shuffle_artist(artist: Arc<Artist>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
    let _ = chilen_backend::playback::play_new_queue(Queue::Artist(artist), initial_position);
}

/// Plays all tracks in the music library with the given genre with the current shuffle state.
pub fn play_genre(genre: Arc<Genre>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::play_new_queue(Queue::Genre(genre), initial_position);
}

/// Disables shuffling and plays all tracks in the music library with the given genre.
pub fn play_genre_no_shuffle(genre: Arc<Genre>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
    let _ = chilen_backend::playback::play_new_queue(Queue::Genre(genre), initial_position);
}

/// Enables shuffling and plays all tracks in the music library with the given genre.
pub fn shuffle_genre(genre: Arc<Genre>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
    let _ = chilen_backend::playback::play_new_queue(Queue::Genre(genre), initial_position);
}

/// Appends multiple tracks to the queue.
pub fn append_tracks_to_queue(mut tracks: Vec<Arc<Track>>) {
    let _ = chilen_backend::playback::append_to_queue(&mut tracks);
}
