use std::{
    io::{BufReader, Write},
    thread::{self, JoinHandle},
};

use interprocess::local_socket::Stream;
use log::{error, info, trace, warn};

use chilen_ipc::{Command, Response, library::LibraryCommand};
use rmp_serde::{Serializer, from_read};
use serde::Serialize;

use crate::{
    Event,
    music_lib::{
        self, add_tracks,
        covers::LoadMode,
        create_playlist, delete_playlists, import_playlist_from_m3u8, remove_tracks,
        state::{get_library, save_library},
    },
    playback, send_event, subscribe_to_events,
};

pub(super) enum ThreadCommand {
    Shutdown,
}

fn respond(conn: &mut BufReader<&Stream>, msg: &Response) -> Result<(), chilen_ipc::Error> {
    let mut data = Vec::new();
    if let Err(e) = msg.serialize(&mut Serializer::new(&mut data)) {
        error!("Could not prepare the command for the client: {e}");
        return Err(chilen_ipc::Error::EncodingError);
    }

    match conn.get_mut().write_all(&data) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed sending the response to the client: {e}");
            warn!("The client likely crashed or was closed forcefully, dropping the connection");
            Err(chilen_ipc::Error::SendingError)
        }
    }
}

pub(crate) fn spawn(conn: Stream, index: u64) -> JoinHandle<()> {
    thread::spawn(move || {
        trace!("Handling client connection");
        loop {
            let mut conn = BufReader::new(&conn);

            let command: Command = match from_read(&mut conn) {
                Ok(cmd) => cmd,
                Err(e) => {
                    error!("Failed decoding a client command: {e}");
                    break;
                }
            };

            match command {
                Command::Shutdown => {
                    info!("Received shutdown command from the client");
                    trace!("Closing client connection (shutdown)");
                    if let Err(chilen_ipc::Error::SendingError) = respond(&mut conn, &Response::Ok)
                    {
                        break;
                    }
                    if let Err(e) = crate::send_command(ThreadCommand::Shutdown) {
                        error!("Could not send the command to the daemon: {e}");
                        trace!("The connection will be closed regardless");
                    }
                    break;
                }
                Command::Library(cmd) => match cmd {
                    LibraryCommand::NewPlaylist { name, tracks } => {
                        if let Err(e) = create_playlist(name, &tracks) {
                            if let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Error(chilen_ipc::Error::LibraryError(e)),
                            ) {
                                break;
                            }
                            continue;
                        }
                        if let Err(chilen_ipc::Error::SendingError) =
                            respond(&mut conn, &Response::Ok)
                        {
                            break;
                        }
                    }
                    LibraryCommand::PlaylistFromM3U8 { name, m3u8_file } => {
                        if let Err(e) = import_playlist_from_m3u8(name, &m3u8_file) {
                            if let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Error(chilen_ipc::Error::LibraryError(e)),
                            ) {
                                break;
                            }
                            continue;
                        }
                        if let Err(chilen_ipc::Error::SendingError) =
                            respond(&mut conn, &Response::Ok)
                        {
                            break;
                        }
                    }
                    LibraryCommand::DeletePlaylists { names } => {
                        if let Err(e) = delete_playlists(names)
                            && let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Error(chilen_ipc::Error::LibraryError(e)),
                            )
                        {
                            break;
                        }
                        if let Err(e) = save_library() {
                            if let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Error(chilen_ipc::Error::LibraryError(e)),
                            ) {
                                break;
                            }
                            continue;
                        }
                        if let Err(chilen_ipc::Error::SendingError) =
                            respond(&mut conn, &Response::Ok)
                        {
                            break;
                        }
                    }
                    LibraryCommand::AddTracksToPlaylist { name, tracks } => {
                        match add_tracks(&name, tracks) {
                            Ok(_) => {
                                if let Err(chilen_ipc::Error::SendingError) =
                                    respond(&mut conn, &Response::Ok)
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                if let Err(chilen_ipc::Error::SendingError) = respond(
                                    &mut conn,
                                    &Response::Error(chilen_ipc::Error::LibraryError(e)),
                                ) {
                                    break;
                                }
                            }
                        }
                    }
                    LibraryCommand::RemoveTracksFromPlaylist { name, ids } => {
                        match remove_tracks(&name, ids) {
                            Ok(_) => {
                                if let Err(chilen_ipc::Error::SendingError) =
                                    respond(&mut conn, &Response::Ok)
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                if let Err(chilen_ipc::Error::SendingError) = respond(
                                    &mut conn,
                                    &Response::Error(chilen_ipc::Error::LibraryError(e)),
                                ) {
                                    break;
                                }
                            }
                        }
                    }
                    LibraryCommand::GetLibrary => match get_library() {
                        Ok(lib) => {
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Library(lib.into()))
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            if let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Error(chilen_ipc::Error::LibraryError(e)),
                            ) {
                                break;
                            }
                        }
                    },
                    LibraryCommand::Reload => match music_lib::state::load(LoadMode::Load) {
                        Ok(_) => {
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Ok)
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            if let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Error(chilen_ipc::Error::LibraryError(e)),
                            ) {
                                break;
                            }
                        }
                    },
                    LibraryCommand::Rebuild => match music_lib::state::load(LoadMode::Rebuild) {
                        Ok(_) => {
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Ok)
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            if let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Error(chilen_ipc::Error::LibraryError(e)),
                            ) {
                                break;
                            }
                        }
                    },
                },
                Command::EventStream => {
                    trace!("Sending initial events to the client");
                    match get_library() {
                        Ok(lib) => {
                            if let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Event(Event::LibraryChanged(lib.into())),
                            ) {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Could not get the contents of the music library: {e}");
                        }
                    }
                    let mut events = match playback::get_initial_events() {
                        Ok(events) => events,
                        Err(e) => {
                            error!("Could not get the initial events: {e}");
                            continue;
                        }
                    };
                    let guard = crate::CONFIG.read().unwrap();
                    events.push(Event::CanRaiseChanged(guard.as_ref().unwrap().can_raise));
                    for event in events {
                        if let Err(chilen_ipc::Error::SendingError) =
                            respond(&mut conn, &Response::Event(event))
                        {
                            return;
                        }
                    }
                    trace!("Streaming daemon events to the client");
                    loop {
                        match subscribe_to_events().recv() {
                            Ok(event) => {
                                if let Err(chilen_ipc::Error::SendingError) =
                                    respond(&mut conn, &Response::Event(event))
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                trace!("Could not receive an event from the daemon: {e}");
                                break;
                            }
                        }
                    }
                }
                Command::Disconnect => {
                    trace!("Thread {index} closing client connection (client request)");
                    if let Err(chilen_ipc::Error::SendingError) = respond(&mut conn, &Response::Ok)
                    {
                        break;
                    }
                    let _ = send_event(Event::ConnectionClosed);
                    break;
                }
                Command::Ping => {
                    trace!("Got a ping command from the client, responding with pong");
                    if let Err(chilen_ipc::Error::SendingError) =
                        respond(&mut conn, &Response::Pong)
                    {
                        break;
                    }
                }
                Command::Playback(cmd) => {
                    let cmd: playback::Command = match cmd.try_into() {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            error!("Could not parse the playback command: {e}");
                            if let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Error(chilen_ipc::Error::LibraryError(e)),
                            ) {
                                break;
                            }
                            continue;
                        }
                    };
                    match playback::run_command(cmd) {
                        Ok(response) => {
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &response)
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Could not execute the playback command: {e}");
                            if let Err(chilen_ipc::Error::SendingError) = respond(
                                &mut conn,
                                &Response::Error(chilen_ipc::Error::PlaybackError(e)),
                            ) {
                                break;
                            }
                        }
                    }
                }
                Command::GetCanRaise => {
                    let guard = crate::CONFIG.read().unwrap();
                    if let Err(chilen_ipc::Error::SendingError) = respond(
                        &mut conn,
                        &Response::CanRaise(guard.as_ref().unwrap().can_raise),
                    ) {
                        break;
                    }
                }
                Command::Raise => {
                    let guard = crate::CONFIG.read().unwrap();
                    if guard.as_ref().unwrap().can_raise {
                        let _ = send_event(Event::RaiseRequested);
                        if let Err(chilen_ipc::Error::SendingError) =
                            respond(&mut conn, &Response::Ok)
                        {
                            break;
                        }
                    } else {
                        if let Err(chilen_ipc::Error::SendingError) = respond(
                            &mut conn,
                            &Response::Error(chilen_ipc::Error::RaiseNotSupported),
                        ) {
                            break;
                        }
                    }
                }
            };
        }
        trace!("Thread {index} finished handling the client connection")
    })
}
