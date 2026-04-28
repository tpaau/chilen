#![doc = include_str!("../README.md")]

use std::{
    env::temp_dir,
    io::{BufReader, Write},
    path::PathBuf,
    time::Duration,
};

pub use interprocess::local_socket::Stream;
use interprocess::local_socket::{GenericFilePath, GenericNamespaced, Name, ToNsName, prelude::*};
use log::info;
use log::{error, trace};
use serde::{Deserialize, Serialize};

/// The default name of the socket the daemon listens on.
pub const DEFAULT_SOCKET_NAME: &str = "DEFAULT_MUSIC_PLAYER.socket";

/// An error originating from the music library module of the `daemon`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MusicLibraryError {
    /// Could not complete the operation because a [playlist](Playlist) with the provided name
    /// already exists.
    PlaylistExists,
    /// Could not perform the operation because the music library is not initialized.
    ///
    /// This can happen if a command is sent to early and the music library is not yet initialized.
    LibraryNotInitialized,
    /// There is not playlist in the music library with the provided name.
    NoSuchPlaylist,
    /// Could not get the path to the cache directory or the cache is unusable.
    CacheError,
    /// The provided item index was out of bounds.
    IndexOutOfBounds,
}

impl std::fmt::Display for MusicLibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaylistExists => write!(f, "Playlist with this name already exists"),
            Self::LibraryNotInitialized => write!(f, "The music library is not initialized"),
            Self::NoSuchPlaylist => write!(f, "There is no playlist with this name"),
            Self::CacheError => write!(f, "Cache is unusable"),
            Self::IndexOutOfBounds => write!(f, "The provided item index was out of bounds"),
        }
    }
}

/// Error originating from the playback module of the `daemon`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaybackError {
    /// The audio player is not connected.
    ///
    /// This may happen if the device doesn't have an audio device or none of the audio devices are
    /// marked as default.
    PlayerNotConnected,
    /// The playback state is not initialized.
    ///
    /// This error may occur when a [PlaybackCommand] is sent to the `daemon` too early, before the
    /// state is restored from cache.
    StateNotInitialized,
    /// The queue is empty.
    QueueEmpty,
    /// The audio file could not be opened, has an unsupported format or is corrupt.
    SourceError,
    /// The player is already playing.
    PlayerPlaying,
    /// The player is already paused.
    PlayerPaused,
    /// Thrown when a client attempts to stop the player when it was already stopped or when a
    /// client attempts to seek while the player is stopped.
    PlayerStopped,
    /// Seek is not supported for the current audio source.
    SeekNotSupported,
    /// Cannot go to the previous track.
    ///
    /// This means that the current track is first in the queue and the [loop state](LoopState) is
    /// set to [LoopState::Off].
    CannotGoPrevious,
    /// Cannot go to the next track.
    ///
    /// This means that the current track is last in the queue and the [loop state](LoopState) is
    /// set to [LoopState::Off].
    CannotGoNext,
    /// The daemon was not built with shuffle support.
    ShuffleNotSupported,
    /// No track at this index.
    NoTrackAtIndex(usize),
    /// The specified rate value was out of the allowed range.
    RateOutOfRange,
    /// The modification of the playback rate is not allowed.
    FixedRate,
    /// The player position could not be set because the duration provided was invalid.
    ///
    /// The player will refuse to seek by 0s to prevent unnecessary audio popping.
    InvalidDuration,
    /// Overflow detected while performing a seek operation.
    DurationOverflow,
}

