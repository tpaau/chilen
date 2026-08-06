use std::path::PathBuf;

use rodio::Player;

use crate::{
    music_lib::{
        MUSIC_DIR,
        covers::LoadMode,
        indexer,
        state::{MUSIC_LIBRARY, MusicLibrary},
    },
    playback::{
        PLAYER_HANDLE, PlayerVolume,
        state::{PLAYER_STATE, PlayerState},
    },
};

/// Set the player state (queue, shuffle state, repeat mode, etc.) to the default value.
pub(crate) fn setup_player_state() {
    let mut state = PlayerState::default();
    state.set_player_volume(PlayerVolume::new(0.0));
    *PLAYER_STATE.write().unwrap() = Some(state);
}

/// Connect a player to the default audio sink and mute it.
pub(crate) fn setup_audio_sink() {
    // IDK what implications this has, ideally I'd open some sort of a "null sink" device.
    let handle =
        rodio::DeviceSinkBuilder::open_default_sink().expect("Couldn't open the default sink");
    let player = Player::connect_new(handle.mixer());
    // Mute the player so I don't hear screams of the damned during testing
    player.set_volume(0.0);
    *PLAYER_HANDLE.write().unwrap() = Some(player);
}

pub(crate) fn setup_music_library() {
    let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets/audio");
    path.canonicalize().expect("path should exist");
    *MUSIC_DIR.write().unwrap() = Some(path);
    let tracks = indexer::index(LoadMode::None, crate::music_lib::Config::default())
        .expect("Couldn't index the audio asset directory");
    *MUSIC_LIBRARY.write().unwrap() = Some(MusicLibrary::new_from_tracks(tracks));
}

pub(crate) fn setup_test_env() {
    setup_player_state();
    setup_audio_sink();
    setup_music_library();
}
