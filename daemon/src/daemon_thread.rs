use std::{
    io::{BufReader, Write},
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

use interprocess::local_socket::Stream;
use log::{error, info, trace, warn};

use mpipc::{ClientCommand, DaemonError, DaemonResponse, PlaylistCommand};
use rmp_serde::{Serializer, from_read};
use serde::Serialize;

use crate::{
    DaemonEvent,
    data::music_lib::{
        self, LoadMode, add_tracks, create_playlist, delete_playlist, get_library,
        import_playlist_from_m3u8, remove_tracks, save_library,
    },
    playback, send_event,
};

fn respond(conn: &mut BufReader<&Stream>, msg: &DaemonResponse) -> Result<(), DaemonError> {
    let mut data = Vec::new();
    if let Err(e) = msg.serialize(&mut Serializer::new(&mut data)) {
        error!("Could not prepare the command for the client: {e}");
        return Err(DaemonError::EncodingError);
    }

    match conn.get_mut().write_all(&data) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed sending the response to the client: {e}");
            warn!("The client likely crashed or was closed forcefully, dropping the connection");
            Err(DaemonError::SendingError)
        }
    }
}

pub(crate) fn spawn(conn: Stream, trx: Receiver<DaemonEvent>, index: u64) -> JoinHandle<()> {
    thread::spawn(move || {
        trace!("Handling client connection");
        loop {
            let mut conn = BufReader::new(&conn);

            let command: ClientCommand = match from_read(&mut conn) {
                Ok(cmd) => cmd,
                Err(e) => {
                    error!("Failed decoding a client command: {e}");
                    break;
                }
            };

            match command {
                ClientCommand::Shutdown => {
                    info!("Received shutdown command from the client");
                    trace!("Closing client connection (shutdown)");
                    if let Err(DaemonError::SocketError) = respond(&mut conn, &DaemonResponse::Ok) {
                        break;
                    }
                    if let Err(e) = send_event(DaemonEvent::Shutdown) {
                        error!("Could not send the event to the daemon: {e}");
                        trace!("The connection will be closed regardles");
                    }
                    break;
                }
                ClientCommand::Playlist(cmd) => match cmd {
                    PlaylistCommand::New { name, tracks } => {
                        if let Err(e) = create_playlist(name, &tracks) {
                            if let Err(DaemonError::SendingError) = respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            ) {
                                break;
                            }
                            continue;
                        }
                        if let Err(DaemonError::SendingError) =
                            respond(&mut conn, &DaemonResponse::Ok)
                        {
                            break;
                        }
                    }
                    PlaylistCommand::FromM3U8 { name, m3u8_file } => {
                        if let Err(e) = import_playlist_from_m3u8(name, &m3u8_file) {
                            if let Err(DaemonError::SendingError) = respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            ) {
                                break;
                            }
                            continue;
                        }
                        if let Err(DaemonError::SendingError) =
                            respond(&mut conn, &DaemonResponse::Ok)
                        {
                            break;
                        }
                    }
                    PlaylistCommand::Delete { names } => {
                        for name in names {
                            if let Err(e) = delete_playlist(&name, false)
                                && let Err(DaemonError::SendingError) = respond(
                                    &mut conn,
                                    &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                                )
                            {
                                break;
                            }
                        }
                        if let Err(e) = save_library() {
                            if let Err(DaemonError::SendingError) = respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            ) {
                                break;
                            }
                            continue;
                        }
                        if let Err(DaemonError::SendingError) =
                            respond(&mut conn, &DaemonResponse::Ok)
                        {
                            break;
                        }
                    }
                    PlaylistCommand::AddTracks { name, tracks } => {
                        match add_tracks(&name, tracks) {
                            Ok(_) => {
                                if let Err(DaemonError::SendingError) =
                                    respond(&mut conn, &DaemonResponse::Ok)
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                if let Err(DaemonError::SendingError) = respond(
                                    &mut conn,
                                    &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                                ) {
                                    break;
                                }
                            }
                        }
                    }
                    PlaylistCommand::RemoveTracks { name, ids } => {
                        match remove_tracks(&name, ids) {
                            Ok(_) => {
                                if let Err(DaemonError::SendingError) =
                                    respond(&mut conn, &DaemonResponse::Ok)
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                if let Err(DaemonError::SendingError) = respond(
                                    &mut conn,
                                    &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                                ) {
                                    break;
                                }
                            }
                        }
                    }
                    PlaylistCommand::List => match get_library() {
                        Ok(lib) => {
                            if let Err(DaemonError::SendingError) =
                                respond(&mut conn, &DaemonResponse::Library(lib.into()))
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            if let Err(DaemonError::SendingError) = respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            ) {
                                break;
                            }
                        }
                    },
                },
                ClientCommand::Cache(cmd) => {
                    match music_lib::load(LoadMode::from_cache_command(cmd)) {
                        Ok(_) => {
                            if let Err(DaemonError::SendingError) =
                                respond(&mut conn, &DaemonResponse::Ok)
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            if let Err(DaemonError::SendingError) = respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            ) {
                                break;
                            }
                        }
                    }
                }
                ClientCommand::EventStream => {
                    trace!("Sending initial events to the client");
                    match get_library() {
                        Ok(lib) => {
                            if let Err(DaemonError::SendingError) = respond(
                                &mut conn,
                                &DaemonResponse::Event(DaemonEvent::MusicLibraryChanged(
                                    lib.into(),
                                )),
                            ) {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Could not get the contents of the music library: {e}");
                        }
                    }
                    let events = match playback::get_initial_events() {
                        Ok(events) => events,
                        Err(e) => {
                            error!("Could not get the initial events: {e}");
                            continue;
                        }
                    };
                    for event in events {
                        if let Err(DaemonError::SendingError) =
                            respond(&mut conn, &DaemonResponse::Event(event))
                        {
                            return;
                        }
                    }
                    trace!("Streaming daemon events to the client");
                    loop {
                        match trx.recv() {
                            Ok(event) => {
                                trace!(
                                    "Received an event from the daemon, realaying it to the client"
                                );
                                if let Err(DaemonError::SendingError) =
                                    respond(&mut conn, &DaemonResponse::Event(event))
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                trace!("Could not receive an event from the daemon: {e}");
                            }
                        }
                    }
                }
                ClientCommand::Disconnect => {
                    trace!("Thread {index} closing client connection (client request)");
                    if let Err(DaemonError::SendingError) = respond(&mut conn, &DaemonResponse::Ok)
                    {
                        break;
                    }
                    let _ = send_event(DaemonEvent::ConnectionClosed);
                    break;
                }
                ClientCommand::Ping => {
                    trace!("Got a ping command from the client, responding with pong");
                    if let Err(DaemonError::SendingError) =
                        respond(&mut conn, &DaemonResponse::Pong)
                    {
                        break;
                    }
                }
                ClientCommand::Playback(cmd) => {
                    let cmd: playback::Command = match cmd.try_into() {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            error!("Could not parse the playback command: {e}");
                            if let Err(DaemonError::SendingError) = respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::MusicLibraryError(e)),
                            ) {
                                break;
                            }
                            continue;
                        }
                    };
                    match playback::run_command(cmd) {
                        Ok(response) => {
                            if let Err(DaemonError::SendingError) =
                                respond(&mut conn, &DaemonResponse::Playback(response))
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Could not execute the playback command: {e}");
                            if let Err(DaemonError::SendingError) = respond(
                                &mut conn,
                                &DaemonResponse::Error(DaemonError::PlaybackError(e)),
                            ) {
                                break;
                            }
                        }
                    }
                }
            };
        }
        trace!("Thread {index} finished handling the client connection")
    })
}