impl std::fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayerNotConnected => write!(f, "The audio player is not connected"),
            Self::StateNotInitialized => write!(f, "The playback state is not initialized"),
            Self::QueueEmpty => write!(f, "The queue is empty"),
            Self::SourceError => write!(
                f,
                "The audio file could not be opened, has an unsupported format or is corrupt"
            ),
            Self::PlayerPlaying => write!(f, "The player is already playing"),
            Self::PlayerPaused => write!(f, "The player is already paused"),
            Self::PlayerStopped => write!(f, "The player is stopped"),
            Self::SeekNotSupported => write!(f, "Seek is not supported"),
            Self::CannotGoPrevious => write!(f, "Cannot go to the previous track"),
            Self::CannotGoNext => write!(f, "Cannot go to the next track"),
            Self::ShuffleNotSupported => write!(f, "The daemon was not built with shuffle support"),
            Self::NoTrackAtIndex(index) => write!(f, "No track was found at index {index}"),
            Self::RateOutOfRange => {
                write!(f, "The specified rate value was out of the allowed range")
            }
            Self::FixedRate => write!(f, "The modification of the playback rate is disallowed"),
            Self::InvalidDuration => write!(
                f,
                "The player position could not be set because the duration provided was invalid"
            ),
            Self::DurationOverflow => {
                write!(f, "Overflow detected while performing a seek operation")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataError {
    PermissionError,
    CacheDirNotAvailable,
    DataDirNotAvailable,
    NoMusicLibrary,
    /// Could not get the home directory path.
    HomeError,
    NoPicturesInTag,
    NoSuitablePicturesInTag,
    CoverWriteError,
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PermissionError => write!(
                f,
                "Could not perform an operation due to a permission error"
            ),
            Self::CacheDirNotAvailable => {
                write!(f, "The cache directory is not readable and/or writable")
            }
            Self::DataDirNotAvailable => {
                write!(f, "The data directory is not readable and/or writable")
            }
            Self::NoMusicLibrary => write!(f, "The music library directory does not exist"),
            Self::HomeError => write!(f, "Could not get the home directory path"),
            Self::NoPicturesInTag => write!(f, "No pictures were found in the audio file tag"),
            Self::NoSuitablePicturesInTag => {
                write!(f, "Couldn't find any suitable images in the audio file tag")
            }
            Self::CoverWriteError => write!(
                f,
                "Could not write the cover image contents to the covers cache"
            ),
        }
    }
}

/// Struct representing a track from the [music library](MusicLibrary).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Track {
    /// The path to the audio file.
    pub path: PathBuf,
    /// The path to the extracted cover file.
    pub cover_path: Option<PathBuf>,
    /// The duration of the track.
    pub duration: Duration,
    // TODO: Add an option to split the tag with characters like ",", ";", "/", etc.
    /// The track artist.
    pub artist: Option<String>,
    /// The track title.
    pub title: Option<String>,
    /// The track album.
    pub album: Option<String>,
    // TODO: Same as with artist
    /// The track genre.
    pub genre: Option<String>,
    /// Possibly synchronized lyrics text.
    pub lyrics: Option<String>,
    /// Contents of the comment tag.
    pub comment: Option<String>,
    pub track: Option<u32>,
    pub track_total: Option<u32>,
    pub disc: Option<u32>,
    pub disc_total: Option<u32>,
    /// Release year.
    pub year: Option<u32>,
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} ({})",
            self.artist.clone().unwrap_or(String::from("Unknown")),
            self.title.clone().unwrap_or(String::from("Unknown")),
            self.path.to_string_lossy()
        )
    }
}

/// Struct representing a playlist in the [music library](MusicLibrary).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Playlist {
    /// The name of the playlist.
    ///
    /// Playlist names are unique.
    pub name: String,
    /// The content of the playlist.
    ///
    /// All tracks must already be in the [music library](MusicLibrary). If a track is removed from
    /// the library (eg. by removing an audio file from the music directory and reloading the
    /// library), it will also be removed from all the playlists.
    pub tracks: Vec<Track>,
}

/// Struct representing the contents of the music library.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MusicLibrary {
    /// The list of [playlists](Playlist) in the music library.
    pub playlists: Vec<Playlist>,
    /// The list of all [tracks](Track) in the music library.
    pub tracks: Vec<Track>,
}

/// Event originating from the playback module of the `daemon`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackEvent {
    /// Sent when the [playback state](PlaybackState) changes, for instance when the player is
    /// paused.
    PlaybackStateChanged(PlaybackState),
    /// Sent when the [loop state](LoopState) of the player changes.
    LoopStateChanged(LoopState),
    /// Sent when the [shuffle state](ShuffleState) of the player changes.
    ShuffleStateChanged(ShuffleState),
    /// Sent when the track queue changes.
    QueueChanged(Vec<Track>),
    /// Sent when the current track changes.
    PositionChanged(usize),
    /// Sent when the position of the player changes.
    PlayerPositionChanged(Duration),
    /// Sent when the [volume](PlayerVolume) of the player changes.
    PlayerVolumeChanged(PlayerVolume),
    /// Sent when the [playback rate](PlaybackRate) of the player changes.
    RateChanged(PlaybackRate),
}

