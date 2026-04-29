use std::{
    io::{BufReader, Write},
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

use interprocess::local_socket::Stream;
use log::{error, info, trace, warn};

use mpipc::{Command, Error, Response, library::LibraryCommand};
use rmp_serde::{Serializer, from_read};
use serde::Serialize;

use crate::{
    Event,
    data::music_lib::{
        self, add_tracks, create_playlist, delete_playlist, get_library, import_playlist_from_m3u8,
        remove_tracks, save_library,
    },
    playback, send_event,
};

fn respond(conn: &mut BufReader<&Stream>, msg: &Response) -> Result<(), Error> {
    let mut data = Vec::new();
    if let Err(e) = msg.serialize(&mut Serializer::new(&mut data)) {
        error!("Could not prepare the command for the client: {e}");
        return Err(Error::EncodingError);
    }

    match conn.get_mut().write_all(&data) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed sending the response to the client: {e}");
            warn!("The client likely crashed or was closed forcefully, dropping the connection");
            Err(Error::SendingError)
        }
    }
}

pub(crate) fn spawn(conn: Stream, trx: Receiver<Event>, index: u64) -> JoinHandle<()> {
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
                    if let Err(Error::SocketError) = respond(&mut conn, &Response::Ok) {
                        break;
                    }
                    if let Err(e) = send_event(Event::Shutdown) {
                        error!("Could not send the event to the daemon: {e}");
                        trace!("The connection will be closed regardles");
                    }
                    break;
                }
                Command::Library(cmd) => match cmd {
                    LibraryCommand::NewPlaylist { name, tracks } => {
                        if let Err(e) = create_playlist(name, &tracks) {
                            if let Err(Error::SendingError) =
                                respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                            {
                                break;
                            }
                            continue;
                        }
                        if let Err(Error::SendingError) = respond(&mut conn, &Response::Ok) {
                            break;
                        }
                    }
                    LibraryCommand::PlaylistFromM3U8 { name, m3u8_file } => {
                        if let Err(e) = import_playlist_from_m3u8(name, &m3u8_file) {
                            if let Err(Error::SendingError) =
                                respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                            {
                                break;
                            }
                            continue;
                        }
                        if let Err(Error::SendingError) = respond(&mut conn, &Response::Ok) {
                            break;
                        }
                    }
                    LibraryCommand::DeletePlaylist { names } => {
                        for name in names {
                            if let Err(e) = delete_playlist(&name, false)
                                && let Err(Error::SendingError) =
                                    respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                            {
                                break;
                            }
                        }
                        if let Err(e) = save_library() {
                            if let Err(Error::SendingError) =
                                respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                            {
                                break;
                            }
                            continue;
                        }
                        if let Err(Error::SendingError) = respond(&mut conn, &Response::Ok) {
                            break;
                        }
                    }
                    LibraryCommand::AddTracksToPlaylist { name, tracks } => {
                        match add_tracks(&name, tracks) {
                            Ok(_) => {
                                if let Err(Error::SendingError) = respond(&mut conn, &Response::Ok)
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                if let Err(Error::SendingError) =
                                    respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                                {
                                    break;
                                }
                            }
                        }
                    }
                    LibraryCommand::RemoveTracksFromPlaylist { name, ids } => {
                        match remove_tracks(&name, ids) {
                            Ok(_) => {
                                if let Err(Error::SendingError) = respond(&mut conn, &Response::Ok)
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                if let Err(Error::SendingError) =
                                    respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                                {
                                    break;
                                }
                            }
                        }
                    }
                    LibraryCommand::GetLibrary => match get_library() {
                        Ok(lib) => {
                            if let Err(Error::SendingError) =
                                respond(&mut conn, &Response::Library(lib.into()))
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            if let Err(Error::SendingError) =
                                respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                            {
                                break;
                            }
                        }
                    },
                    LibraryCommand::Reload => match music_lib::load(false) {
                        Ok(_) => {
                            if let Err(Error::SendingError) = respond(&mut conn, &Response::Ok) {
                                break;
                            }
                        }
                        Err(e) => {
                            if let Err(Error::SendingError) =
                                respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                            {
                                break;
                            }
                        }
                    },
                    LibraryCommand::Rebuild => match music_lib::load(true) {
                        Ok(_) => {
                            if let Err(Error::SendingError) = respond(&mut conn, &Response::Ok) {
                                break;
                            }
                        }
                        Err(e) => {
                            if let Err(Error::SendingError) =
                                respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                            {
                                break;
                            }
                        }
                    },
                },
                Command::EventStream => {
                    trace!("Sending initial events to the client");
                    match get_library() {
                        Ok(lib) => {
                            if let Err(Error::SendingError) = respond(
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
                    let events = match playback::get_initial_events() {
                        Ok(events) => events,
                        Err(e) => {
                            error!("Could not get the initial events: {e}");
                            continue;
                        }
                    };
                    for event in events {
                        if let Err(Error::SendingError) =
                            respond(&mut conn, &Response::Event(event))
                        {
                            return;
                        }
                    }
                    trace!("Streaming daemon events to the client");
                    loop {
                        match trx.recv() {
                            Ok(event) => {
                                if let Err(Error::SendingError) =
                                    respond(&mut conn, &Response::Event(event))
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
                Command::Disconnect => {
                    trace!("Thread {index} closing client connection (client request)");
                    if let Err(Error::SendingError) = respond(&mut conn, &Response::Ok) {
                        break;
                    }
                    let _ = send_event(Event::ConnectionClosed);
                    break;
                }
                Command::Ping => {
                    trace!("Got a ping command from the client, responding with pong");
                    if let Err(Error::SendingError) = respond(&mut conn, &Response::Pong) {
                        break;
                    }
                }
                Command::Playback(cmd) => {
                    let cmd: playback::Command = match cmd.try_into() {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            error!("Could not parse the playback command: {e}");
                            if let Err(Error::SendingError) =
                                respond(&mut conn, &Response::Error(Error::LibraryError(e)))
                            {
                                break;
                            }
                            continue;
                        }
                    };
                    match playback::run_command(cmd) {
                        Ok(response) => {
                            if let Err(Error::SendingError) = respond(&mut conn, &response) {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Could not execute the playback command: {e}");
                            if let Err(Error::SendingError) =
                                respond(&mut conn, &Response::Error(Error::PlaybackError(e)))
                            {
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
