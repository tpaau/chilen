use std::{
    env::temp_dir,
    io::{BufReader, Read, Write},
    path::PathBuf,
    time::Duration,
};

use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, Name, Stream, ToNsName, prelude::*,
};
use log::info;
use log::{error, trace};
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};

/// The default name of the socket the daemon listens on.
pub const DEFAULT_SOCKET_NAME: &str = "DEFAULT_MUSIC_PLAYER.socket";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// An error originating from the music library module of the daemon.
pub enum MusicLibraryError {
    /// Could not complete the operation because a playlist with this name already exists.
    PlaylistExists,
    /// Could not perform the operation because the music library is not initialized.
    ///
    /// This either means that the command was sent too early, or that the music library is
    /// currently being rebuilt.
    LibraryNotInitialized,
    /// Could not perform an operation on a nonexistent playlist.
    NoSuchPlaylist,
    /// Could not get the path to the cache or the cache is unusable.
    CacheError,
    /// The provided item index was out of bounds.
    ///
    /// Eg. there are 68 tracks in the playlist, so the maximum index is 67, but the index
    /// 69 was provided.
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Error related to the playback module.
pub enum PlaybackError {
    /// The audio player is not connected.
    PlayerNotConnected,
    /// The queue is empty.
    QueueEmpty,
    /// Could not open an audio source.
    SourceError,
    /// The audio file could not be opened, has an unsupported format or is corrupt.
    PlayerPlaying,
    /// The player is already paused.
    PlayerPaused,
    /// The player has already stopped.
    PlayerStoppped,
    /// Seek is not supported.
    SeekNotSupported,
    /// Cannot go to the previous track.
    CannotGoPrevious,
    /// Cannot go to the next track.
    CannotGoNext,
    /// The daemon was not built with shuffle support.
    ShuffleNotSupported,
    /// No track at this index.
    NoTrackAtIndex(usize),
    /// The specified rate value was out of the allowed range.
    RateOutOfRange,
    /// The modification of the playback rate is disallowed.
    FixedRate,
}

impl std::fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayerNotConnected => write!(f, "The audio player is not connected"),
            Self::QueueEmpty => write!(f, "The queue is empty"),
            Self::SourceError => write!(
                f,
                "The audio file could not be opened, has an unsupported format or is corrupt"
            ),
            Self::PlayerPlaying => write!(f, "The player is already playing"),
            Self::PlayerPaused => write!(f, "The player is already paused"),
            Self::PlayerStoppped => write!(f, "The player has already stopped"),
            Self::SeekNotSupported => write!(f, "Seek is not supported"),
            Self::CannotGoPrevious => write!(f, "Cannot go to the previous track"),
            Self::CannotGoNext => write!(f, "Cannot go to the next track"),
            Self::ShuffleNotSupported => write!(f, "The daemon was not built with shuffle support"),
            Self::NoTrackAtIndex(index) => write!(f, "No track was found at index {index}"),
            Self::RateOutOfRange => {
                write!(f, "The specified rate value was out of the allowed range")
            }
            Self::FixedRate => write!(f, "The modification of the playback rate is disallowed"),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Struct representing a track from the music library.
pub struct Track {
    /// The path to the audio file.
    pub path: PathBuf,
    /// The path to the extracted cover file, if exists.
    pub cover_path: Option<PathBuf>,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub lyrics: Option<String>,
    pub comment: Option<String>,
    pub track: Option<u32>,
    pub track_total: Option<u32>,
    pub disk: Option<u32>,
    pub disk_total: Option<u32>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Struct representing a playlist from the music library.
pub struct Playlist {
    /// The name of the playlist. Playlist names are unique.
    pub name: String,
    /// The tracks added to the playlist.
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Struct representing the contents of the music library.
pub struct MusicLibrary {
    /// The list of playlists in the music library.
    pub playlists: Vec<Playlist>,
    /// The list of all tracks in the music library.
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Event originating from the playback module of the daemon.
pub enum PlaybackEvent {
    /// Emitted when the playback state changes, for instance when the player is paused.
    PlaybackStateChanged(PlaybackState),
    /// Emitted when the loop state of the player changes.
    LoopStateChanged(LoopState),
    /// Emitted when the shuffle state of the player changes.
    ShuffleStateChanged(ShuffleState),
    /// Emitted when the current queue changes.
    QueueChanged(Vec<Track>),
    /// Emitted when the current track changes.
    PositionChanged(usize),
    /// Emitted when the position of the player changes.
    PlayerPositionChanged(Duration),
    /// Emitted when the volume of the player changes.
    PlayerVolumeChanged(PlayerVolume),
    /// Emitted when the playback rate of the player changes.
    RateChanged(PlaybackRate),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Event from the daemon sent to clients.
pub enum DaemonEvent {
    /// Event sent before the daemon stops.
    Shutdown,
    /// Event sent after the contents of the music library have changed.
    MusicLibraryChanged(MusicLibrary),
    /// Event sent when a client disconnects from the daemon.
    ConnectionClosed,
    /// Event originating from the playback module of the daemon.
    PlaybackEvent(PlaybackEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Error that can occur while creating a daemon configuration.
pub enum ConfigError {
    /// The bus name suffix provided was invalid.
    ///
    /// The suffix must only contain ASCII characters.
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
/// Error related to the daemon.
///
/// Can either originate from the daemon itself or while connecting to the daemon.
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
    /// Could not parse the command sent to the daemon.
    ParsingError,
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
    /// Got an unexpected response from the daemon.
    UnexpectedResponse,
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncodingError => write!(f, "Could not encode the deamon conmmand"),
            Self::DecodingError => write!(f, "Could not decode the response from the daemon"),
            Self::SocketError => write!(f, "Could not obtain the daemon socket"),
            Self::ConnectionError => write!(f, "Could not connect to the daemon"),
            Self::SendingError => write!(f, "Could not send the command to the daemon"),
            Self::ParsingError => write!(f, "Could not parse the command sent to the daemon."),
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
            Self::UnexpectedResponse => write!(f, "Got an unexpected response from the daemon"),
        }
    }
}

/// Response sent to a client from the daemon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DaemonResponse {
    /// The client command was executed successfully.
    Ok,
    /// Response to the `Ping` client command.
    Pong,
    /// List of some playlists returned by the daemon.
    Library(MusicLibrary),
    /// An event from the daemon.
    Event(DaemonEvent),
    /// A response from the playback command.
    Playback(PlaybackResponse),
    /// An internal error occurred.
    Error(DaemonError),
}

/// Signed duration type used for seeking.
///
/// This is just a bare bones enum. Do not use it as a `Duration` replacement outside of
/// controlling the playback.
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackCommand {
    /// Play the current track or play a track at a specific index in the queue.
    Play(Option<usize>),
    /// Pause the player.
    Pause,
    /// Stop the player.
    Stop,
    /// Toggle between play/pause.
    TogglePlaying,
    /// Get the playback state (Playing, Paused, Stopped) of the player.
    GetPlaybackState,
    /// Set a new queue for the player.
    SetQueue(Vec<PathBuf>),
    /// Append tracks to the queue.
    AppendToQueue(Vec<PathBuf>),
    /// Load a playlist and append its tracks to the queue.
    AppendPlaylist(String),
    /// Load a playlist and put its tracks in the queue.
    SetPlaylist(String),
    /// Get the current track.
    GetCurrentTrack,
    /// Skip to the next track.
    Next,
    /// Skip to the previous track.
    Previous,
    /// Set the loop state of the player.
    SetLoopState(LoopState),
    /// Get the loop state of the player.
    GetLoopState,
    /// Set the playback rate of the player.
    ///
    /// This command will fail if the daemon is configured to not allow playback rate modification
    /// or if the specified rate value is out of the acceptable range.
    SetRate(f64),
    /// Get the playback rate of the player.
    GetRate,
    /// Set the shuffle state of the player.
    ///
    /// This command will fail if the daemon was not built with shuffle support.
    SetShuffleState(ShuffleState),
    /// Get the shuffle state of the player.
    ///
    /// The daemon will always respond with `ShuffleState::Off` if it wasn't built shuffle support.
    GetShuffleState,
    /// Set the position of the player.
    SetPlayerPosition(Duration),
    /// Change the player position by a time delta.
    Seek(SignedDuration),
    /// Get the position of the player.
    GetPlayerPosition,
    /// Set the volume of the player.
    SetPlayerVolume(PlayerVolume),
    /// Get the volume of the player.
    GetPlayerVolume,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackResponse {
    Ok,
    PlaybackState(PlaybackState),
    LoopState(LoopState),
    PlaybackRate(PlaybackRate),
    ShuffleState(ShuffleState),
    Track(Box<Option<Track>>),
    PlayerVolume(PlayerVolume),
    PlayerPosition(Duration),
}

impl std::fmt::Display for PlaybackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "Ok"),
            Self::PlaybackState(state) => write!(f, "{state}"),
            Self::LoopState(state) => write!(f, "{state}"),
            Self::PlaybackRate(rate) => write!(f, "{rate}"),
            Self::ShuffleState(state) => write!(f, "{state}"),
            Self::Track(track) => match &**track {
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
/// Setting rate to 1.0 will play audio files at their original speed, 2.0 will speed them up two
/// times, etc.
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
    /// Create a new [`PlaybackRate`](mpipc::PlaybackRate) object.
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
    pub fn new(volume: f64) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
        }
    }