/// Event from the `daemon` received in [DaemonResponse::Event].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DaemonEvent {
    /// Sent before the `daemon` closes.
    Shutdown,
    /// Sent after the contents of the music library have changed.
    MusicLibraryChanged(MusicLibrary),
    /// Sent when a client disconnects from the daemon.
    ConnectionClosed,
    /// Event originating from the playback module of the daemon.
    PlaybackEvent(PlaybackEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Error that can occur while creating a daemon configuration.
pub enum ConfigError {
    /// The provided bus name suffix for MPRIS was invalid.
    InvalidBusNameSuffix,
    /// Could not get the home directory path.
    HomeError,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBusNameSuffix => write!(f, "The bus name suffix provided was invalid"),
            Self::HomeError => write!(f, "Could not get the home directory path"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Error related to the `daemon`.
///
/// Can either originate from a [DaemonResponse] or from a function in [mpipc](crate).
pub enum DaemonError {
    /// The value could not be encoded before sending it.
    EncodingError,
    /// The encoded value could not be decoded back into usable form.
    DecodingError,
    /// Could not obtain the daemon socket address.
    SocketError,
    /// Could not connect to the daemon.
    ConnectionError,
    /// Could not send the command to the daemon.
    SendingError,
    /// Error related to the music library.
    MusicLibraryError(MusicLibraryError),
    /// Error related to the data and cache modules.
    DataError(DataError),
    /// The response received from the daemon was unexpected or invalid.
    InvalidResponse,
    /// Error related to the playback module.
    PlaybackError(PlaybackError),
    /// Error that can occur while creating a daemon configuration.
    ConfigError(ConfigError),
    /// The socket address is already in use.
    AddrInUse,
    /// Emitted when the daemon event channel is already initialized when starting the daemon.
    ///
    /// This likely means a second daemon was started in the same context.
    EventChannelInitialized,
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncodingError => write!(f, "Could not encode the deamon conmmand"),
            Self::DecodingError => write!(f, "Could not decode the response from the daemon"),
            Self::SocketError => write!(f, "Socket cration/connection failed"),
            Self::ConnectionError => write!(f, "Could not connect to the daemon"),
            Self::SendingError => write!(f, "Could not send the command to the daemon"),
            Self::MusicLibraryError(e) => {
                write!(f, "{e}")
            }
            Self::DataError(e) => write!(f, "Data error: {e}"),
            Self::InvalidResponse => {
                write!(f, "The response from the daemon was invalid or malformed")
            }
            Self::PlaybackError(e) => write!(f, "Playback error: {e}"),
            Self::ConfigError(e) => write!(f, "Could not create daemon configuration: {e}"),
            Self::AddrInUse => write!(f, "The socket address is already in use"),
            Self::EventChannelInitialized => {
                write!(f, "The event channel for the daemon is already initialized")
            }
        }
    }
}

/// Response sent to a client from the `daemon`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DaemonResponse {
    /// The client command was executed successfully.
    Ok,
    /// Response to the `Ping` client command.
    Pong,
    /// The contents of the music library.
    Library(MusicLibrary),
    /// An event from the daemon.
    Event(DaemonEvent),
    /// Response from a playback command.
    Playback(PlaybackResponse),
    /// The client command has failed.
    Error(DaemonError),
}

/// Signed duration type used for seeking.
///
/// This is just a bare bones type that should only be used for
/// [seeking audio playback](PlaybackCommand::Seek), and not as a [Duration] replacement.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SignedDuration {
    Positive(Duration),
    Negative(Duration),
}

impl SignedDuration {
    pub fn from_secs<T: Into<i64> + Copy>(secs: T) -> SignedDuration {
        if secs.into() < 0 {
            SignedDuration::Negative(Duration::from_secs(secs.into().abs().try_into().unwrap()))
        } else {
            SignedDuration::Positive(Duration::from_secs(secs.into().abs().try_into().unwrap()))
        }
    }
}

