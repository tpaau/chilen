#![doc = include_str!("../README.md")]

pub mod library;
pub mod playback;

use std::{
    env::temp_dir,
    io::{BufReader, Write},
    path::PathBuf,
};

pub use interprocess::local_socket::Stream;
use interprocess::local_socket::{GenericFilePath, GenericNamespaced, Name, ToNsName, prelude::*};
use log::{error, info, trace};
use serde::{Deserialize, Serialize};

use crate::{
    library::{LibraryCommand, MusicLibrary},
    playback::{PlaybackCommand, PlaybackEvent, PlaybackResponse},
};

/// The default name of the socket the daemon listens on.
///
/// This can be used for testing, but please do not use this socket name in a finished project.
pub const DEFAULT_SOCKET_NAME: &str = "DEFAULT_MUSIC_PLAYER.socket";

/// Error related to the daemon.
///
/// Can either originate from a [`Response`] or from a function in this crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Error {
    /// The provided command could not be encoded.
    EncodingError,
    /// The daemon response could not be decoded.
    DecodingError,
    /// Could not connect to the daemon.
    ConnectionError,
    /// Could not send the command to the daemon.
    SendingError,
    /// The response received from the daemon was unexpected or invalid.
    InvalidResponse,
    /// Could not obtain a socket.
    SocketError(String),
    /// Raise requests from external clients are not allowed.
    RaiseDisabled,
    /// Quit requests from external clients are not allowed.
    QuitDisabled,
    /// The audio player is not connected.
    ///
    /// This may happen if the device doesn't have an audio device or none of the audio devices are
    /// marked as default.
    PlayerNotConnected,
    /// The playback state is not initialized.
    ///
    /// This error may occur when a [`PlaybackCommand`] is sent to the daemon too early, before the
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
    /// This means that the current track is first in the queue and the
    /// [loop state](playback::LoopState) is set to [`LoopState::Off`](playback::LoopState::Off).
    CannotGoPrevious,
    /// Cannot go to the next track.
    ///
    /// This means that the current track is last in the queue and the
    /// [loop state](playback::LoopState) is set to [`LoopState::Off`](playback::LoopState).
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
    /// The player will refuse to seek by 0s to prevent audio popping.
    InvalidDuration,
    /// Overflow detected while performing a seek operation.
    DurationOverflow,
    /// Could not complete the operation because a [playlist](library::Playlist) with the provided
    /// name already exists.
    PlaylistExists,
    /// Could not perform the operation because the [music library](MusicLibrary) is not
    /// initialized.
    ///
    /// This can happen if a command is sent to early and the music library is not yet initialized.
    LibraryNotInitialized,
    /// There is no [playlist](library::Playlist) in the [music library](MusicLibrary) with the
    /// provided name.
    UnknownPlaylist,
    /// The provided item index was out of bounds.
    IndexOutOfBounds,
    /// The provided list contained duplicate values.
    DuplicateItems,
    /// The provided track is not registered in the library.
    UnknownTrack,
    /// Could not read the contents of the library state file.
    StateNotReadable,
    /// Could not write the library state to a file.
    StateWriteFailed,
    /// The library state path is not a file.
    StateNotAFile,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncodingError => write!(f, "Could not encode the daemon command"),
            Self::DecodingError => write!(f, "Could not decode the response from the daemon"),
            Self::ConnectionError => write!(f, "Could not connect to the daemon"),
            Self::SendingError => write!(f, "Could not send the command to the daemon"),
            Self::InvalidResponse => {
                write!(f, "The response from the daemon was invalid or malformed")
            }
            Self::SocketError(e) => write!(f, "Could not obtain a socket: {e}"),
            Self::RaiseDisabled => {
                write!(f, "Raise requests from external clients are not allowed")
            }
            Self::QuitDisabled => write!(f, "Quit requests from external clients are not allowed"),
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
            Self::UnknownTrack => {
                write!(f, "The provided track was not found in the music library")
            }
            Self::PlaylistExists => write!(f, "Playlist with this name already exists"),
            Self::LibraryNotInitialized => write!(f, "The music library is not initialized"),
            Self::UnknownPlaylist => write!(f, "There is no playlist with this name"),
            Self::IndexOutOfBounds => write!(f, "The provided item index was out of bounds"),
            Self::DuplicateItems => write!(f, "The provided vector contained duplicate values"),
            Self::StateNotReadable => {
                write!(f, "Could not read the contents of the library state file")
            }
            Self::StateWriteFailed => write!(f, "Could not write the library state to a file"),
            Self::StateNotAFile => write!(f, "The library state path is not a file"),
        }
    }
}

/// Event from the daemon received in [`Response::Event`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// Sent before the daemon quits.
    Quit,
    /// Sent after the contents of the music library have changed.
    LibraryChanged(MusicLibrary),
    /// Event originating from the playback module of the daemon.
    PlaybackEvent(PlaybackEvent),
    /// Sent when the `can_raise` property of the daemon config changes.
    CanRaiseChanged(bool),
    /// Sent when the `can_quit` property of the daemon config changes.
    CanQuitChanged(bool),
}