    pub fn set(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn get(&self) -> f64 {
        self.volume
    }
}

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoopState {
    /// The playback will stop when there are no more tracks to play.
    #[default]
    Off,
    /// The current track will start again from the beginning once it has finished playing.
    Track,
    /// The playback will loop through a list of tracks.
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Command for managing the music library, can be sent to a deamon instance by wrapping it in a
/// `ClientCommand`.
pub enum PlaylistCommand {
    /// Create a new playlist, optionally with some tracks in it.
    New {
        /// The name for the new playlist, must not already exist in the music library.
        name: String,
        /// The optional list of tracks that will be added to the playlist.
        ///
        /// Tracks that are outside of the music library (usually the `~/Music/` directory)
        /// will be ignored.
        tracks: Option<Vec<PathBuf>>,
    },
    /// Import a playlist from an M3U8 file.
    FromM3U8 {
        /// The name for the imported playlist, must not already exist in the music library.
        ///
        /// If left unspecified, it will be derived from the name of the imported file.
        name: Option<String>,
        /// The path to the M3U8 file to import.
        m3u8_file: PathBuf,
    },
    /// Delete playlists from the music library.
    Delete {
        /// List of the playlists to delete.
        names: Vec<String>,
    },
    /// Add tracks to an already existing playlist.
    AddTracks {
        /// The name of the playlist to operate on.
        name: String,
        /// The list of tracks to add.
        tracks: Vec<PathBuf>,
    },
    /// Remove tracks from an already existing playlist.
    RemoveTracks {
        /// The name of the playlist to operate on.
        name: String,
        /// The list of IDs of tracks to remove.
        ids: Vec<usize>,
    },
    /// Get a list of the playlists from the music library.
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Manage the cache.
pub enum CacheCommand {
    /// Rebuild the cache. May resolve some issues with badly extracted covers.
    Rebuild,
    /// Reinitialize the music library to find newly added tracks.
    Reload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Command that can be executed by a daemon instance.
pub enum ClientCommand {
    /// Stop the daemon instance.
    Shutdown,
    /// Manage the music library.
    Playlist(PlaylistCommand),
    /// Manage the music library cache.
    Cache(CacheCommand),
    /// Stream events from the daemon. Causes the thread to stop accepting requests.
    EventStream,
    /// Close the connection to the daemon.
    Disconnect,
    /// Command to the playback module.
    Playback(PlaybackCommand),
    /// Ping the daemon.
    Ping,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocketType {
    NamespacedOnly,
    #[default]
    NamespacedOrFilesystem,
    FilesystemOnly,
}

impl TryFrom<DaemonCommand> for ClientCommand {
    type Error = String;
    fn try_from(value: DaemonCommand) -> Result<Self, Self::Error> {
        match value {
            DaemonCommand::Start => Err(String::from(
                "This command is not meant to be sent to the daemon",
            )),
            DaemonCommand::ClientCommand(command) => Ok(command),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// A parsed CLI command that can either be a daemon start command, or a message to be sent to
/// an already running deamon instance.
pub enum DaemonCommand {
    /// Start the daemon.
    Start,
    /// Command to be sent to an already running daemon instance.
    ClientCommand(ClientCommand),
}

pub fn get_fs_socket_path(socket_name: &str) -> PathBuf {
    let mut temp_dir = temp_dir();
    temp_dir.push(socket_name);
    temp_dir
}

/// Returns a namespaced socket for the given socket name.
///
/// # Examples
/// ```
/// let socket = mpipc::get_ns_daemon_socket(mpipc::DEFAULT_SOCKET_NAME).unwrap();
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

/// Returns a namespaced socket for the given socket name.
///
/// # Examples
/// ```
/// let socket = mpipc::get_fs_daemon_socket(mpipc::DEFAULT_SOCKET_NAME).unwrap();
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

/// Attempts to get a socket address for daemon IPC.
///
/// If the library is built with namespaced socket support, additional configuration options can be
/// used to specify whether a namespaced or a filesystem socket should be used.
///
/// # Examples
/// ```
/// match mpipc::get_daemon_socket(
///     mpipc::DEFAULT_SOCKET_NAME,
///     &mpipc::SocketType::NamespacedOrFilesystem
/// ) {
///     Ok(socket) => eprintln!("Got a socket: {socket:?}"),
///     Err(e) => panic!("Could not obtain a socket: {e}"),
/// }
/// ```
pub fn get_daemon_socket<'a>(
    socket_name: &'a str,
    mode: &SocketType,
) -> Result<Name<'a>, std::io::Error> {
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
/// ```no_run
/// // Connect to the daemon and immediately disconnect.
/// # use std::io::{BufReader, Write};
/// let mut conn = BufReader::new(mpipc::connect_to_daemon(mpipc::DEFAULT_SOCKET_NAME, &mpipc::SocketType::default()).unwrap());
/// let cmd = mpipc::serialize_client_command(mpipc::ClientCommand::Disconnect).unwrap();
/// conn.get_mut().write_all(&cmd).unwrap();
/// ```
pub fn serialize_client_command(cmd: ClientCommand) -> Result<Vec<u8>, DaemonError> {
    let mut data = Vec::new();
    if let Err(e) = cmd.serialize(&mut rmp_serde::Serializer::new(&mut data)) {
        error!("Could not encode the client command: {e}");
        return Err(DaemonError::EncodingError);
    }
    Ok(data)
}

/// Receive a daemon response from a buffered stream connection.
///
/// This function will block until a response is received or the connection ends.
///
/// # Examples
/// ```no_run
/// # use std::io::{BufReader, Write};
/// let mut conn = BufReader::new(mpipc::connect_to_daemon(mpipc::DEFAULT_SOCKET_NAME, &mpipc::SocketType::default()).unwrap());
/// let cmd = mpipc::serialize_client_command(mpipc::ClientCommand::EventStream).unwrap();
/// conn.get_mut().write_all(&cmd).unwrap();
/// loop {
///     let response = mpipc::receive_daemon_response(&mut conn).unwrap();
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

/// Disconnect from the daemon (send the disconnect client command).
///
/// # Examples
/// ```no_run
/// # use std::io::BufReader;
/// let conn = mpipc::connect_to_daemon(mpipc::DEFAULT_SOCKET_NAME, &mpipc::SocketType::default()).unwrap();
/// let mut conn = BufReader::new(conn);
/// // Do some stuff with the connection here...
/// match mpipc::disconnect(&mut conn) {
///     Ok(_) => eprintln!("Disconnected from the deamon!"),
///     Err(e) => panic!("Could not close the daemon connection: {e}"),
/// }
/// ```
pub fn disconnect(conn: &mut BufReader<Stream>) -> Result<(), DaemonError> {
    trace!("Closing connection with the daemon");

    let mut data = Vec::new();
    if let Err(e) = ClientCommand::Disconnect.serialize(&mut Serializer::new(&mut data)) {
        error!("Failed encoding the client command: {e}");
        return Err(DaemonError::EncodingError);
    }

    match conn.get_mut().write_all(&data) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed sending the command to the daemon: {e}");
            return Err(DaemonError::SendingError);
        }
    }

    let mut data = Vec::new();
    if let Err(e) = conn.get_ref().read_to_end(&mut data) {
        error!("Failed connecting to the daemon: {e}");
        return Err(DaemonError::ConnectionError);
    }

    let response: DaemonResponse = match rmp_serde::from_read(&mut data.as_slice()) {
        Ok(response) => response,
        Err(e) => {
            error!("Failed decoding a daemon response: {e}");
            return Err(DaemonError::DecodingError);
        }
    };

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

/// Connects to the daemon via a local socket and returns the stream connection.
///
/// # Examples
/// ```no_run
/// match mpipc::connect_to_daemon(mpipc::DEFAULT_SOCKET_NAME, &mpipc::SocketType::default()) {
///     Ok(stream) => eprintln!("Connected to the daemon: {stream:?}"),
///     Err(error) => panic!("Could not connect to the daemon: {error}"),
/// }
/// ```
pub fn connect_to_daemon(
    socket_name: &str,
    socket_type: &SocketType,
) -> Result<Stream, DaemonError> {
    trace!("Connecting to daemon on socket '{socket_name}'");

    let socket = match get_daemon_socket(socket_name, socket_type) {
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

/// Executes a single daemon command and ends the connection.
///
/// > [!WARNING]
/// > The `Ok` variant only means that the command was sent and received, and that the connection
/// > was successfully closed. It does not mean that it was properly executed by the daemon!
///
/// # Examples
/// ```no_run
/// match mpipc::exec_client_command(mpipc::ClientCommand::Shutdown, mpipc::DEFAULT_SOCKET_NAME, &mpipc::SocketType::default()) {
///     Ok(response) => eprintln!("Got a response from the daemon: {response:?}"),
///     Err(error) => panic!("Could not send a command to the daemon: {error}"),
/// }
/// ```
pub fn exec_client_command(
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

    let mut data = Vec::new();
    if let Err(e) = cmd.serialize(&mut Serializer::new(&mut data)) {
        error!("Failed encoding the client command: {e}");
        return Err(DaemonError::EncodingError);
    }

    if let Err(e) = conn.get_mut().write_all(&data) {
        error!("Failed sending the daemon command: {e}");
        return Err(DaemonError::SendingError);
    }

    let response: DaemonResponse = match rmp_serde::from_read(&mut conn) {
        Ok(response) => response,
        Err(e) => {
            error!("Failed decoding a daemon response: {e}");
            return Err(DaemonError::DecodingError);
        }
    };

    if cmd == ClientCommand::Shutdown {
        trace!("Not trying to close the connection to the daemon");
    } else if let Err(e) = disconnect(&mut conn) {
        error!("Could not close the connection to the daemon: {e}");
    }

    Ok(response)
}