/// Subcommand of [ClientCommand] for managing audio playback in the `daemon`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackCommand {
    /// Play the current track or play a track at a specific index in the queue.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Play(Option<usize>),
    /// Pause the player.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Pause,
    /// Stop the player.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Stop,
    /// Toggle between play/pause.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    TogglePlaying,
    /// Get the [PlaybackState] of the player.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    GetPlaybackState,
    // TODO: Update this after patching the daemon
    /// Set a new queue for the player.
    ///
    /// **Note:** tracks outside of the music directory or not registered by the music player
    /// (added after the last library reload), will be discarded.
    SetQueue(Vec<PathBuf>),
    // TODO: Update this after patching the daemon
    /// Append tracks to the queue.
    ///
    /// **Note:** tracks outside of the music directory or not registered by the music player
    /// (added after the last library reload), will be discarded.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    AppendToQueue(Vec<PathBuf>),
    /// Load a playlist and append its tracks to the queue.
    ///
    /// If there's no [Playlist] with the provided name present in the [MusicLibrary],
    /// [MusicLibraryError::NoSuchPlaylist] will be returned, and no changes to the queue will be
    /// made.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    AppendPlaylist(String),
    /// Load a playlist and put its tracks in the queue.
    ///
    /// If there's no [Playlist] with the provided name present in the [MusicLibrary],
    /// [MusicLibraryError::NoSuchPlaylist] will be returned, and no changes to the queue will be
    /// made.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    SetPlaylist(String),
    /// Get the current [track](Track).
    ///
    /// The `daemon` will respond to this with [PlaybackResponse::Track] if successful.
    GetCurrentTrack,
    /// Skip to the next track.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Next,
    /// Skip to the previous track.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Previous,
    /// Set the [loop state](LoopState) of the player.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    SetLoopState(LoopState),
    /// Get the [loop state](LoopState) of the player.
    ///
    /// The `daemon` will respond to this with [PlaybackResponse::LoopState] if
    /// successful.
    GetLoopState,
    /// Set the [playback rate](PlaybackRate) of the player.
    ///
    /// This command will fail if the `daemon` is configured to not allow playback rate
    /// modification or if the specified rate value was out of the acceptable range.
    ///
    /// If the `daemon` is configured not to allow playback rate modification,
    /// [PlaybackError::FixedRate] will be returned.
    ///
    /// If the provided rate value is out of the allowed range, [PlaybackError::RateOutOfRange]
    /// will be returned.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    SetRate(f64),
    /// Get the [playback rate](PlaybackRate) of the player.
    ///
    /// The `daemon` will respond to this with [PlaybackResponse::PlaybackRate] if
    /// successful.
    GetRate,
    /// Set the [shuffle state](ShuffleState) of the player.
    ///
    /// The `daemon` will always respond to this command with [PlaybackError::ShuffleNotSupported]
    /// if it was built without shuffle support.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    SetShuffleState(ShuffleState),
    /// Get the [shuffle state](ShuffleState) of the player.
    ///
    /// The `daemon` will always respond to this command with [ShuffleState::Off] if it was built
    /// without shuffle support.
    ///
    /// The `daemon` will respond to this with [PlaybackResponse::ShuffleState] if
    /// successful.
    GetShuffleState,
    /// Set the position of the player.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    SetPlayerPosition(Duration),
    /// Change the player position by a time delta.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Seek(SignedDuration),
    /// Get the position of the player.
    ///
    /// The `daemon` will respond to this with [PlaybackResponse::PlayerPosition] if
    /// successful.
    GetPlayerPosition,
    /// Set the volume of the player.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    SetPlayerVolume(PlayerVolume),
    /// Get the volume of the player.
    ///
    /// The `daemon` will respond to this with [PlaybackResponse::PlayerVolume] if
    /// successful.
    GetPlayerVolume,
}

/// Response originating from the playback module of the `daemon`.
///
/// Used as an enum value for the [DaemonResponse].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackResponse {
    PlaybackState(PlaybackState),
    LoopState(LoopState),
    PlaybackRate(PlaybackRate),
    ShuffleState(ShuffleState),
    Track(Option<Box<Track>>),
    PlayerVolume(PlayerVolume),
    PlayerPosition(Duration),
}

impl std::fmt::Display for PlaybackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaybackState(state) => write!(f, "{state}"),
            Self::LoopState(state) => write!(f, "{state}"),
            Self::PlaybackRate(rate) => write!(f, "{rate}"),
            Self::ShuffleState(state) => write!(f, "{state}"),
            Self::Track(track) => match track {
                Some(track) => write!(f, "{track}"),
                None => write!(f, "None"),
            },
            Self::PlayerVolume(volume) => write!(f, "{volume}"),
            Self::PlayerPosition(position) => write!(f, "{position:?}"),
        }
    }
}

/// The speed at which tracks are played.
///
/// Rate of 1.0 will play tracks at their original speed, 2.0 will play them twice as fast, and 0.5
/// will slow them to half speed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlaybackRate {
    rate: f64,
    min: f64,
    max: f64,
}

impl Default for PlaybackRate {
    fn default() -> Self {
        Self {
            rate: 1.0,
            min: 0.1,
            max: 10.0,
        }
    }
}

impl<T: Into<f64>> From<T> for PlaybackRate {
    fn from(value: T) -> Self {
        let mut def = Self::default();
        def.set_value(value.into());
        def
    }
}

impl std::fmt::Display for PlaybackRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Current playback rate: {}, min: {}, max: {}",
            self.rate, self.min, self.max
        )
    }
}

