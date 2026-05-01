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
pub const DEFAULT_SOCKET_NAME: &str = "DEFAULT_MUSIC_PLAYER.socket";

/// Error related to the `daemon`.
///
/// Can either originate from a [Response] or from a function in [mpipc](crate).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Error {
    /// The message could not be encoded before sending it.
    EncodingError,
    /// Response could not be decoded back into usable form.
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
    /// Could not obtain a socket
    SocketError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncodingError => write!(f, "Could not encode the deamon command"),
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
        }
    }
}

/// Event from the `daemon` received in [Response::Event].
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
}

/// Response sent to a client from the `daemon`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Response {
    /// The client command was executed successfully.
    Ok,
    /// Response to [Command::Ping].
    Pong,
    /// The contents of the music library.
    Library(MusicLibrary),
    /// An event from the daemon.
    Event(Event),
    /// Response from the playback module.
    Playback(PlaybackResponse),
    /// The client command has failed.
    Error(Error),
}

/// Command that can be executed by a `daemon` instance.
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
    ///
    /// The `daemon` will respond to this with [Response::Ok] if successful.
    Disconnect,
    /// Command to the playback module.
    ///
    /// The `daemon` will respond to this with [Response::Ok] if successful.
    Playback(PlaybackCommand),
    /// Ping the `daemon`.
    ///
    /// The `daemon` will respond to this with [Response::Pong] if successful.
    Ping,
}

/// Defines the socket type to use when attempting to connect to a `deamon`.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocketType {
    /// Only use a namespaced socket with no fallback.
    ///
    /// The `daemon` will throw an error at startup if a socket with the specified name already
    /// exists.
    NamespacedOnly,
    /// Use a namespaced socket when possible, but allow a fallback to a filesystem socket.
    ///
    /// The `daemon` will only throw an error if it fails to create both a namespaced and
    /// filesystem socket listener.
    #[default]
    NamespacedOrFilesystem,
    /// Only use a filesystem socket with no fallback.
    ///
    /// The `daemon` will throw an error at startup if a socket with the specified name already
    /// exists.
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
/// # use mpipc::{connect, DEFAULT_SOCKET_NAME, SocketType, serialize_client_command, Command, Error};
/// let conn = connect(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap();
/// let mut conn = BufReader::new(conn);
/// let cmd = mpipc::serialize_client_command(&mpipc::Command::Disconnect).unwrap();
/// conn.get_mut().write_all(&cmd).unwrap();
/// ```
pub fn serialize_client_command(cmd: &Command) -> Result<Vec<u8>, Error> {
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
/// # use mpipc::{connect, DEFAULT_SOCKET_NAME, SocketType, serialize_client_command, Command, receive_response};
/// let conn = connect(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap();
/// let mut conn = BufReader::new(conn);
/// let cmd = serialize_client_command(&Command::EventStream).unwrap();
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

/// Disconnects from the `daemon` by sending the [Command::Disconnect] command.
///
/// This is a convenience function, its effect could be achieved using utilities already provided
/// by [mpipc](crate).
///
/// # Examples
/// ```no_run
/// # use std::io::BufReader;
/// # use mpipc::{connect, DEFAULT_SOCKET_NAME, SocketType, disconnect};
/// let conn = connect(DEFAULT_SOCKET_NAME, &SocketType::default()).unwrap();
/// let mut conn = BufReader::new(conn);
///
/// // Do some stuff with the connection here...
///
/// match disconnect(&mut conn) {
///     Ok(_) => eprintln!("Disconnected from the deamon!"),
///     Err(e) => panic!("Could not close the daemon connection: {e}"),
/// }
/// ```
pub fn disconnect(conn: &mut BufReader<Stream>) -> Result<(), Error> {
    trace!("Closing connection with the daemon");

    let data = serialize_client_command(&Command::Disconnect)?;

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

/// Connects to the `daemon` via a local socket and returns the connection [Stream].
///
/// # Examples
/// ```no_run
/// use std::io::BufReader;
/// # use mpipc::{connect, DEFAULT_SOCKET_NAME, SocketType, disconnect};
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
/// # use mpipc::{send_command, Command, DEFAULT_SOCKET_NAME, SocketType, Response};
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

    let data = serialize_client_command(&cmd)?;

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
