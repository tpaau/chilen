use std::{
    io::{BufReader, Read, Write},
    path::PathBuf,
    time::Duration,
};

use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, Name, Stream, ToNsName, prelude::*,
};
use lofty::tag::{Accessor, ItemValue, Tag};
use log::{error, trace, warn};
use rmp_serde::{Serializer, from_read};
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
    /// Seek is not supported.
    SeekNotSupported,
    /// Cannot go to the previous track.
    CannotGoPrevious,
    /// Cannot go to the next track.
    CannotGoNext,
    /// The daemon was not built with shuffle support.
    ShuffleNotSupported,
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
            Self::SeekNotSupported => write!(f, "Seek is not supported"),
            Self::CannotGoPrevious => write!(f, "Cannot go to the previous track"),
            Self::CannotGoNext => write!(f, "Cannot go to the next track"),
            Self::ShuffleNotSupported => write!(f, "The daemon was not built with shuffle support"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataError {
    PermissionError,
    CacheDirNotAvailable,
    DataDirNotAvailable,
    NoMusicLibrary,
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

impl From<&Tag> for Track {
    fn from(tag: &Tag) -> Self {
        let lyrics = match tag.get(&lofty::tag::ItemKey::Lyrics) {
            Some(tag_item) => match tag_item.value() {
                ItemValue::Text(lyrics) => Some(lyrics.clone()),
                _ => None,
            },
            None => None,
        };

        Track {
            path: PathBuf::new(),
            cover_path: None,
            artist: tag.artist().map(|artist| artist.into()),
            title: tag.title().map(|title| title.into()),
            album: tag.album().map(|album| album.into()),
            genre: tag.genre().map(|genre| genre.into()),
            lyrics,
            comment: tag.comment().map(|comment| comment.into()),
            track: tag.track(),
            track_total: tag.track_total(),
            disk: tag.disk(),
            disk_total: tag.disk_total(),
            year: tag.year(),
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Event from the daemon sent to clients.
pub enum DaemonEvent {
    /// Event sent before the daemon stops.
    Shutdown,
    /// Event sent before the daemon restarts.
    Restart,
    /// Event sent after the content of the music library changed.
    MusicLibraryChanged(MusicLibrary),
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
    /// Could not initialize the listener in the daemon.
    ListenerError,
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
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncodingError => write!(f, "Could not encode the deamon conmmand"),
            Self::DecodingError => write!(f, "Could not decode the response from the daemon"),
            Self::SocketError => write!(f, "Could not obtain the daemon socket"),
            Self::ConnectionError => write!(f, "Could not connect to the daemon"),
            Self::ListenerError => write!(f, "Could not initialize the listener in the daemon"),
            Self::SendingError => write!(f, "Could not send the commadnd to the daemon"),
            Self::ParsingError => write!(f, "Could not parse the command sent to the daemon."),
            Self::MusicLibraryError(e) => {
                write!(f, "{e}")
            }
            Self::DataError(e) => write!(f, "Data error: {e}"),
            Self::InvalidResponse => {
                write!(f, "The response from the daemon was invalid or malformed")
            }
            Self::PlaybackError(e) => write!(f, "Playback error: {e}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Response sent to a client from the daemon.
pub enum DaemonResponse {
    /// The client command was executed successfully.
    Ok,
    /// List of some playlists returned by the daemon.
    Library(MusicLibrary),
    /// An event from the daemon.
    Event(DaemonEvent),
    /// An internal error occurred.
    Error(DaemonError),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaybackCommand {
    Play,
    Pause,
    SetQueue(Vec<PathBuf>),
    AppendToQueue(Vec<PathBuf>),
    SetPlaylist(String),
    Next,
    Previous,
    SetLoopState(LoopState),
    SetShuffleState(ShuffleState),
    SetPosition(Duration),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaybackEvent {
    Test,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaybackState {
    Playing,
    Paused,
    #[default]
    Stopped,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShuffleState {
    On,
    #[default]
    Off,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoopState {
    #[default]
    Off,
    Track,
    Playlist,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Command that can be executed by a daemon instance.
pub enum ClientCommand {
    /// Stop the daemon instance.
    Shutdown,
    /// Restart the daemon instance.
    Restart,
    /// Manage the music library.
    Playlist(PlaylistCommand),
    /// Manage the music library cache.
    Cache(CacheCommand),
    /// Stream events from the daemon. Causes the thread to stop accepting requests.
    EventStream,
    /// Close the connection to the daemon.
    Disconnect,
    // Command to the playback module.
    Playback(PlaybackCommand),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A parsed CLI command that can either be a daemon start command, or a message to be sent to
/// an already running deamon instance.
pub enum DaemonCommand {
    /// Start the daemon.
    Start,
    /// Command to be sent to an already running daemon instance.
    ClientCommand(ClientCommand),
}

/// Try to get a namespaced socket or a filesystem socket for daemon IPC.
///
/// # Examples
/// ```
/// # use mpipc::get_daemon_socket;
/// let socket_name = mpipc::DEFAULT_SOCKET_NAME;
/// match get_daemon_socket(socket_name) {
///     Ok(socket) => eprintln!("Got a socket: {socket:?}"),
///     Err(e) => panic!("Could not obtain a socket: {e}"),
/// }
/// ```
pub fn get_daemon_socket<'a>(socket_name: &'a str) -> Result<Name<'a>, std::io::Error> {
    trace!("Obtaining a namespaced socket for '{socket_name}'");

    match socket_name.to_ns_name::<GenericNamespaced>() {
        Ok(socket) => Ok(socket),
        Err(e) => {
            warn!("Could not obtain a namespaced socket (is your system supported?): {e}");
            match socket_name.to_fs_name::<GenericFilePath>() {
                Ok(socket) => Ok(socket),
                Err(e) => {
                    error!("Could not obtain both a namespaced and a filesystem socket: {e}");
                    Err(e)
                }
            }
        }
    }
}

/// Serialize a client command to a format that can be sent to the daemon.
///
/// # Examples
/// ```no_run
/// // Connect to the daemon and immediately disconnect.
/// # use std::io::{BufReader, Write};
/// # use mpipc;
/// let mut conn = BufReader::new(mpipc::connect_to_daemon(mpipc::DEFAULT_SOCKET_NAME).unwrap());
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
/// # use mpipc;
/// let mut conn = BufReader::new(mpipc::connect_to_daemon(mpipc::DEFAULT_SOCKET_NAME).unwrap());
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
    match from_read(conn) {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Failed decoding a daemon response: {e}");
            Err(DaemonError::EncodingError)
        }
    }
}

/// Disconnect from the daemon (send the disconnect client command).
///
/// # Examples
/// ```no_run
/// # use std::io::BufReader;
/// # use mpipc::{connect_to_daemon, disconnect};
/// let socket_name = mpipc::DEFAULT_SOCKET_NAME;
/// let conn = connect_to_daemon(socket_name).unwrap();
/// let mut conn = BufReader::new(conn);
/// // Do some stuff with the connection here...
/// match disconnect(&mut conn) {
///     Ok(_) => eprintln!("Disconnected from the deamon!"),
///     Err(e) => panic!("Could not terminate the daemon connection: {e}"),
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

    let response: DaemonResponse = match from_read(conn) {
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
            trace!("Connection with the daemon closed.");
        }
        _ => {
            error!("Got an unexpected response from the daemon: {response:?}");
            return Err(DaemonError::InvalidResponse);
        }
    }

    Ok(())
}

/// Connects to the daemon via a local socket and returns the stream connection.
///
/// # Examples
/// ```no_run
/// # use mpipc::connect_to_daemon;
/// let socket_name = mpipc::DEFAULT_SOCKET_NAME;
/// match connect_to_daemon(socket_name) {
///     Ok(stream) => eprintln!("Connected to the daemon: {stream:?}"),
///     Err(error) => panic!("Could not connect to the daemon: {error}"),
/// }
/// ```
pub fn connect_to_daemon(socket_name: &str) -> Result<Stream, DaemonError> {
    trace!("Connecting to daemon on socket '{socket_name}'");

    let socket = match get_daemon_socket(socket_name) {
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
/// # use mpipc::{exec_client_command, ClientCommand};
/// let socket_name = mpipc::DEFAULT_SOCKET_NAME;
/// match exec_client_command(ClientCommand::Shutdown, socket_name) {
///     Ok(response) => eprintln!("Got a response from the daemon: {response:?}"),
///     Err(error) => panic!("Could not send a command to the daemon: {error}"),
/// }
/// ```
pub fn exec_client_command(
    cmd: ClientCommand,
    socket_name: &str,
) -> Result<DaemonResponse, DaemonError> {
    trace!("Executing daemon command: {cmd:?}");

    let mut conn = match connect_to_daemon(socket_name) {
        Ok(conn) => BufReader::new(conn),
        Err(e) => {
            error!("Could not execute daemon command: {e}");
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

    let mut data = Vec::new();
    if let Err(e) = conn.get_ref().read(&mut data) {
        error!("Failed connecting to the daemon: {e}");
        return Err(DaemonError::ConnectionError);
    }

    let response: DaemonResponse = match from_read(&mut conn) {
        Ok(response) => response,
        Err(e) => {
            error!("Failed decoding a daemon response: {e}");
            return Err(DaemonError::DecodingError);
        }
    };

    if let Err(e) = disconnect(&mut conn) {
        error!("Could not close the connection to the daemon: {e}");
    }

    Ok(response)
}
