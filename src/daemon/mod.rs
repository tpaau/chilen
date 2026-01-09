mod daemon_thread;

use std::{process::exit, sync::mpsc::channel, thread};

use interprocess::local_socket::{ListenerOptions, Stream, traits::ListenerExt};
use log::{debug, error, info, trace, warn};

use mpipc::{ClientCommand, DaemonError, SOCKET_NAME, get_daemon_socket};

use crate::{cache::playlists, daemon::daemon_thread::ThreadCommand};

#[derive(Debug)]
/// Parsed CLI arguments for the daemon.
pub enum DaemonCommand {
    /// Start the daemon. This is not to be sent over the daemon socket.
    Start,
    /// Command for the daemon instance.
    Message {
        /// The command content meant to be sent over the daemon socket.
        command: ClientCommand,
    },
}

impl TryFrom<DaemonCommand> for ClientCommand {
    type Error = String;
    fn try_from(value: DaemonCommand) -> Result<Self, Self::Error> {
        match value {
            DaemonCommand::Start => Err(String::from(
                "Daemon start command cannot be converted to a client command",
            )),
            DaemonCommand::Message { command } => Ok(command),
        }
    }
}

fn handle_error(conn: std::io::Result<Stream>) -> Option<Stream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            warn!("Incoming connection failed: {e}");
            None
        }
    }
}

pub async fn start() -> Result<(), DaemonError> {
    debug!("Starting daemon on '{SOCKET_NAME}'");

    let socket = match get_daemon_socket() {
        Ok(sock) => sock,
        Err(e) => {
            error!("Could not obtain a socket: {e}");
            return Err(DaemonError::SocketError {
                error: e.to_string(),
            });
        }
    };

    trace!("Creating a listener on '{SOCKET_NAME}'");
    let opts = ListenerOptions::new().name(socket.clone());

    let listener = match opts.create_sync() {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed creating a listener for the daemon socket: '{e}'");
            return Err(DaemonError::SocketError {
                error: e.to_string(),
            });
        }
    };

    info!("Listening for incomming connections");

    thread::spawn(playlists::load);

    for conn in listener.incoming().filter_map(handle_error) {
        let (ttx, drx) = channel();

        daemon_thread::spawn(conn, ttx);

        async {
            while let Ok(msg) = drx.recv() {
                match msg {
                    ThreadCommand::Shutdown => {
                        trace!("Received shutdown command from a thread");
                        exit(0);
                    }
                    ThreadCommand::Restart => {
                        trace!("Received restart command from a thread");
                        todo!();
                    }
                }
            }
        }
        .await;
    }

    Ok(())
}
