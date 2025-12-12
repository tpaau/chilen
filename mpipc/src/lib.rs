use std::{fmt::Display, sync::LazyLock};

use bincode::{Decode, Encode, config::standard, encode_to_vec};
use interprocess::local_socket::{GenericNamespaced, Name, Stream, ToNsName, prelude::*};
use log::{error, trace};
use serde::{Deserialize, Serialize};

pub const SOCKET_NAME: &str = "MUSIC_PLAYER.socket";
pub static SOCKET: LazyLock<Result<Name<'_>, std::io::Error>> =
    LazyLock::new(|| SOCKET_NAME.to_ns_name::<GenericNamespaced>());

#[derive(Serialize, Deserialize, Decode, Debug)]
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

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
pub enum DaemonError {
    StoppedUnexpectedly,
    EncodingError { error: String },
    SocketError { error: String },
    ConnectionError { error: String },
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
            DaemonError::SocketError { error } => {
                write!(f, "Socket error: {error}")
            }
            DaemonError::ConnectionError { error } => {
                write!(f, "Connection error: {error}")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Decode, Debug)]
pub enum DaemonResponse {
    Ok,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
pub enum DaemonCommand {
    Start,
    Stop,
    Restart,
    Status,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub enum ClientCommand {
    Stop,
    Restart,
    Status,
}

pub fn get_daemon_socket<'a>() -> Result<Name<'a>, &'a std::io::Error> {
    trace!("Obtaining a namespaced socket for '{SOCKET_NAME}'");

    match &*SOCKET {
        Ok(socket) => Ok(socket.clone()),
        Err(e) => {
            error!("Could not obtain a socket: {e}");
            Err(e)
        }
    }
}

pub fn serialize_command(cmd: ClientCommand) -> Result<Vec<u8>, DaemonError> {
    let cmd = match encode_to_vec(cmd, standard()) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed serializing command: {e}");
            return Err(DaemonError::EncodingError {
                error: e.to_string(),
            });
        }
    };

    Ok(cmd)
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

pub fn exec_daemon_command(cmd: DaemonCommand) -> Result<DaemonResponse, DaemonError> {
    trace!("Executing daemon command: {cmd:?}");

    let conn = match connect_to_daemon() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Could not execute daemon command: {e:?}");
            return Err(e);
        }
    };

    panic!("Not implemented!")
}
