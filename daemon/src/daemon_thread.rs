use std::{
    io::{BufReader, Write},
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

use interprocess::local_socket::Stream;
use log::{error, info, trace};

use mpipc::{ClientCommand, DaemonError, DaemonResponse, PlaylistCommand};
use rmp_serde::{Serializer, from_read};
use serde::Serialize;

use crate::{
    DaemonEvent,
    cache::music_lib::{
        self, add_tracks, create_playlist, delete_playlist, get_library, import_playlist_from_m3u8,
        remove_tracks, save_library,
    },
    send_event,
};

fn respond(conn: &mut BufReader<&Stream>, msg: &DaemonResponse) {
    let mut data = Vec::new();
    if let Err(e) = msg.serialize(&mut Serializer::new(&mut data)) {
        error!("Could not prepare the command for the client: {e}");
        return;
    }

    match conn.get_mut().write_all(&data) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed sending response message: {e}");
        }
    };
}

pub(crate) fn spawn(conn: Stream, trx: Receiver<DaemonEvent>) -> JoinHandle<()> {
    trace!("New connection to the daemon: {conn:?}");

    thread::spawn(move || {
        trace!("Handling client connection");

        loop {
            let mut conn = BufReader::new(&conn);

            trace!("Awaiting client command");

            let command: ClientCommand = match from_read(&mut conn) {
                Ok(cmd) => cmd,
                Err(e) => {
                    error!("Failed decoding a client command: {e}");
                    return;
                }
            };

            match command {
                ClientCommand::Shutdown | ClientCommand::Restart => {
                    let event = if command == ClientCommand::Shutdown {
                        info!("Received shutdown command from the client");
                        trace!("Closing client connection (shutdown)");
                        DaemonEvent::Shutdown
                    } else {
                        info!("Received restart command from the client");
                        trace!("Closing client connection (restart)");
                        DaemonEvent::Restart
                    };
                    respond(&mut conn, &DaemonResponse::Ok);
                    // TODO: Error handling
                    if let Err(e) = send_event(event) {
                        error!("Could not send the command to the daemon: {e}");
                        trace!(
                            "The connection will be closed regardles due to client expectations"
                        );
                    }
                    return;
                }
                ClientCommand::Playlist(cmd) => match cmd {
                    PlaylistCommand::New { name, tracks } => {
                        if let Err(e) = create_playlist(name, &tracks) {
                            respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            );
                            continue;
                        }
                        respond(&mut conn, &DaemonResponse::Ok);
                    }
                    PlaylistCommand::FromM3U8 { name, m3u8_file } => {
                        if let Err(e) = import_playlist_from_m3u8(name, &m3u8_file) {
                            respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            );
                            continue;
                        }
                        respond(&mut conn, &DaemonResponse::Ok);
                    }
                    PlaylistCommand::Delete { names } => {
                        for name in names {
                            if let Err(e) = delete_playlist(&name, false) {
                                respond(
                                    &mut conn,
                                    &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                                );
                            }
                        }
                        if let Err(e) = save_library() {
                            respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            );
                            continue;
                        }
                        respond(&mut conn, &DaemonResponse::Ok);
                    }
                    PlaylistCommand::AddTracks { name, tracks } => {
                        match add_tracks(&name, tracks) {
                            Ok(_) => respond(&mut conn, &DaemonResponse::Ok),
                            Err(e) => respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            ),
                        }
                    }
                    PlaylistCommand::RemoveTracks { name, ids } => {
                        match remove_tracks(&name, ids) {
                            Ok(_) => respond(&mut conn, &DaemonResponse::Ok),
                            Err(e) => respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            ),
                        }
                    }
                    PlaylistCommand::List => match get_library() {
                        Ok(lib) => {
                            let mut playlists = Vec::new();
                            for playlist in lib.playlists {
                                playlists.push(playlist.into());
                            }
                            respond(&mut conn, &DaemonResponse::Playlists(playlists));
                        }
                        Err(e) => {
                            respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            );
                        }
                    },
                },
                ClientCommand::Cache(cmd) => {
                    match music_lib::load(music_lib::LoadMode::from_cache_command(cmd)) {
                        Ok(_) => respond(&mut conn, &DaemonResponse::Ok),
                        Err(e) => respond(
                            &mut conn,
                            &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                        ),
                    }
                }
                ClientCommand::EventStream => {
                    trace!("Streaming daemon events to the client");
                    loop {
                        match trx.recv() {
                            Ok(event) => {
                                trace!(
                                    "Received an event from the daemon: {event:?}, realaying it to the client"
                                );
                                respond(&mut conn, &DaemonResponse::Event(event));
                            }
                            Err(e) => {
                                trace!("Could not receive an event from the daemon: {e}");
                                return;
                            }
                        }
                    }
                }
                ClientCommand::Disconnect => {
                    respond(&mut conn, &DaemonResponse::Ok);
                    trace!("Closing client connection (client request)");
                    return;
                }
            };
        }
    })
}
