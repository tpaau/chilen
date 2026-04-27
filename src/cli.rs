use std::{
    env::home_dir,
    io::{BufReader, Write},
    thread,
};

use clap::crate_name;
use daemon::{AddrClaimMode, playback};
use log::{error, info, trace};
use mpipc::{
    ClientCommand, DaemonError, DaemonResponse, Playlist, SocketType, connect_to_daemon,
    exec_client_command,
};

use crate::argparse::{Command, DaemonCommand, PlaylistCommand};

#[cfg(feature = "gui")]
use crate::{argparse::GuiCommand, gui};

pub const SOCKET_NAME_HEADLESS: &str = "MUSIC_PLAYER_HEADLESS.socket";

pub const IDENTITY_HEADLESS: &str = "Prototype music player daemon";

#[cfg(feature = "gui")]
pub const IDENTITY_GUI: &str = "Prototype music player user interface";

// #[cfg(feature = "gui")]
// pub const SOCKET_NAME_GUI: &str = "MUSIC_PLAYER_GUI.socket";

fn display_playlists(playlists: &Vec<Playlist>, full: bool, debug: bool) {
    if debug {
        println!("{playlists:?}");
    } else if playlists.is_empty() {
        println!("There are no playlists in the library");
    } else if full {
        for playlist in playlists {
            println!("Playlist \"{}\":", playlist.name);
            for (i, track) in playlist.tracks.iter().enumerate() {
                println!("  {}: {track}", i + 1);
            }
        }
    } else {
        for playlist in playlists {
            println!("{} ({} tracks)", playlist.name, playlist.tracks.len());
        }
    }
}

fn print_daemon_error(error: DaemonError) {
    if let DaemonError::ConnectionError = error {
        error!("Could not connect to the daemon (is the daemon running?)");
        eprintln!("You must first start the daemon to run this command");
    } else {
        error!("Could not connect to the daemon: {error}");
    }
}

fn event_stream(
    json: bool,
    pretty: bool,
    socket_name: &str,
    socket_type: &SocketType,
) -> Result<(), ()> {
    let mut conn = match connect_to_daemon(socket_name, socket_type) {
        Ok(conn) => BufReader::new(conn),
        Err(e) => {
            error!("Could not start the event stream: {e}");
            return Err(());
        }
    };

    let command = match mpipc::serialize_client_command(ClientCommand::EventStream) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Could not encode the client command: {e}");
            return Err(());
        }
    };

    if let Err(e) = conn.get_mut().write_all(&command) {
        error!("Could not send the command to the daemon: {e}");
        return Err(());
    }

    loop {
        let response: DaemonResponse = match mpipc::receive_daemon_response(&mut conn) {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to receive a response from the daemon: {e}");
                return Err(());
            }
        };

        if json {
            let result = match pretty {
                true => serde_json::to_string_pretty(&response),
                false => serde_json::to_string(&response),
            };
            let data = match result {
                Ok(data) => data,
                Err(e) => {
                    error!("Could not serialize the response to JSON: {e}");
                    continue;
                }
            };
            println!("{data}");
        } else {
            println!("{response:?}");
        }
    }
}