impl PlaybackRate {
    /// Create a new [PlaybackRate] struct.
    pub fn new(rate: f64, min: f64, max: f64) -> PlaybackRate {
        Self { rate, min, max }
    }

    fn clamp(&mut self) {
        self.rate = self.rate.clamp(self.min, self.max)
    }

    /// Checks if the given rate value is in the allowed range.
    pub fn is_in_range(&self, rate: f64) -> bool {
        rate >= self.min && rate <= self.max
    }

    /// Set the rate multiplier.
    ///
    /// This value will be clamped with the min and max values.
    pub fn set_value(&mut self, rate: f64) {
        self.rate = rate.clamp(self.min, self.max)
    }

    /// Get the rate multiplier.
    pub fn get_value(&self) -> f64 {
        self.rate
    }

    /// Get the rate multiplier clamped to `f32`.
    pub fn get_value_f32(&self) -> f32 {
        if self.rate.is_nan() {
            f32::NAN
        } else if self.rate > f32::MAX as f64 {
            f32::MAX
        } else if self.rate < f32::MIN as f64 {
            f32::MIN
        } else {
            self.rate as f32
        }
    }

    /// Set the minimum acceptable value for the playback rate.
    pub fn set_min(&mut self, min: f64) {
        self.min = min.clamp(0.0, self.max);
        self.clamp();
    }

    /// Get the minimum acceptable value for the playback rate.
    pub fn get_min(&self) -> f64 {
        self.min
    }

    /// Set the maximum acceptable value for the playback rate.
    pub fn set_max(&mut self, max: f64) {
        self.max = max.clamp(self.min, f64::MAX);
        self.clamp();
    }

    /// Get the maximum acceptable value for the playback rate.
    pub fn get_max(&self) -> f64 {
        self.max
    }
}

/// Player volume.
///
/// Values passed to this struct will be clamped between 0.0 (no sound at all) and 1.0 (regular
/// volume).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlayerVolume {
    volume: f64,
}

impl Default for PlayerVolume {
    fn default() -> Self {
        Self { volume: 1.0 }
    }
}

impl std::fmt::Display for PlayerVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player volume: {}", self.volume)
    }
}

impl PlayerVolume {
    /// Create a new [PlayerVolume] struct with the specified volume.
    ///
    /// The passed `volume` parameter will be clamped between 0.0 and 1.0.
    pub fn new(volume: f64) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
        }
    }

    /// Set the volume.
    ///
    /// The passed `volume` parameter will be clamped between 0.0 and 1.0.
    pub fn set(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Get the wrapped value.
    pub fn get(&self) -> f64 {
        self.volume
    }
}

/// Playback state of the player.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaybackState {
    /// The player is playing.
    Playing,
    /// The player is paused.
    Paused,
    /// Audio playback is stopped. Playing will play the current track from the beginning.
    #[default]
    Stopped,
}

impl std::fmt::Display for PlaybackState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Playing => write!(f, "Playing"),
            Self::Paused => write!(f, "Paused"),
            Self::Stopped => write!(f, "Stopped"),
        }
    }
}

/// Shuffle state of the player.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShuffleState {
    /// Do not shuffle the queue.
    #[default]
    Off,
    /// Shuffle the tracks in the queue.
    On,
}

impl From<ShuffleState> for bool {
    fn from(s: ShuffleState) -> bool {
        match s {
            ShuffleState::Off => false,
            ShuffleState::On => true,
        }
    }
}

impl From<bool> for ShuffleState {
    fn from(value: bool) -> Self {
        if value { Self::On } else { Self::Off }
    }
}

impl std::fmt::Display for ShuffleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::On => write!(f, "On"),
        }
    }
}

/// Loop state defining the looping behavior of the player.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoopState {
    /// The playback will stop when there are no more tracks to play.
    #[default]
    Off,
    /// The current track will start again from the beginning once it has finished playing.
    Track,
    /// The playback will loop through the entire queue.
    Playlist,
}

impl std::fmt::Display for LoopState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Track => write!(f, "Track"),
            Self::Playlist => write!(f, "Playlist"),
        }
    }
}

