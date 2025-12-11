use std::{
    sync::LazyLock,
    fmt::Display
};

use bincode::Decode;
use serde::{Deserialize, Serialize};
use interprocess::local_socket::{
    GenericNamespaced, Name, ToNsName
};

pub const SOCKET_NAME: &str = "MUSIC_PLAYER.socket";
pub static SOCKET: LazyLock<Result<Name<'_>, std::io::Error>> = LazyLock::new(|| {
    SOCKET_NAME.to_ns_name::<GenericNamespaced>()
});

#[derive(Serialize, Deserialize, Decode, Debug, Clone)]
pub enum DaemonExitStatus {
    ExitRequested,
    ExitedUnexpectedly,
    SocketTaken,
    SocketError,
}

impl Display for DaemonExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonExitStatus::ExitRequested => {
                write!(f, "Daemon stopped because another process requested it to exit")
            }
            DaemonExitStatus::ExitedUnexpectedly => {
                write!(f, "Daemonm stopped unexpectedly for an unknown reason")
            }
            DaemonExitStatus::SocketTaken => {
                write!(f, "The daemon socket is taken, likely by another daemon instance")
            }
            DaemonExitStatus::SocketError => {
                write!(f, "A socket could not be created")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Decode, Debug, Clone)]
pub enum ClientCommand {
    Shutdown
}
