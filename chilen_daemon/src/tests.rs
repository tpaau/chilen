use std::path::PathBuf;

use chilen_ipc::{SocketType, playback::PlayerVolume};
use rodio::Player;

use crate::{
    AddrClaimMode, Config, Error, get_listener,
    music_lib::{
        MUSIC_DIR,
        covers::LoadMode,
        indexer,
        state::{MUSIC_LIBRARY, MusicLibrary},
    },
    playback::{
        self, PLAYER_HANDLE,
        state::{PLAYER_STATE, PlayerState},
    },
    quit, set_can_quit, set_can_raise, set_can_set_fullscreen,
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
    let tracks =
        indexer::index(LoadMode::Rebuild).expect("Couldn't index the audio asset directory");
    *MUSIC_LIBRARY.write().unwrap() = Some(MusicLibrary::new_from_tracks(tracks));
}

pub(crate) fn setup_test_env() {
    setup_player_state();
    setup_audio_sink();
    setup_music_library();
}

#[test]
fn default_config_works() {
    Config::try_default().unwrap();
}

// Tests:
//   - If the host supports namespaced sockets
//   - If namespaced socket listener creation fails if there is another listener present
#[test]
fn ns_connections() {
    let socket_name = "DAEMON_TEST_NS_SOCKET.socket";
    let st = SocketType::NamespacedOnly;
    let listener = get_listener(socket_name, &st, &AddrClaimMode::ClaimIfUnresponsive).unwrap();
    assert_eq!(
        get_listener(socket_name, &st, &AddrClaimMode::DoNotClaim).unwrap_err(),
        Error::SocketError
    );
    assert_eq!(
        get_listener(socket_name, &st, &AddrClaimMode::ClaimIfUnresponsive).unwrap_err(),
        Error::SocketError
    );
    assert_eq!(
        get_listener(socket_name, &st, &AddrClaimMode::ForceClaim).unwrap_err(),
        Error::SocketError
    );
    drop(listener);
}

// Tests:
//   - If filesystem socket name reclamation works properly
#[test]
fn fs_addr_reclamation() {
    let socket_name = "DAEMON_TEST_FS_SOCKET.socket";
    let st = SocketType::FilesystemOnly;
    let listener = get_listener(socket_name, &st, &AddrClaimMode::ClaimIfUnresponsive).unwrap();
    assert_eq!(
        get_listener(socket_name, &st, &AddrClaimMode::DoNotClaim).unwrap_err(),
        Error::AddrInUse
    );
    drop(listener);
    get_listener(socket_name, &st, &AddrClaimMode::ClaimIfUnresponsive).unwrap();
    get_listener(socket_name, &st, &AddrClaimMode::ForceClaim).unwrap();
}

// Make sure daemon functions fail but don't panic when the daemon isn't running.
#[test]
fn daemon_functions_fail() {
    assert_eq!(set_can_raise(false).unwrap_err(), Error::DaemonNotRunning);
    assert_eq!(
        set_can_set_fullscreen(false).unwrap_err(),
        Error::DaemonNotRunning
    );
    assert_eq!(set_can_quit(false).unwrap_err(), Error::DaemonNotRunning);
    assert_eq!(quit().unwrap_err(), Error::DaemonNotRunning);
    assert_eq!(
        playback::set_allow_rate_modification(false).unwrap_err(),
        Error::DaemonNotRunning
    );
}