/// Subcommand of [ClientCommand] for managing playlists in the music library.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaylistCommand {
    /// Create a new playlist, optionally with some tracks in it.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    New {
        /// The name for the new playlist, must not already exist in the music library.
        ///
        /// If a playlist with the specified name already exists
        /// [MusicLibraryError::PlaylistExists] will be returned.
        name: String,
        // TODO: Update this after patching the daemon
        /// Optional list of paths to tracks to be added to the playlist.
        ///
        /// **Note:** tracks outside of the music directory or not registered by the music player
        /// (added after the last library reload), will be discarded.
        tracks: Option<Vec<PathBuf>>,
    },
    /// Import a playlist from an M3U8 file.
    ///
    /// This is currently unimplemented and will cause the `daemon` to panic every time.
    FromM3U8 {
        /// The name for the imported playlist, must not already exist in the music library.
        ///
        /// If left unspecified, it will be derived from the name of the imported file.
        ///
        /// If a playlist with the specified name already exists
        /// [MusicLibraryError::PlaylistExists] will be returned.
        name: Option<String>,
        /// The path to the M3U8 file to import.
        m3u8_file: PathBuf,
    },
    /// Delete playlists from the music library.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Delete {
        /// List of the playlists to delete.
        ///
        /// If any of the provided playlists don't exist in the music library,
        /// [MusicLibraryError::NoSuchPlaylist] will be returned, and no changes to the music
        /// library will be made.
        names: Vec<String>,
    },
    /// Add tracks to an already existing playlist.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    AddTracks {
        /// The name of the playlist to add tracks to.
        ///
        /// If a playlist with the specified name doesn't exist in the music library,
        /// [MusicLibraryError::NoSuchPlaylist] will be returned.
        name: String,
        // TODO: Update this after patching the daemon
        /// List of paths to tracks to add to the playlist.
        ///
        /// **Note:** tracks with paths outside of the music directory or not registered by the
        /// music player (added after the last library reload), will be discarded.
        tracks: Vec<PathBuf>,
    },
    /// Remove tracks from a playlist.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    RemoveTracks {
        /// The name of the playlist to remove tracks from.
        ///
        /// If a playlist with the specified name doesn't exist in the music library,
        /// [MusicLibraryError::NoSuchPlaylist] will be returned.
        name: String,
        /// The list of track indices in the playlist to remove.
        ///
        /// Eg. to remove the first track you would pass `[0]`, to remove the first three
        /// `[0, 1, 2]`, etc.
        ///
        /// If one of the indices is out of range, the daemon will return
        /// [MusicLibraryError::IndexOutOfBounds]. No changes will be made.
        ids: Vec<usize>,
    },
    /// Get the contents of the [music library](MusicLibrary).
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Library] if successful.
    GetLibrary,
}

/// Subcommand of [`ClientCommand`] for managing `daemon` cover art cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheCommand {
    /// Rebuild the cache ignoring already cached covers.
    ///
    /// Will take more time than just reloading the cache.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Rebuild,
    /// Reinitialize cache using already cached covers when possible.
    ///
    /// This can be used to discover newly added tracks.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Reload,
}

/// Command that can be executed by a `daemon` instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientCommand {
    /// Stop the `daemon` instance.
    ///
    /// After sending this command, the `daemon` will close almost immediately, so the connection
    /// to it should be considered to be closed.
    Shutdown,
    /// Subcommand for managing playlists in the music library.
    Playlist(PlaylistCommand),
    /// Subcommand for managing cover art cache.
    Cache(CacheCommand),
    /// Stream events from the `daemon`.
    ///
    /// The daemon will stop accepting requests from the connection this command was executed on.
    EventStream,
    /// Close the connection to the `daemon`.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Disconnect,
    /// Command to the playback module.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Ok] if successful.
    Playback(PlaybackCommand),
    /// Ping the `daemon`.
    ///
    /// The `daemon` will respond to this with [DaemonResponse::Pong] if successful.
    Ping,
}

/// Defines the socket type to use when attempting to connect to a `deamon`.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocketType {
    /// Only use a namespaced socket with no fallback.
    ///
    /// The `daemon` will return [DaemonError::SocketError] at startup if a socket with the
    /// specified name already exists.
    NamespacedOnly,
    /// Use a namespaced socket when possible, but allow a fallback to a filesystem socket.
    #[default]
    NamespacedOrFilesystem,
    /// Only use a filesystem socket with no fallback.
    ///
    /// The `daemon` will return [DaemonError::AddrInUse] at startup if a socket with the specified
    /// name already exists
    FilesystemOnly,
}

/// Returns a filesystem path for a given socket address.
pub fn get_fs_socket_path(socket_name: &str) -> PathBuf {
    let mut temp_dir = temp_dir();
    temp_dir.push(socket_name);
    temp_dir
}