pub fn run_cli_command(
    command: Option<Command>,
    socket_type: SocketType,
    socket_name: Option<String>,
) -> Result<(), ()> {
    let socket_name = socket_name.unwrap_or(SOCKET_NAME_HEADLESS.to_string());
    if let Some(command) = command {
        match command {
            Command::Daemon { command } => match command {
                DaemonCommand::Start {
                    cache_dir,
                    data_dir,
                    music_dir,
                    allow_rate_modification,
                } => {
                    let home = if cache_dir.is_none() || data_dir.is_none() || music_dir.is_none() {
                        match home_dir() {
                            Some(home) => Some(home),
                            None => {
                                error!("Could not get the path to the home directory");
                                return Err(());
                            }
                        }
                    } else {
                        None
                    };
                    let cache_dir = match cache_dir {
                        Some(cache) => cache,
                        None => {
                            let mut cache = home.clone().unwrap();
                            cache.push(".cache");
                            cache.push(crate_name!());
                            cache
                        }
                    };
                    let data_dir = match data_dir {
                        Some(data) => data,
                        None => {
                            let mut data = home.clone().unwrap();
                            data.push(".local/share");
                            data.push(crate_name!());
                            data
                        }
                    };
                    let music_dir = match music_dir {
                        Some(music) => music,
                        None => {
                            let mut music = home.clone().unwrap();
                            music.push("Music");
                            music
                        }
                    };
                    let config = daemon::Config {
                        cache_dir,
                        data_dir,
                        music_dir,
                        socket_name,
                        addr_claim_mode: AddrClaimMode::default(),
                        socket_type,
                        playback_config: playback::Config {
                            #[cfg(feature = "mpris")]
                            identity: String::from(IDENTITY_HEADLESS),
                            #[cfg(feature = "mpris")]
                            bus_name_suffix: format!("com.dev.{}", crate_name!()),
                            allow_rate_modification,
                        },
                    };
                    match daemon::start(config) {
                        Ok(_) => info!("Daemon exited"),
                        Err(e) => {
                            error!("Daemon failed: {e}");
                            return Err(());
                        }
                    }
                }
                DaemonCommand::EventStream { json, pretty_json } => {
                    return event_stream(
                        json || pretty_json,
                        pretty_json,
                        &socket_name,
                        &socket_type,
                    );
                }
                DaemonCommand::Ping => {
                    match exec_client_command(ClientCommand::Ping, &socket_name, &socket_type) {
                        Ok(response) => println!("{response:?}"),
                        Err(e) => println!("{e}"),
                    }
                }
                _ => {
                    let cmd: mpipc::DaemonCommand = command.into();
                    let cmd = match cmd.try_into() {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            error!("Could not send the command to the daemon: {e}");
                            return Err(());
                        }
                    };
                    match mpipc::exec_client_command(cmd, &socket_name, &socket_type) {
                        Ok(response) => info!("Got a response from the daemon: {response:?}"),
                        Err(e) => {
                            print_daemon_error(e);
                            return Err(());
                        }
                    }
                }
            },
            #[cfg(feature = "gui")]
            Command::Gui { command } => {
                if let GuiCommand::Start = command {
                    match gui::start() {
                        Ok(status) => info!("GUI exited: {status:?}"),
                        Err(e) => {
                            error!("GUI failed: {e}");
                            return Err(());
                        }
                    }
                } else {
                    panic!("GUI command execution not supported!");
                }
            }
            Command::Playlist { command } => {
                let (full, debug) = match command {
                    PlaylistCommand::List { full, debug } => (full, debug),
                    _ => (false, false),
                };
                match mpipc::exec_client_command(
                    mpipc::ClientCommand::Playlist(command.into()),
                    &socket_name,
                    &socket_type,
                ) {
                    Ok(response) => match response {
                        DaemonResponse::Ok => {
                            println!("Ok");
                        }
                        DaemonResponse::Library(lib) => {
                            display_playlists(&lib.playlists, full, debug);
                        }
                        DaemonResponse::Error(e) => {
                            error!("{e}");
                            return Err(());
                        }
                        _ => {
                            error!("Got an unexpected response from the daemon: {response:?}");
                            return Err(());
                        }
                    },
                    Err(e) => {
                        print_daemon_error(e);
                        return Err(());
                    }
                }
            }
            Command::Cache { command } => {
                let cmd = ClientCommand::Cache(command.into());
                match mpipc::exec_client_command(cmd, &socket_name, &socket_type) {
                    Ok(response) => match response {
                        DaemonResponse::Ok => println!("Ok"),
                        DaemonResponse::Error(e) => {
                            error!("{e}");
                            return Err(());
                        }
                        _ => {
                            error!("Got an unexpected response from the daemon: {response:?}");
                            return Err(());
                        }
                    },
                    Err(e) => {
                        print_daemon_error(e);
                        return Err(());
                    }
                }
            }
            Command::Playback { command } => {
                let cmd = ClientCommand::Playback(command.into());
                match mpipc::exec_client_command(cmd, &socket_name, &socket_type) {
                    Ok(response) => match response {
                        DaemonResponse::Playback(response) => {
                            println!("{response}");
                        }
                        DaemonResponse::Error(e) => {
                            error!("Playback command failed: {e}");
                            return Err(());
                        }
                        _ => {
                            error!("Got an unexpected response from the daemon: {response:?}");
                            return Err(());
                        }
                    },
                    Err(e) => {
                        print_daemon_error(e);
                        return Err(());
                    }
                }
            }
        }
    } else {
        #[cfg(feature = "gui")]
        trace!("No command specified, starting a deamon with GUI");

        #[cfg(not(feature = "gui"))]
        trace!("No command specified, starting the deamon");

        #[cfg(feature = "gui")]
        let identity = IDENTITY_GUI.to_string();

        #[cfg(not(feature = "gui"))]
        let identity = IDENTITY_HEADLESS.to_string();

        let conf = match daemon::Config::try_from_name(&identity, &socket_name) {
            Ok(conf) => conf,
            Err(e) => {
                error!("Could not create a config for the daemon: {e}");
                return Err(());
            }
        };
        let handle = thread::spawn(|| match daemon::start(conf) {
            Ok(_) => info!("Daemon exited"),
            Err(e) => error!("Daemon failed: {e}"),
        });

        #[cfg(feature = "gui")]
        {
            trace!("Starting GUI");
            match gui::start() {
                Ok(status) => info!("GUI exited: {status:?}"),
                Err(e) => {
                    error!("GUI failed: {e}");
                    return Err(());
                }
            };
        }

        handle.join().unwrap();
    }
    Ok(())
}
