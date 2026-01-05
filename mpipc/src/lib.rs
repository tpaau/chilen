use std::{
    fmt::Display,
    io::{BufReader, Write},
    path::PathBuf,
};

use bincode::{Decode, Encode, config::standard, decode_from_reader, encode_to_vec};
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, Name, Stream, ToNsName, prelude::*,
};
use log::{error, trace, warn};
use serde::{Deserialize, Serialize};

/// The name of the socket the daemon listens on.
pub const SOCKET_NAME: &str = "MUSIC_PLAYER.socket";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Decode)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Error related to the daemon.
pub enum DaemonError {
    StoppedUnexpectedly,
    EncodingError { error: String },
    DecodingError { error: String },
    SocketError { error: String },
    ConnectionError { error: String },
    SendingError { error: String },
    MusicLibraryError { error: MusicLibraryError },
}

impl Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonError::StoppedUnexpectedly => {
                write!(f, "Daemonm stopped unexpectedly for an unknown reason")
            }
            DaemonError::EncodingError { error } => {
                write!(f, "Failed encoding daemon command: {error}")
            }
            DaemonError::DecodingError { error } => {
                write!(f, "Could not decode the response from the daemon: {error}")
            }
            DaemonError::SocketError { error } => {
                write!(f, "Socket error: {error}")
            }
            DaemonError::ConnectionError { error } => {
                write!(f, "Could not connect to the daemon: {error}")
            }
            DaemonError::SendingError { error } => {
                write!(f, "Could not send the commadnd to the daemon: {error}")
            }
            DaemonError::MusicLibraryError { error } => {
                write!(f, "{error}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Response sent to a client from the daemon.
pub enum DaemonResponse {
    Ok,
    Status {},
    Error { error: DaemonError },
}

impl Display for DaemonResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonResponse::Ok => write!(f, "Command executed successfully"),
            DaemonResponse::Status {} => write!(f, "Daemon status response"),
            DaemonResponse::Error { error } => write!(f, "An error occured in the daemon: {error}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// A parsed CLI command that can either be an init command for the daemon, or a message to be
/// sent to a deamon.
pub enum DaemonCommand {
    /// Init command to start the daemon.
    Start,
    /// Message to be sent to an already running daemon.
    Message { command: ClientCommand },
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Command that can be sent to the daemon over a socket.
pub enum ClientCommand {
    Stop,
    Restart,
    Status,
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

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum PlaylistCommand {
    New {
        name: String,
        tracks: Option<Vec<PathBuf>>,
    },
    FromM3U8 {
        name: String,
        m3u8_file: PathBuf,
    },
    Delete {
        name: String,
    },
}

/// Try to get a namespaced socket or a filesystem socket for daemon IPC.
///
/// # Examples
///
/// ```
/// use mpipc::get_daemon_socket;
///
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

    let mut buf = BufReader::new(conn);

    match buf.get_mut().write_all(&cmd) {
        Ok(_) => {}
        Err(e) => {
            return Err(DaemonError::SendingError {
                error: e.to_string(),
            });
        }
    }

    let response = match decode_from_reader(buf, standard()) {
        Ok(response) => response,
        Err(e) => {
            return Err(DaemonError::DecodingError {
                error: e.to_string(),
            });
        }
    };

    Ok(response)
}