/// Response sent to a client from the daemon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Response {
    /// The client command was executed successfully.
    Ok,
    /// Response to [`Command::Ping`].
    Pong,
    /// The contents of the music library.
    Library(MusicLibrary),
    /// An event from the daemon.
    Event(Event),
    /// Response from the playback module.
    Playback(PlaybackResponse),
    /// The client command has failed.
    Error(Error),
    /// Response to [`Command::CanRaise`].
    CanRaise(bool),
    /// Response to [`Command::CanQuit`].
    CanQuit(bool),
}

/// Command that can be executed by a daemon instance.
///
/// The expected response may be different depending on the command sent. If it isn't specified in
/// the variant documentation, assume [`Response::Ok`] is the expected response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Stop the daemon instance.
    ///
    /// After sending this command, the daemon will close almost immediately, so all connections
    /// to it should be considered closed.
    Quit,
    /// Subcommand for managing the music library.
    Library(LibraryCommand),
    /// Stream [events](Event) from the daemon.
    ///
    /// The daemon will stop accepting requests from the connection this command was executed on.
    ///
    /// This command may fail if initial events cannot be obtained (eg. the music library is not
    /// initialized or the queue state is not ready). The daemon will never return an incomplete
    /// set of initial events if some cannot be obtained.
    EventStream,
    /// Close the connection to the daemon.
    Disconnect,
    /// Command to the playback module.
    Playback(PlaybackCommand),
    /// Ping the daemon.
    ///
    /// The daemon will respond to this with [`Response::Pong`] if successful.
    Ping,
    /// Check if the daemon can raise.
    CanRaise,
    /// Request the daemon to raise.
    Raise,
    /// Check if the daemon accepts quit requests from clients.
    CanQuit,
}

/// Defines the socket type to use when attempting to connect to a daemon.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocketType {
    /// Only use a namespaced socket with no fallback.
    ///
    /// The connection will fail if a namespaced socket with the specified name doesn't exist.
    NamespacedOnly,
    /// Use a namespaced socket when possible, but allow a fallback to a filesystem socket.
    ///
    /// The connection will fail if there are no namespaced or filesystem sockets with the
    /// specified name.
    #[default]
    NamespacedOrFilesystem,
    /// Only use a filesystem socket with no fallback.
    ///
    /// The connection will fail if a filesystem socket with the specified name doesn't exist.
    FilesystemOnly,
}

/// Returns a filesystem path for the given socket name.
fn get_fs_socket_path(socket_name: &str) -> PathBuf {
    let mut temp_dir = temp_dir();
    temp_dir.push(socket_name);
    temp_dir
}

