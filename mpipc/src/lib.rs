use std::{
    fmt::Display, io::BufReader, sync::LazyLock
};

use bincode::{config::standard, encode_to_vec, Decode, Encode};
use serde::{Deserialize, Serialize};
use interprocess::local_socket::{
    GenericNamespaced, Name, Stream, ToNsName
};
use log::{trace, error};

pub const SOCKET_NAME: &str = "MUSIC_PLAYER.socket";
pub static SOCKET: LazyLock<Result<Name<'_>, std::io::Error>> = LazyLock::new(|| {
    SOCKET_NAME.to_ns_name::<GenericNamespaced>()
});

#[derive(Serialize, Deserialize, Decode)]
pub enum DaemonExitStatus {
    ExitRequested,
    StoppedUnexpectedly,
    SocketTaken,
    SocketError,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub enum CommandError {
    EncodingError { error: String }
}

impl Display for DaemonExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonExitStatus::ExitRequested => {
                write!(f, "Daemon stopped because another process requested it to exit")
            }
            DaemonExitStatus::StoppedUnexpectedly => {
                write!(f, "Daemonm stopped unexpectedly for an unknown reason")
            }
            DaemonExitStatus::SocketTaken => {
                write!(f, "The socket is taken, likely by another daemon instance")
            }
            DaemonExitStatus::SocketError => {
                write!(f, "A socket could not be created")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub enum ClientCommand {
    Stop,
    Restart,
    Status
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

pub fn serialize_command(cmd: ClientCommand) -> Result<Vec<u8>, CommandError> {
    let cmd = match encode_to_vec(cmd, standard()) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed serializing command: {e}");
            return Err(CommandError::EncodingError{error: e.to_string()});
        }
    };

    Ok(cmd)
}

pub fn connect_to_daemon() -> Result<BufReader<Stream>, String> {

    Err(String::from("Not implemented"))
}
