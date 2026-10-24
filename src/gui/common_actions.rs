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

use crate::gui::{Chilen, persistent_state};

/// Plays all tracks in the music library with the current shuffle state.
pub fn play_all_tracks(state: &Chilen, initial_position: usize) {
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
pub fn play_all_tracks_no_shuffle(state: &Chilen, initial_position: Option<usize>) {
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
pub fn play_playlist(state: &mut Chilen, playlist: Arc<Playlist>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::play_new_queue(playlist.clone(), initial_position);
    persistent_state::push_recent(state, playlist);
}

/// Disables shuffling and plays tracks in the given playlist.
pub fn play_playlist_no_shuffle(
    state: &mut Chilen,
    playlist: Arc<Playlist>,
    initial_position: Option<usize>,
) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
    let _ = chilen_backend::playback::play_new_queue(playlist.clone(), initial_position);
    persistent_state::push_recent(state, playlist);
}

/// Enables shuffling and plays tracks in the given playlist.
pub fn shuffle_playlist(
    state: &mut Chilen,
    playlist: Arc<Playlist>,
    initial_position: Option<usize>,
) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
    let _ = chilen_backend::playback::play_new_queue(playlist.clone(), initial_position);
    persistent_state::push_recent(state, playlist);
}

/// Plays tracks in the given album with the current shuffle state.
pub fn play_album(state: &mut Chilen, album: Arc<Album>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::play_new_queue(album.clone(), initial_position);
    persistent_state::push_recent(state, album);
}

/// Disables shuffling and plays tracks in the given album.
pub fn play_album_no_shuffle(
    state: &mut Chilen,
    album: Arc<Album>,
    initial_position: Option<usize>,
) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
    let _ = chilen_backend::playback::play_new_queue(album.clone(), initial_position);
    persistent_state::push_recent(state, album);
}

/// Enables shuffling and plays tracks in the given album.
pub fn shuffle_album(state: &mut Chilen, album: Arc<Album>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
    let _ = chilen_backend::playback::play_new_queue(album.clone(), initial_position);
    persistent_state::push_recent(state, album);
}

/// Plays tracks of the given artist with the current shuffle state.
pub fn play_artist(state: &mut Chilen, artist: Arc<Artist>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::play_new_queue(artist.clone(), initial_position);
    persistent_state::push_recent(state, artist);
}

/// Disables shuffling and plays tracks of the given artist.
pub fn play_artist_no_shuffle(
    state: &mut Chilen,
    artist: Arc<Artist>,
    initial_position: Option<usize>,
) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
    let _ = chilen_backend::playback::play_new_queue(artist.clone(), initial_position);
    persistent_state::push_recent(state, artist);
}

/// Enables shuffling and plays tracks of the given artist.
pub fn shuffle_artist(state: &mut Chilen, artist: Arc<Artist>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
    let _ = chilen_backend::playback::play_new_queue(artist.clone(), initial_position);
    persistent_state::push_recent(state, artist);
}

/// Plays all tracks in the music library with the given genre with the current shuffle state.
pub fn play_genre(state: &mut Chilen, genre: Arc<Genre>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::play_new_queue(genre.clone(), initial_position);
    persistent_state::push_recent(state, genre);
}

/// Disables shuffling and plays all tracks in the music library with the given genre.
pub fn play_genre_no_shuffle(
    state: &mut Chilen,
    genre: Arc<Genre>,
    initial_position: Option<usize>,
) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::Off);
    let _ = chilen_backend::playback::play_new_queue(genre.clone(), initial_position);
    persistent_state::push_recent(state, genre);
}

/// Enables shuffling and plays all tracks in the music library with the given genre.
pub fn shuffle_genre(state: &mut Chilen, genre: Arc<Genre>, initial_position: Option<usize>) {
    let _ = chilen_backend::playback::set_shuffle_state(ShuffleState::On);
    let _ = chilen_backend::playback::play_new_queue(genre.clone(), initial_position);
    persistent_state::push_recent(state, genre);
}

/// Appends multiple tracks to the queue.
pub fn append_tracks_to_queue(mut tracks: Vec<Arc<Track>>) {
    let _ = chilen_backend::playback::append_to_queue(&mut tracks);
}