/// Returns a namespaced socket for the given socket name.
///
/// # Examples
/// ```
/// # use mpipc::{get_ns_daemon_socket, DEFAULT_SOCKET_NAME};
/// let socket = get_ns_daemon_socket(DEFAULT_SOCKET_NAME).unwrap();
/// ```
pub fn get_ns_daemon_socket<'a>(socket_name: &'a str) -> Result<Name<'a>, std::io::Error> {
    match socket_name.to_ns_name::<GenericNamespaced>() {
        Ok(socket) => Ok(socket),
        Err(e) => {
            error!("Could not obtain a namespaced socket: {e}");
            Err(e)
        }
    }
}

/// Returns a filesystem socket for the given socket name.
///
/// # Examples
/// ```
/// # use mpipc::{get_fs_daemon_socket, DEFAULT_SOCKET_NAME};
/// let socket = get_fs_daemon_socket(DEFAULT_SOCKET_NAME).unwrap();
/// ```
pub fn get_fs_daemon_socket<'a>(socket_name: &'a str) -> Result<Name<'a>, std::io::Error> {
    let socket_path = get_fs_socket_path(socket_name);
    match socket_path.to_fs_name::<GenericFilePath>() {
        Ok(socket) => Ok(socket),
        Err(e) => {
            error!("Could not obtain a filesystem socket: {e}");
            Err(e)
        }
    }
}

/// Attempts to get a socket address for `daemon` IPC.
///
/// # Examples
/// ```
/// match mpipc::get_socket(
///     mpipc::DEFAULT_SOCKET_NAME,
///     &mpipc::SocketType::NamespacedOrFilesystem
/// ) {
///     Ok(socket) => eprintln!("Got a socket: {socket:?}"),
///     Err(e) => panic!("Could not obtain a socket: {e}"),
/// }
/// ```
pub fn get_socket<'a>(socket_name: &'a str, mode: &SocketType) -> Result<Name<'a>, std::io::Error> {
    match mode {
        SocketType::NamespacedOnly => get_ns_daemon_socket(socket_name),
        SocketType::FilesystemOnly => {
            get_fs_socket_path(socket_name).to_fs_name::<GenericFilePath>()
        }
        SocketType::NamespacedOrFilesystem => match get_ns_daemon_socket(socket_name) {
            Ok(socket) => Ok(socket),
            Err(e) => {
                info!("Could not obtain a namespaced socket: {e}");
                info!("Trying a filesystem socket instead");
                get_fs_socket_path(socket_name).to_fs_name::<GenericFilePath>()
            }
        },
    }
}

/// Serialize a client command to a format that can be sent to the daemon.
///
/// # Examples
///
/// Connect to the daemon and immediately disconnect.
/// ```no_run
/// # use std::io::{BufReader, Write};
/// let mut conn = BufReader::new(mpipc::connect_to_daemon(mpipc::DEFAULT_SOCKET_NAME, &mpipc::SocketType::default()).unwrap());
/// let cmd = mpipc::serialize_client_command(&mpipc::ClientCommand::Disconnect).unwrap();
/// conn.get_mut().write_all(&cmd).unwrap();
/// ```
pub fn serialize_client_command(cmd: &ClientCommand) -> Result<Vec<u8>, DaemonError> {
    let mut data = Vec::new();
    if let Err(e) = cmd.serialize(&mut rmp_serde::Serializer::new(&mut data)) {
        error!("Could not encode the client command: {e}");
        return Err(DaemonError::EncodingError);
    }
    Ok(data)
}

/// Receive a daemon response from a buffered stream connection.
///
/// This function will block until a response is received or the connection is dropped.
///
/// # Examples
/// ```no_run
/// # use std::io::{BufReader, Write};
/// # use mpipc::{connect_to_daemon, DEFAULT_SOCKET_NAME, SocketType, serialize_client_command, ClientCommand, receive_daemon_response};
/// let mut conn = BufReader::new(connect_to_daemon(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap());
/// let cmd = serialize_client_command(&ClientCommand::EventStream).unwrap();
/// conn.get_mut().write_all(&cmd).unwrap();
/// loop {
///     let response = receive_daemon_response(&mut conn).unwrap();
///     println!("Got a response from the daemon: {response:?}");
/// }
/// ```
pub fn receive_daemon_response(
    conn: &mut BufReader<Stream>,
) -> Result<DaemonResponse, DaemonError> {
    match rmp_serde::from_read(conn) {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Failed decoding a daemon response: {e}");
            Err(DaemonError::DecodingError)
        }
    }
}

