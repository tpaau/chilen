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
    music_lib::{
        self, add_tracks,
        covers::LoadMode,
        create_playlist, delete_playlists, import_playlist_from_m3u8, remove_tracks,
        state::{get_library, save_library},
    },
    playback, raise, set_fullscreen, subscribe_to_events,
};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ThreadCommand {
    Raise,
    SetFullscreen(bool),
    Quit,
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
                Command::Quit => {
                    info!("Received quit command from the client");
                    trace!("Closing client connection (quit)");
                    let guard = crate::CONFIG.read().unwrap();
                    if !guard.as_ref().unwrap().can_quit {
                        error!("{}", crate::Error::QuitDisabled);
                        if let Err(chilen_ipc::Error::SendingError) =
                            respond(&mut conn, &Response::Error(chilen_ipc::Error::QuitDisabled))
                        {
                            break;
                        }
                        continue;
                    } else {
                        if let Err(chilen_ipc::Error::SendingError) =
                            respond(&mut conn, &Response::Ok)
                        {
                            break;
                        }
                    }
                    if let Err(e) = crate::send_command(ThreadCommand::Quit) {
                        let _ =
                            respond(&mut conn, &Response::Error(chilen_ipc::Error::QuitDisabled));
                        error!("{e}");
                        trace!("The connection will be closed regardless");
                    }
                    break;
                }
                Command::Library(cmd) => match cmd {
                    LibraryCommand::NewPlaylist { name, tracks } => {
                        if let Err(e) = create_playlist(name, &tracks) {
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Error(e))
                            {
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
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Error(e))
                            {
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
                            && let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Error(e))
                        {
                            break;
                        }
                        if let Err(e) = save_library() {
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Error(e))
                            {
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
                                if let Err(chilen_ipc::Error::SendingError) =
                                    respond(&mut conn, &Response::Error(e))
                                {
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
                                if let Err(chilen_ipc::Error::SendingError) =
                                    respond(&mut conn, &Response::Error(e))
                                {
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
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Error(e))
                            {
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
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Error(e))
                            {
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
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Error(e))
                            {
                                break;
                            }
                        }
                    },
                },
                Command::EventStream => {
                    trace!("Streaming daemon events to the client");
                    let stream = subscribe_to_events();
                    if let Err(e) = subscribe_to_events()
                        && let Err(chilen_ipc::Error::SendingError) =
                            respond(&mut conn, &Response::Error(e))
                    {
                        break;
                    }
                    let stream = stream.unwrap();
                    loop {
                        match stream.recv() {
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
                    break;
                }
                Command::Disconnect => {
                    trace!("Thread {index} closing client connection (client request)");
                    if let Err(chilen_ipc::Error::SendingError) = respond(&mut conn, &Response::Ok)
                    {
                        break;
                    }
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
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Error(e))
                            {
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
                            if let Err(chilen_ipc::Error::SendingError) =
                                respond(&mut conn, &Response::Error(e))
                            {
                                break;
                            }
                        }
                    }
                }
                Command::CanRaise => {
                    let guard = crate::CONFIG.read().unwrap();
                    if let Err(chilen_ipc::Error::SendingError) = respond(
                        &mut conn,
                        &Response::CanRaise(guard.as_ref().unwrap().can_raise),
                    ) {
                        break;
                    }
                }
                Command::Raise => {
                    let response = match raise() {
                        Ok(_) => Response::Ok,
                        Err(_) => Response::Error(chilen_ipc::Error::RaiseDisabled),
                    };
                    if let Err(chilen_ipc::Error::SendingError) = respond(&mut conn, &response) {
                        break;
                    }
                }
                Command::CanSetFullscreen => {
                    let guard = crate::CONFIG.read().unwrap();
                    if let Err(chilen_ipc::Error::SendingError) = respond(
                        &mut conn,
                        &Response::CanSetFullscreen(guard.as_ref().unwrap().can_set_fullscreen),
                    ) {
                        break;
                    }
                }
                Command::SetFullscreen(fullscreen) => {
                    let response = match set_fullscreen(fullscreen) {
                        Ok(_) => Response::Ok,
                        Err(_) => Response::Error(chilen_ipc::Error::SetFullscreenDisabled),
                    };
                    if let Err(chilen_ipc::Error::SendingError) = respond(&mut conn, &response) {
                        break;
                    }
                }
                Command::CanQuit => {
                    let guard = crate::CONFIG.read().unwrap();
                    if let Err(chilen_ipc::Error::SendingError) = respond(
                        &mut conn,
                        &Response::CanQuit(guard.as_ref().unwrap().can_quit),
                    ) {
                        break;
                    }
                }
            };
        }
        trace!("Thread {index} finished handling the client connection")
    })
}
