use std::{
    io::{BufReader, Write},
    sync::mpsc::{Receiver, Sender, channel},
    thread::{self, JoinHandle},
};

use bincode::{config::standard, decode_from_std_read, encode_to_vec};
use interprocess::local_socket::{ListenerOptions, Stream, traits::ListenerExt};
use log::{debug, error, info, trace, warn};

use mpipc::{
    ClientCommand, DaemonError, DaemonExitStatus, DaemonResponse, SOCKET_NAME, get_daemon_socket,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
/// Command sent from the daemon to its threads.
enum ThreadCommand {
    Quit,
    Test,
}

#[derive(Serialize, Deserialize, Debug)]
enum ThreadMessage {
    Shutdown,
    Restart,
}

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

async fn handle_client_connection(
    conn: Stream,
    ttx: Sender<ThreadMessage>,
    trx: Receiver<ThreadCommand>,
) {
    trace!("Handling client connection");

    async move {
        while let Ok(msg) = trx.recv() {
            match msg {
                ThreadCommand::Quit => {
                    trace!("Shutdown command recieved");
                    return;
                }
                ThreadCommand::Test => {
                    trace!("Test command recieved")
                }
            }
        }
    }
    .await;

    loop {
        async {
            let mut conn = BufReader::new(&conn);

            let command = match decode_from_std_read(&mut conn, standard()) {
                Ok(cmd) => cmd,
                Err(e) => {
                    error!("Failed to decode a client command: {e}");
                    return;
                }
            };

            match command {
                ClientCommand::Stop => {
                    info!("Shutting down (client request)");

                    if let Err(e) = ttx.send(ThreadMessage::Shutdown) {
                        error!("Failed sending message to the daemon: {e}");
                    } else {
                        let msg = match encode_to_vec(DaemonResponse::Ok, standard()) {
                            Ok(msg) => msg,
                            Err(e) => {
                                error!("Could not serialize the response: {e}");
                                return;
                            }
                        };

                        match conn.get_mut().write_all(&msg) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Failed sending response message: {e}");
                            }
                        };
                    }

                    // panic!("Shutdown not implemented!");
                }
                ClientCommand::Restart => {
                    info!("Restarting (client request)");
                    panic!("Daemon restart is not implemented!");
                }
                ClientCommand::Status => {
                    panic!("Daemon status is not implemented!");
                }
            }
        }
        .await;
    }
}

fn spawn_daemon_thread(
    conn: Stream,
    ttx: Sender<ThreadMessage>,
    trx: Receiver<ThreadCommand>,
) -> JoinHandle<impl Future> {
    trace!("New connection to the daemon: {conn:?}");
    thread::spawn(move || handle_client_connection(conn, ttx, trx))
}

pub async fn start() -> Result<DaemonExitStatus, DaemonError> {
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

    debug!("Daemon listening on '{SOCKET_NAME}'");

    let mut threads = Vec::new();
    for conn in listener.incoming().filter_map(handle_error) {
        let (dtx, trx) = channel();
        let (ttx, drx) = channel();

        threads.push((spawn_daemon_thread(conn, ttx, trx), dtx));

        async {
            while let Ok(msg) = drx.recv() {
                match msg {
                    ThreadMessage::Shutdown => {
                        for (thread, dtx) in &threads {
                            trace!("Sending quit command to thread: {thread:?}");
                            if let Err(e) = dtx.send(ThreadCommand::Quit) {
                                error!("Failed sending command to the thread: {e}");
                            }
                        }
                    }
                    ThreadMessage::Restart => {
                        for (thread, dtx) in &threads {
                            trace!("Sending quit command to thread: {thread:?}");
                            if let Err(e) = dtx.send(ThreadCommand::Quit) {
                                error!("Failed sending command to the thread: {e}");
                            }
                        }
                    }
                }
            }
        }
        .await;
    }

    for (thread, _) in threads {
        if let Err(e) = thread.join() {
            error!("Failed joining a thread handle: {e:?}");
        }
    }

    Ok(DaemonExitStatus::ExitRequested)
}
