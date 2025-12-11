use std::io::BufReader;

use bincode::{config::standard, decode_from_std_read};
use interprocess::local_socket::{
    traits::ListenerExt, ListenerOptions, Stream
};
use log::{trace, debug, warn, error};

use mpipc::{
    DaemonExitStatus,
    ClientCommand,
    SOCKET_NAME,
    SOCKET,
};

fn handle_error(conn: std::io::Result<Stream>) -> Option<Stream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            warn!("Incoming connection failed: {e}");
            None
        }
    }
}

pub fn start() -> Result<DaemonExitStatus, DaemonExitStatus> {
    debug!("Starting daemon on '{SOCKET_NAME}'");

    trace!("Obtaining a namespaced socket for '{SOCKET_NAME}'");
    let socket = match &*SOCKET {
        Ok(socket) => socket,
        Err(e) => {
            error!("Could not create a socket: {e}");
            return Err(DaemonExitStatus::SocketError);
        }
    };

    let opts = ListenerOptions::new().name(socket.clone());

    trace!("Creating a namespaced listener on '{SOCKET_NAME}'");
    let listener = match opts.create_sync() {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed creating a listener for the daemon socket: '{e}'");
            return Err(DaemonExitStatus::SocketTaken);
        }
    };

    debug!("Daemon listening on '{SOCKET_NAME}'");

    for conn in listener.incoming().filter_map(handle_error) {
        trace!("New connection to the daemon: {conn:?}");

        let mut conn = BufReader::new(conn);

        let command = match decode_from_std_read(&mut conn, standard()) {
            Ok(cmd) => cmd,
            Err(e) => {
                error!("Faild to decode a client command: {e}");
                continue;
            }
        };

        match command {
            ClientCommand::Shutdown => {
                return Ok(DaemonExitStatus::ExitRequested);
            }
        }
    }

    Err(DaemonExitStatus::ExitedUnexpectedly)
}
