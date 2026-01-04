use std::{
    io::{BufReader, Write},
    process::exit,
    sync::mpsc::{Sender, channel},
    thread::{self, JoinHandle},
};

use bincode::{config::standard, decode_from_std_read, encode_to_vec};
use interprocess::local_socket::{ListenerOptions, Stream, traits::ListenerExt};
use log::{debug, error, info, trace, warn};

use mpipc::{
    ClientCommand, DaemonError, DaemonExitStatus, DaemonResponse, PlaylistCommand, SOCKET_NAME,
    get_daemon_socket,
};
use serde::{Deserialize, Serialize};

use crate::cache::playlists;

#[derive(Serialize, Deserialize, Debug)]
enum ThreadCommand {
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

fn respond(mut conn: BufReader<&Stream>, msg: &DaemonResponse) {
    let msg = match encode_to_vec(msg, standard()) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Could not prepare the command for the client: {e}");
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

fn spawn_daemon_thread(conn: Stream, ttx: Sender<ThreadCommand>) -> JoinHandle<()> {
    trace!("New connection to the daemon: {conn:?}");

    thread::spawn(move || {
        trace!("Handling client connection");

        loop {
            let mut conn = BufReader::new(&conn);

            let command = match decode_from_std_read(&mut conn, standard()) {
                Ok(cmd) => cmd,
                Err(e) => {
                    error!("Failed decoding a client command: {e}");
                    break;
                }
            };

            match command {
                ClientCommand::Stop => {
                    info!("Received shutdown command from the client");

                    if let Err(e) = ttx.send(ThreadCommand::Shutdown) {
                        error!("Failed sending message to the daemon: {e}");
                    } else {
                        respond(conn, &DaemonResponse::Ok);
                    }

                    break;
                }
                ClientCommand::Restart => {
                    info!("Received restart command from the client");
                    panic!("Daemon restart is not implemented!");
                }
                ClientCommand::Status => todo!(),
                ClientCommand::Playlist { cmd } => match cmd {
                    PlaylistCommand::New { name, tracks } => {
                        if let Err(e) = playlists::create_playlist(name, &tracks) {
                            respond(
                                conn,
                                &DaemonResponse::Error {
                                    error: DaemonError::MusicLibraryError { error: e },
                                },
                            );
                            break;
                        }
                        respond(conn, &DaemonResponse::Ok);
                    }
                    PlaylistCommand::FromM3U8 { name, m3u8_file } => {
                        if let Err(e) = playlists::create_playlist_from_m3u8(&name, &m3u8_file) {
                            respond(
                                conn,
                                &DaemonResponse::Error {
                                    error: DaemonError::MusicLibraryError { error: e },
                                },
                            );
                            break;
                        }
                        respond(conn, &DaemonResponse::Ok);
                    }
                    PlaylistCommand::Delete { name } => {
                        if let Err(e) = playlists::delete_playlist(&name) {
                            respond(
                                conn,
                                &DaemonResponse::Error {
                                    error: DaemonError::MusicLibraryError { error: e },
                                },
                            );
                            break;
                        }
                        respond(conn, &DaemonResponse::Ok);
                    }
                },
            };
        }

        trace!("Closing client connection");
    })
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

    info!("Listening for incomming connections");

    thread::spawn(playlists::load);

    for conn in listener.incoming().filter_map(handle_error) {
        let (ttx, drx) = channel();

        spawn_daemon_thread(conn, ttx);

        async {
            while let Ok(msg) = drx.recv() {
                match msg {
                    ThreadCommand::Shutdown => {
                        trace!("Received shutdown command from a thread");
                        exit(0);
                    }
                    ThreadCommand::Restart => {
                        trace!("Received restart command from a thread");
                    }
                }
            }
        }
        .await;
    }

    Ok(DaemonExitStatus::ExitRequested)
}
