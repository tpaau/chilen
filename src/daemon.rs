use std::{io::BufReader, thread};

use bincode::{config::standard, decode_from_std_read};
use interprocess::local_socket::{ListenerOptions, Stream, traits::ListenerExt};
use log::{debug, error, info, trace, warn};

use mpipc::{ClientCommand, DaemonExitStatus, SOCKET_NAME, get_daemon_socket};

fn handle_error(conn: std::io::Result<Stream>) -> Option<Stream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            warn!("Incoming connection failed: {e}");
            None
        }
    }
}

fn handle_connection(conn: Stream) {
    trace!("New connection to the daemon: {conn:?}");

    thread::spawn(|| {
        trace!("New thread spawned for the connection");

        let mut conn = BufReader::new(conn);

        let command = match decode_from_std_read(&mut conn, standard()) {
            Ok(cmd) => cmd,
            Err(e) => {
                error!("Faild to decode a client command: {e}");
                return;
            }
        };

        match command {
            ClientCommand::Stop => {
                info!("Shutting down (client request)");
                panic!("Shutdown not implemented!")
            }
            ClientCommand::Restart => {
                info!("Restarting down (client request)");
                panic!("Daemon restart is not implemented!");
            }
            ClientCommand::Status => {
                panic!("Daemon status is not implemented!");
            }
        }
    });
}

pub fn start() -> Result<DaemonExitStatus, DaemonExitStatus> {
    debug!("Starting daemon on '{SOCKET_NAME}'");

    let socket = match get_daemon_socket() {
        Ok(sock) => sock,
        Err(e) => {
            error!("Could not obtain a socket: {e}");
            return Err(DaemonExitStatus::SocketError);
        }
    };

    trace!("Creating a listener on '{SOCKET_NAME}'");
    let opts = ListenerOptions::new().name(socket.clone());

    let listener = match opts.create_sync() {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed creating a listener for the daemon socket: '{e}'");
            return Err(DaemonExitStatus::SocketTaken);
        }
    };

    debug!("Daemon listening on '{SOCKET_NAME}'");

    for conn in listener.incoming().filter_map(handle_error) {
        handle_connection(conn);
    }

    Err(DaemonExitStatus::StoppedUnexpectedly)
}