/// Disconnects from the `daemon` by sending the [ClientCommand::Disconnect] command.
///
/// This is a convenience function, its effect could be achieved using utilities already provided
/// by [mpipc](crate).
///
/// # Examples
/// ```no_run
/// # use std::io::BufReader;
/// # use mpipc::{connect_to_daemon, DEFAULT_SOCKET_NAME, SocketType, disconnect};
/// let conn = connect_to_daemon(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap();
/// let mut conn = BufReader::new(conn);
///
/// // Do some stuff with the connection here...
///
/// match disconnect(&mut conn) {
///     Ok(_) => eprintln!("Disconnected from the deamon!"),
///     Err(e) => panic!("Could not close the daemon connection: {e}"),
/// }
/// ```
pub fn disconnect(conn: &mut BufReader<Stream>) -> Result<(), DaemonError> {
    trace!("Closing connection with the daemon");

    let data = serialize_client_command(&ClientCommand::Disconnect)?;

    match conn.get_mut().write_all(&data) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed sending the command to the daemon: {e}");
            return Err(DaemonError::SendingError);
        }
    }

    let response = receive_daemon_response(conn)?;

    match response {
        DaemonResponse::Error(e) => {
            error!("Could not close the connection to the daemon: {e}");
            return Err(e);
        }
        DaemonResponse::Ok => {
            trace!("Connection with the daemon closed");
        }
        _ => {
            error!("Got an unexpected response while closing the daemon connection: {response:?}");
            return Err(DaemonError::InvalidResponse);
        }
    }

    Ok(())
}

/// Connects to the `daemon` via a local socket and returns the connection [Stream].
///
/// # Examples
/// ```no_run
/// use std::io::BufReader;
/// # use mpipc::{connect_to_daemon, DEFAULT_SOCKET_NAME, SocketType, disconnect};
/// let mut conn = BufReader::new(connect_to_daemon(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap());
/// eprintln!("Connected to the daemon: {conn:?}");
/// // Run all your commands here!
/// disconnect(&mut conn).unwrap();
/// ```
pub fn connect_to_daemon(
    socket_name: &str,
    socket_type: &SocketType,
) -> Result<Stream, DaemonError> {
    trace!("Connecting to daemon on socket '{socket_name}'");

    let socket = match get_socket(socket_name, socket_type) {
        Ok(sock) => sock,
        Err(e) => {
            error!("Could not obtain a socket: {e}");
            return Err(DaemonError::SocketError);
        }
    };

    let conn = match Stream::connect(socket) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Could not initialize a connection to the daemon: {e}");
            return Err(DaemonError::ConnectionError);
        }
    };

    Ok(conn)
}

/// Executes a single `daemon` command and ends the connection.
///
/// This function needs to connect to the `daemon` every time it's called, which is very
/// inefficient.
///
/// **Warning:** if this function returns the [Ok] variant, this only means that the command was
/// successfully delivered to the `daemon`. It doesn't necessarily mean that the command was
/// executed successfully on the `daemon` side.
///
/// # Examples
/// ```no_run
/// # use mpipc::{send_client_command, ClientCommand, DEFAULT_SOCKET_NAME, SocketType, DaemonResponse};
/// match send_client_command(ClientCommand::Shutdown, DEFAULT_SOCKET_NAME, &SocketType::default()) {
///     // The `Ok` variant only means the command was delivered
///     Ok(response) => {
///         match response {
///             DaemonResponse::Ok => eprintln!("Got an `Ok` response, all good!"),
///             // Depending on the type of command sent, the response from the daemon may be different.
///             _ => panic!("Got an unexpected response: {response:?}"),
///         }
///     },
///     Err(error) => panic!("Could not send a command to the daemon: {error}"),
/// }
/// ```
pub fn send_client_command(
    cmd: ClientCommand,
    socket_name: &str,
    socket_type: &SocketType,
) -> Result<DaemonResponse, DaemonError> {
    trace!("Executing daemon command: {cmd:?}");

    let mut conn = match connect_to_daemon(socket_name, socket_type) {
        Ok(conn) => BufReader::new(conn),
        Err(e) => {
            error!("{e}");
            return Err(e);
        }
    };

    let data = serialize_client_command(&cmd)?;

    if let Err(e) = conn.get_mut().write_all(&data) {
        error!("Failed sending the daemon command: {e}");
        return Err(DaemonError::SendingError);
    }

    let response = receive_daemon_response(&mut conn)?;

    if cmd == ClientCommand::Shutdown {
        trace!(
            "Not trying to close the connection to the daemon, it will likely shut down by then"
        );
    } else if let Err(e) = disconnect(&mut conn) {
        error!("Could not close the connection to the daemon: {e}");
    }

    Ok(response)
}
