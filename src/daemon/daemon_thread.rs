use std::{
    io::{BufReader, Write},
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

use bincode::{config::standard, decode_from_std_read, encode_to_vec};
use interprocess::local_socket::Stream;
use log::{debug, error, info, trace};

use mpipc::{ClientCommand, DaemonError, DaemonResponse, PlaylistCommand};

use crate::cache::playlists::{
    create_playlist, delete_playlist, get_library, import_playlist_from_m3u8, save_library,
};

#[derive(Debug)]
pub enum ThreadCommand {
    Shutdown,
    Restart,
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

pub fn spawn(conn: Stream, ttx: Sender<ThreadCommand>) -> JoinHandle<()> {
    trace!("New connection to the daemon: {conn:?}");

    thread::spawn(move || {
        trace!("Handling client connection");

        loop {
            let mut conn = BufReader::new(&conn);

            trace!("Awaiting client command");

            let command = match decode_from_std_read(&mut conn, standard()) {
                Ok(cmd) => cmd,
                Err(e) => {
                    error!("Failed decoding a client command: {e}");
                    return;
                }
            };

            match command {
                ClientCommand::Stop | ClientCommand::Restart => {
                    let thread_command = if command == ClientCommand::Stop {
                        info!("Received shutdown command from the client");
                        ThreadCommand::Shutdown
                    } else {
                        info!("Received restart command from the client");
                        ThreadCommand::Restart
                    };
                    respond(conn, &DaemonResponse::Ok);
                    if let Err(e) = ttx.send(thread_command) {
                        error!("Failed sending message to the daemon: {e}");
                        debug!("The connection will be closed anyway due to client expectation");
                    }
                    trace!("Closing client connection (implicit)");
                    return;
                }
                ClientCommand::Status => todo!(),
                ClientCommand::Playlist { cmd } => match cmd {
                    PlaylistCommand::New { name, tracks } => {
                        if let Err(e) = create_playlist(name, &tracks) {
                            respond(
                                conn,
                                &DaemonResponse::Error {
                                    error: DaemonError::MusicLibraryError { error: e },
                                },
                            );
                            return;
                        }
                        respond(conn, &DaemonResponse::Ok);
                        return;
                    }
                    PlaylistCommand::FromM3U8 { name, m3u8_file } => {
                        if let Err(e) = import_playlist_from_m3u8(name, &m3u8_file) {
                            respond(
                                conn,
                                &DaemonResponse::Error {
                                    error: DaemonError::MusicLibraryError { error: e },
                                },
                            );
                            return;
                        }
                        respond(conn, &DaemonResponse::Ok);
                        return;
                    }
                    PlaylistCommand::Delete { names } => {
                        for name in names {
                            if let Err(e) = delete_playlist(&name, false) {
                                respond(
                                    conn,
                                    &DaemonResponse::Error {
                                        error: DaemonError::MusicLibraryError { error: e },
                                    },
                                );
                                return;
                            }
                        }
                        if let Err(e) = save_library() {
                            respond(
                                conn,
                                &DaemonResponse::Error {
                                    error: DaemonError::MusicLibraryError { error: e },
                                },
                            );
                            return;
                        }
                        respond(conn, &DaemonResponse::Ok);
                        return;
                    }
                    PlaylistCommand::List => {
                        match get_library() {
                            Ok(lib) => {
                                let mut playlists = Vec::new();
                                for playlist in lib.playlists {
                                    playlists.push(playlist.into());
                                }
                                respond(conn, &DaemonResponse::Playlists { playlists });
                            }
                            Err(e) => {
                                respond(
                                    conn,
                                    &DaemonResponse::Error {
                                        error: DaemonError::MusicLibraryError { error: e },
                                    },
                                );
                            }
                        }
                        return;
                    }
                },
            };
        }
    })
}