/// Attempts to get a socket address for daemon IPC.
///
/// # Examples
/// ```
/// match chilen_ipc::get_socket(
///     chilen_ipc::DEFAULT_SOCKET_NAME,
///     &chilen_ipc::SocketType::NamespacedOrFilesystem
/// ) {
///     Ok(socket) => eprintln!("Got a socket: {socket:?}"),
///     Err(e) => panic!("Could not obtain a socket: {e}"),
/// }
/// ```
pub fn get_socket<'a>(socket_name: &'a str, mode: &SocketType) -> Result<Name<'a>, std::io::Error> {
    match mode {
        SocketType::NamespacedOnly => socket_name.to_ns_name::<GenericNamespaced>(),
        SocketType::FilesystemOnly => {
            get_fs_socket_path(socket_name).to_fs_name::<GenericFilePath>()
        }
        SocketType::NamespacedOrFilesystem => match socket_name.to_ns_name::<GenericNamespaced>() {
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
/// This uses the [`rmp_serde`] crate under the hood.
///
/// # Examples
///
/// Connect to the daemon and immediately disconnect.
/// ```no_run
/// # use std::io::{BufReader, Write};
/// # use chilen_ipc::{connect, DEFAULT_SOCKET_NAME, SocketType, serialize_command, Command, Error};
/// let conn = connect(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap();
/// let mut conn = BufReader::new(conn);
/// let cmd = serialize_command(&Command::Disconnect).unwrap();
/// conn.get_mut().write_all(&cmd).unwrap();
/// ```
pub fn serialize_command(cmd: &Command) -> Result<Vec<u8>, Error> {
    let mut data = Vec::new();
    if let Err(e) = cmd.serialize(&mut rmp_serde::Serializer::new(&mut data)) {
        error!("Could not encode the client command: {e}");
        return Err(Error::EncodingError);
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
/// # use chilen_ipc::{connect, DEFAULT_SOCKET_NAME, SocketType, serialize_command, Command, receive_response};
/// let conn = connect(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap();
/// let mut conn = BufReader::new(conn);
/// let cmd = serialize_command(&Command::EventStream).unwrap();
/// conn.get_mut().write_all(&cmd).unwrap();
/// loop {
///     let response = receive_response(&mut conn).unwrap();
///     println!("Got a response from the daemon: {response:?}");
/// }
/// ```
pub fn receive_response(conn: &mut BufReader<Stream>) -> Result<Response, Error> {
    match rmp_serde::from_read(conn) {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Failed decoding a daemon response: {e}");
            Err(Error::DecodingError)
        }
    }
}

/// Disconnects from the daemon by sending the [`Command::Disconnect`] command.
///
/// This is a convenience function, its effect could be achieved using utilities already provided
/// in this crate.
///
/// # Examples
/// ```no_run
/// # use std::io::BufReader;
/// # use chilen_ipc::{connect, DEFAULT_SOCKET_NAME, SocketType, disconnect};
/// let conn = connect(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap();
/// let mut conn = BufReader::new(conn);
///
/// // Do some stuff with the connection here...
///
/// match disconnect(&mut conn) {
///     Ok(_) => eprintln!("Disconnected from the daemon!"),
///     Err(e) => panic!("Could not close the daemon connection: {e}"),
/// }
/// ```
pub fn disconnect(conn: &mut BufReader<Stream>) -> Result<(), Error> {
    trace!("Closing connection with the daemon");

    let data = serialize_command(&Command::Disconnect)?;

    match conn.get_mut().write_all(&data) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed sending the command to the daemon: {e}");
            return Err(Error::SendingError);
        }
    }

    let response = receive_response(conn)?;

    match response {
        Response::Error(e) => {
            error!("Could not close the connection to the daemon: {e}");
            return Err(e);
        }
        Response::Ok => {
            trace!("Connection with the daemon closed");
        }
        _ => {
            error!("Got an unexpected response while closing the daemon connection: {response:?}");
            return Err(Error::InvalidResponse);
        }
    }

    Ok(())
}

/// Connects to the daemon via a local socket and returns the connection [`Stream`].
///
/// # Examples
/// ```no_run
/// # use std::io::BufReader;
/// # use chilen_ipc::{connect, DEFAULT_SOCKET_NAME, SocketType, disconnect};
/// let conn = connect(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap();
/// let mut conn = BufReader::new(conn);
/// eprintln!("Connected to the daemon: {conn:?}");
/// // Run all your commands here!
/// disconnect(&mut conn).unwrap();
/// ```
pub fn connect(socket_name: &str, socket_type: &SocketType) -> Result<Stream, Error> {
    trace!("Connecting to daemon on socket \"{socket_name}\"");

    let socket = match get_socket(socket_name, socket_type) {
        Ok(sock) => sock,
        Err(e) => {
            error!("Could not obtain a socket: {e}");
            return Err(Error::SocketError(e.to_string()));
        }
    };

    let conn = match Stream::connect(socket) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Could not initialize a connection to the daemon: {e}");
            return Err(Error::ConnectionError);
        }
    };

    Ok(conn)
}

/// Executes a single daemon command on a new connection and closes it.
///
/// **Warning:** if this function returns the [`Ok`] variant, this only means that the command was
/// successfully delivered to the daemon. It doesn't necessarily mean it was executed
/// successfully on the daemon side.
///
/// # Examples
/// ```no_run
/// # use chilen_ipc::{send_command, Command, DEFAULT_SOCKET_NAME, SocketType, Response};
/// match send_command(Command::Ping, DEFAULT_SOCKET_NAME, &SocketType::default()) {
///     // The `Ok` variant only means the command was delivered
///     Ok(response) => {
///         match response {
///             Response::Ok => eprintln!("Got an `Ok` response, all good!"),
///             // Depending on the type of command sent, the response from the daemon may be different.
///             _ => panic!("Got an unexpected response: {response:?}"),
///         }
///     },
///     Err(error) => panic!("Could not send a command to the daemon: {error}"),
/// }
/// ```
pub fn send_command(
    cmd: Command,
    socket_name: &str,
    socket_type: &SocketType,
) -> Result<Response, Error> {
    trace!("Executing daemon command: {cmd:?}");

    let mut conn = match connect(socket_name, socket_type) {
        Ok(conn) => BufReader::new(conn),
        Err(e) => {
            error!("{e}");
            return Err(e);
        }
    };

    let data = serialize_command(&cmd)?;

    if let Err(e) = conn.get_mut().write_all(&data) {
        error!("Failed sending the daemon command: {e}");
        return Err(Error::SendingError);
    }

    let response = receive_response(&mut conn)?;

    if cmd == Command::Quit {
        trace!("Not trying to close the connection to the daemon");
    } else if let Err(e) = disconnect(&mut conn) {
        error!("Could not close the connection to the daemon: {e}");
    }

    Ok(response)
}
