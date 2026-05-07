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
    library::{LibraryCommand, LibraryError, MusicLibrary},
    playback::{PlaybackCommand, PlaybackError, PlaybackEvent, PlaybackResponse},
};

/// The default name of the socket the daemon listens on.
///
/// This can be used for testing, but please do not use this socket name in a finished project.
pub const DEFAULT_SOCKET_NAME: &str = "DEFAULT_MUSIC_PLAYER.socket";

/// Error related to the `daemon`.
///
/// Can either originate from a [`Response`] or from a function in this crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Error {
    /// The provided command could not be encoded.
    EncodingError,
    /// The `daemon` response could not be decoded.
    DecodingError,
    /// Could not connect to the daemon.
    ConnectionError,
    /// Could not send the command to the daemon.
    SendingError,
    /// Error related to the music library.
    LibraryError(LibraryError),
    /// The response received from the daemon was unexpected or invalid.
    InvalidResponse,
    /// Error related to the playback module.
    PlaybackError(PlaybackError),
    /// Could not obtain a socket.
    SocketError(String),
    /// Raise is not supported by the daemon.
    RaiseNotSupported,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncodingError => write!(f, "Could not encode the daemon command"),
            Self::DecodingError => write!(f, "Could not decode the response from the daemon"),
            Self::ConnectionError => write!(f, "Could not connect to the daemon"),
            Self::SendingError => write!(f, "Could not send the command to the daemon"),
            Self::LibraryError(e) => {
                write!(f, "{e}")
            }
            Self::InvalidResponse => {
                write!(f, "The response from the daemon was invalid or malformed")
            }
            Self::PlaybackError(e) => write!(f, "Playback error: {e}"),
            Self::SocketError(e) => write!(f, "Could not obtain a socket: {e}"),
            Self::RaiseNotSupported => write!(f, "Raise is not supported by the daemon"),
        }
    }
}

/// Event from the `daemon` received in [`Response::Event`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// Sent before the `daemon` closes.
    Shutdown,
    /// Sent after the contents of the music library have changed.
    LibraryChanged(MusicLibrary),
    /// Sent when a client disconnects from the daemon.
    ConnectionClosed,
    /// Event originating from the playback module of the daemon.
    PlaybackEvent(PlaybackEvent),
    /// Sent when the `can_raise` property of the daemon changes.
    CanRaiseChanged(bool),
    /// Sent when a client requests the daemon to raise, and raising is enabled.
    RaiseRequested,
}

/// Response sent to a client from the `daemon`.
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
}

/// Command that can be executed by a `daemon` instance.
///
/// The expected response may be different depending on the command sent. If it isn't specified in
/// the variant documentation, assume [`Response::Ok`] is the expected response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Stop the `daemon` instance.
    ///
    /// After sending this command, the `daemon` will close almost immediately, so all connections
    /// to it should be considered closed.
    Shutdown,
    /// Subcommand for managing the music library.
    Library(LibraryCommand),
    /// Stream [events](Event) from the `daemon`.
    ///
    /// The daemon will stop accepting requests from the connection this command was executed on.
    EventStream,
    /// Close the connection to the `daemon`.
    Disconnect,
    /// Command to the playback module.
    Playback(PlaybackCommand),
    /// Ping the `daemon`.
    ///
    /// The `daemon` will respond to this with [`Response::Pong`] if successful.
    Ping,
    /// Check if the daemon can raise.
    GetCanRaise,
    /// Request the daemon to raise.
    Raise,
}

/// Defines the socket type to use when attempting to connect to a `daemon`.
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

/// Attempts to get a socket address for `daemon` IPC.
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

/// Disconnects from the `daemon` by sending the [`Command::Disconnect`] command.
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

/// Connects to the `daemon` via a local socket and returns the connection [`Stream`].
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
    trace!("Connecting to daemon on socket '{socket_name}'");

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

/// Executes a single `daemon` command on a new connection and closes it.
///
/// **Warning:** if this function returns the [`Ok`] variant, this only means that the command was
/// successfully delivered to the `daemon`. It doesn't necessarily mean it was executed
/// successfully on the `daemon` side.
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

    if cmd == Command::Shutdown {
        trace!(
            "Not trying to close the connection to the daemon, it will likely shut down by then"
        );
    } else if let Err(e) = disconnect(&mut conn) {
        error!("Could not close the connection to the daemon: {e}");
    }

    Ok(response)
}
