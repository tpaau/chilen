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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
/// The exit status of the daemon.
pub enum DaemonExitStatus {
    ExitRequested,
}

impl Display for DaemonExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonExitStatus::ExitRequested => {
                write!(
                    f,
                    "Daemon stopped because another process requested it to exit"
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
/// Error related to the daemon.
pub enum DaemonError {
    StoppedUnexpectedly,
    EncodingError { error: String },
    DecodingError { error: String },
    SocketError { error: String },
    ConnectionError { error: String },
    SendingError { error: String },
    MusicLibraryError { error: MusicLibraryError },
    UnknownError,
}

impl Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StoppedUnexpectedly => {
                write!(f, "Daemonm stopped unexpectedly for an unknown reason")
            }
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
            Self::MusicLibraryError { error } => {
                write!(f, "{error}")
            }
            Self::UnknownError => write!(f, "An unknown error occurred in the daemon"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum MusicLibraryError {
    PlaylistExists,
    LibraryNotInitialized,
    NoSuchPlaylist,
    HomeDirNotFound,
    ArcInnerError,
    CacheError,
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
            Self::ArcInnerError => write!(f, "Could not get the underlying arc data"),
            Self::CacheError => write!(f, "Cache is unusable"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
/// Response sent to a client from the daemon.
pub enum DaemonResponse {
    /// The client command was executed successfully.
    Ok,
    /// List of some playlists returned by the daemon.
    Playlists { playlists: Vec<Playlist> },
    /// An internal error occurred.
    Error { error: DaemonError },
}

impl Display for DaemonResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonResponse::Ok => write!(f, "The client command executed successfully"),
            DaemonResponse::Playlists { playlists } => {
                write!(
                    f,
                    "List of some playlists returned by the daemon: {playlists:?}"
                )
            }
            DaemonResponse::Error { error } => write!(f, "An error occured in the daemon: {error}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
/// A parsed CLI command that can either be a daemon start command, or a message to be sent to
/// an already running deamon.
pub enum DaemonCommand {
    /// Start the daemon.
    Start,
    /// Message to be sent to an already running daemon.
    Message { command: ClientCommand },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
/// Command that can be sent to the daemon over a socket.
pub enum ClientCommand {
    Shutdown,
    Restart,
    Status,
    Disconnect,
    Playlist { cmd: PlaylistCommand },
}

impl TryFrom<DaemonCommand> for ClientCommand {
    type Error = String;
    fn try_from(value: DaemonCommand) -> Result<Self, Self::Error> {
        match value {
            DaemonCommand::Start => Err(String::from(
                "The start command is not meant to be sent to the daemon",
            )),
            DaemonCommand::Message { command } => Ok(command),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub enum PlaylistCommand {
    New {
        name: String,
        tracks: Option<Vec<PathBuf>>,
    },
    FromM3U8 {
        name: Option<String>,
        m3u8_file: PathBuf,
    },
    Delete {
        names: Vec<String>,
    },
    List,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct Track {
    pub path: PathBuf,
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
            "{} - {}",
            self.artist.clone().unwrap_or(String::from("Unknown")),
            self.title.clone().unwrap_or(String::from("Unknown")),
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
pub struct Playlist {
    pub name: String,
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
        DaemonResponse::Error { error } => {
            error!("Could not close the connection to the daemon: {error}");
            return Err(error);
        }
        DaemonResponse::Ok => {
            trace!("Connection with the daemon closed.");
        }
        _ => {
            error!("Got an unexpected response from the daemon: {response}");
            return Err(DaemonError::UnknownError);
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
/// # Examples
/// ```no_run
/// # use mpipc::{exec_client_command, ClientCommand};
/// match exec_client_command(ClientCommand::Stop) {
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
