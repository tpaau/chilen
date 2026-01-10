use std::{
    fmt::Display,
    io::{BufReader, Write},
    path::PathBuf,
};

use bincode::{Decode, Encode, config::standard, decode_from_reader, encode_to_vec};
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, Name, Stream, ToNsName, prelude::*,
};
use lofty::tag::{Accessor, Tag};
use log::{error, trace, warn};

/// The name of the socket the daemon listens on.
pub const SOCKET_NAME: &str = "MUSIC_PLAYER.socket";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
/// Error related to the daemon.
///
/// Can either originate from the daemon itself or while connecting to the daemon.
pub enum DaemonError {
    /// The value could not be encoded before sending it.
    EncodingError { error: String },
    /// The encoded value could not be decoded back into usable form.
    DecodingError { error: String },
    /// Could not obtain the daemon socket address.
    SocketError { error: String },
    /// Could not connect to the daemon.
    ConnectionError { error: String },
    /// Could not send the command to the daemon.
    SendingError { error: String },
    /// An error occurred in the music library module.
    MusicLibraryError(MusicLibraryError),
    /// The response received from the daemon was unexpected or invalid.
    InvalidResponse,
}

impl Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncodingError { error } => {
                write!(f, "Failed encoding daemon command: {error}")
            }
            Self::DecodingError { error } => {
                write!(f, "Could not decode the response from the daemon: {error}")
            }
            Self::SocketError { error } => {
                write!(f, "Socket error: {error}")
            }
            Self::ConnectionError { error } => {
                write!(f, "Could not connect to the daemon: {error}")
            }
            Self::SendingError { error } => {
                write!(f, "Could not send the commadnd to the daemon: {error}")
            }
            Self::MusicLibraryError(e) => {
                write!(f, "{e}")
            }
            Self::InvalidResponse => {
                write!(f, "The response from the daemon was invalid or malformed")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
/// An error originating from the music library module of the daemon.
pub enum MusicLibraryError {
    /// Cannot complete the operation because a playlist with this name already exists.
    PlaylistExists,
    /// The operation could not be completed because the music library is not initialized.
    ///
    /// This either means that the command was sent too early, or that the music library is
    /// currently being rebuilt.
    LibraryNotInitialized,
    /// Cannot perform an operation on a nonexistent playlist.
    NoSuchPlaylist,
    /// Could not get the path to the home directory, or the home directory does not exist.
    HomeDirNotFound,
    /// Could not get the path to the cache or the cache is unusable.
    CacheError,
    IndexOutOutBounds,
}

impl Display for MusicLibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaylistExists => write!(f, "Playlist with this name already exists"),
            Self::LibraryNotInitialized => write!(f, "The music library is not yet initialized"),
            Self::NoSuchPlaylist => write!(f, "There is no playlist with this name"),
            Self::HomeDirNotFound => write!(
                f,
                "Could not get the path to the home directory, or the home directory does not exist"
            ),
            Self::CacheError => write!(f, "Cache is unusable"),
            Self::IndexOutOutBounds => write!(f, "The provided item index was out of bounds"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
/// Response sent to a client from the daemon.
pub enum DaemonResponse {
    /// The client command was executed successfully.
    Ok,
    /// List of some playlists returned by the daemon.
    Playlists(Vec<Playlist>),
    /// An internal error occurred.
    Error(DaemonError),
}

impl Display for DaemonResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonResponse::Ok => write!(f, "The client command executed successfully"),
            DaemonResponse::Playlists(playlists) => {
                write!(
                    f,
                    "List of some playlists returned by the daemon: {playlists:?}"
                )
            }
            DaemonResponse::Error(error) => write!(f, "An error occurred in the daemon: {error}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
/// A parsed CLI command that can either be a daemon start command, or a message to be sent to
/// an already running deamon instance.
pub enum DaemonCommand {
    /// Start the daemon.
    Start,
    /// Command to be sent to an already running daemon instance.
    ClientCommand(ClientCommand),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
/// Manage the cache.
pub enum CacheCommand {
    /// Rebuild the cache. May resolve some issues with badly extracted covers.
    Rebuild,
    /// Reinitialize the music library to find newly added tracks.
    Reload,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
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
    /// Close the connection to the daemon.
    Disconnect,
}

impl TryFrom<DaemonCommand> for ClientCommand {
    type Error = String;
    fn try_from(value: DaemonCommand) -> Result<Self, Self::Error> {
        match value {
            DaemonCommand::Start => Err(String::from(
                "The start command is not meant to be sent to the daemon",
            )),
            DaemonCommand::ClientCommand(command) => Ok(command),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
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
            "{} - {} ({:?})",
            self.artist.clone().unwrap_or(String::from("Unknown")),
            self.title.clone().unwrap_or(String::from("Unknown")),
            self.path
        )
    }
}

impl From<&Tag> for Track {
    fn from(tag: &Tag) -> Self {
        Track {
            path: PathBuf::new(),
            cover_path: None,
            artist: tag.artist().map(|artist| artist.into()),
            title: tag.title().map(|title| title.into()),
            album: tag.album().map(|album| album.into()),
            genre: tag.genre().map(|genre| genre.into()),
            comment: tag.comment().map(|comment| comment.into()),
            track: tag.track(),
            track_total: tag.track_total(),
            disk: tag.disk(),
            disk_total: tag.disk_total(),
            year: tag.year(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
/// Struct representing a playlist from the music library.
pub struct Playlist {
    /// The name of the playlist. Playlist names are unique.
    pub name: String,
    /// The tracks added to the playlist.
    pub tracks: Vec<Track>,
}

/// Try to get a namespaced socket or a filesystem socket for daemon IPC.
///
/// # Examples
/// ```
/// # use mpipc::get_daemon_socket;
/// match get_daemon_socket() {
///     Ok(socket) => eprintln!("Got a socket: {socket:?}"),
///     Err(e) => panic!("Could not obtain a socket: {e}"),
/// }
/// ```
pub fn get_daemon_socket<'a>() -> Result<Name<'a>, std::io::Error> {
    trace!("Obtaining a namespaced socket for '{SOCKET_NAME}'");

    match SOCKET_NAME.to_ns_name::<GenericNamespaced>() {
        Ok(socket) => Ok(socket),
        Err(e) => {
            warn!("Could not obtain a namespaced socket (is your system supported?): {e}");
            match SOCKET_NAME.to_fs_name::<GenericFilePath>() {
                Ok(socket) => Ok(socket),
                Err(e) => {
                    error!("Could not obtain both a namespaced and a filesystem socket: {e}");
                    Err(e)
                }
            }
        }
    }
}

/// Disconnect from the daemon (send the disconnect client command).
///
/// # Examples
/// ```no_run
/// # use std::io::BufReader;
/// # use mpipc::{connect_to_daemon, disconnect};
/// let conn = connect_to_daemon().unwrap();
/// let mut conn = BufReader::new(&conn);
/// // Do some stuff with the connection here...
/// match disconnect(&mut conn) {
///     Ok(_) => eprintln!("Disconnected from the deamon!"),
///     Err(e) => panic!("Could not terminate the daemon connection: {e}"),
/// }
/// ```
pub fn disconnect(conn: &mut BufReader<&Stream>) -> Result<(), DaemonError> {
    trace!("Closing connection with the daemon");

    let cmd = match encode_to_vec(ClientCommand::Disconnect, standard()) {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(DaemonError::EncodingError {
                error: e.to_string(),
            });
        }
    };

    match conn.get_mut().write_all(&cmd) {
        Ok(_) => {}
        Err(e) => {
            return Err(DaemonError::SendingError {
                error: e.to_string(),
            });
        }
    }

    let response: DaemonResponse = match decode_from_reader(conn, standard()) {
        Ok(response) => response,
        Err(e) => {
            return Err(DaemonError::DecodingError {
                error: e.to_string(),
            });
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
            error!("Got an unexpected response from the daemon: {response}");
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
/// match connect_to_daemon() {
///     Ok(stream) => eprintln!("Connected to the daemon: {stream:?}"),
///     Err(error) => panic!("Could not connect to the daemon: {error}"),
/// }
/// ```
pub fn connect_to_daemon() -> Result<Stream, DaemonError> {
    trace!("Connecting to daemon on socket '{SOCKET_NAME}'");

    let socket = match get_daemon_socket() {
        Ok(sock) => sock,
        Err(e) => {
            error!("Could not obtain a socket: {e}");
            return Err(DaemonError::SocketError {
                error: e.to_string(),
            });
        }
    };

    let conn = match Stream::connect(socket) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Could not initialize a connection to the daemon: {e}");
            return Err(DaemonError::ConnectionError {
                error: e.to_string(),
            });
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
/// match exec_client_command(ClientCommand::Shutdown) {
///     Ok(response) => eprintln!("Got a response from the daemon: {response}"),
///     Err(error) => panic!("Could not send a command to the daemon: {error}"),
/// }
/// ```
pub fn exec_client_command(cmd: ClientCommand) -> Result<DaemonResponse, DaemonError> {
    trace!("Executing daemon command: {cmd:?}");

    let conn = match connect_to_daemon() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Could not execute daemon command: {e:?}");
            return Err(e);
        }
    };

    let cmd = match encode_to_vec(cmd, standard()) {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(DaemonError::EncodingError {
                error: e.to_string(),
            });
        }
    };

    let mut buf = BufReader::new(&conn);

    match buf.get_mut().write_all(&cmd) {
        Ok(_) => {}
        Err(e) => {
            return Err(DaemonError::SendingError {
                error: e.to_string(),
            });
        }
    }

    let response = match decode_from_reader(&mut buf, standard()) {
        Ok(response) => response,
        Err(e) => {
            return Err(DaemonError::DecodingError {
                error: e.to_string(),
            });
        }
    };

    if let Err(e) = disconnect(&mut buf) {
        error!("Could not close the connection to the daemon: {e}");
    }

    Ok(response)
}
