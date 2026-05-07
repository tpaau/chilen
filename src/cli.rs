use std::{
    env::home_dir,
    io::{BufReader, Write},
    thread,
};

use chilen_daemon::{AddrClaimMode, playback};
use chilen_ipc::{Error, Response, SocketType, connect, library::Playlist, send_command};
use clap::crate_name;
use log::{error, info, trace};

use crate::argparse::{Command, DaemonCommand, LibraryCommand, PlaylistCommand};

#[cfg(feature = "gui")]
use crate::{argparse::GuiCommand, gui};

pub const SOCKET_NAME_HEADLESS: &str = "CHILEN_HEADLESS.socket";

pub const IDENTITY_HEADLESS: &str = "Chilen daemon";

#[cfg(feature = "gui")]
pub const IDENTITY_GUI: &str = "Chilen";

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

fn print_daemon_error(error: Error) {
    if let Error::ConnectionError = error {
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
    let mut conn = match connect(socket_name, socket_type) {
        Ok(conn) => BufReader::new(conn),
        Err(e) => {
            error!("Could not start the event stream: {e}");
            return Err(());
        }
    };

    let command = match chilen_ipc::serialize_command(&chilen_ipc::Command::EventStream) {
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
        let response: Response = match chilen_ipc::receive_response(&mut conn) {
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
                    let config = chilen_daemon::Config {
                        cache_dir,
                        data_dir,
                        music_dir,
                        socket_name,
                        addr_claim_mode: AddrClaimMode::default(),
                        socket_type,
                        can_raise: false,
                        playback_config: playback::Config {
                            #[cfg(feature = "mpris")]
                            identity: String::from(IDENTITY_HEADLESS),
                            #[cfg(feature = "mpris")]
                            bus_name_suffix: format!("com.dev.{}", crate_name!()),
                            allow_rate_modification,
                        },
                    };
                    match chilen_daemon::start(config) {
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
                    match send_command(chilen_ipc::Command::Ping, &socket_name, &socket_type) {
                        Ok(response) => println!("{response:?}"),
                        Err(e) => println!("{e}"),
                    }
                }
                DaemonCommand::Stop => {
                    match chilen_ipc::send_command(
                        chilen_ipc::Command::Shutdown,
                        &socket_name,
                        &socket_type,
                    ) {
                        Ok(_) => println!("Ok"),
                        Err(e) => {
                            print_daemon_error(e);
                            return Err(());
                        }
                    }
                }
                DaemonCommand::GetCanRaise => {
                    match chilen_ipc::send_command(
                        chilen_ipc::Command::GetCanRaise,
                        &socket_name,
                        &socket_type,
                    ) {
                        Ok(response) => match response {
                            Response::CanRaise(can_raise) => println!("{can_raise}"),
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
                DaemonCommand::Raise => {
                    match chilen_ipc::send_command(
                        chilen_ipc::Command::Raise,
                        &socket_name,
                        &socket_type,
                    ) {
                        Ok(response) => match response {
                            Response::Ok => println!("Ok"),
                            Response::Error(e) => {
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
            Command::Lib { command } => {
                let (full, debug) = match command {
                    LibraryCommand::Playlist {
                        command: PlaylistCommand::List { full, debug },
                    } => (full, debug),
                    _ => (false, false),
                };
                match chilen_ipc::send_command(
                    chilen_ipc::Command::Library(command.into()),
                    &socket_name,
                    &socket_type,
                ) {
                    Ok(response) => match response {
                        Response::Ok => {
                            println!("Ok");
                        }
                        Response::Library(lib) => {
                            display_playlists(&lib.playlists, full, debug);
                        }
                        Response::Error(e) => {
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
                let cmd = chilen_ipc::Command::Playback(command.into());
                match chilen_ipc::send_command(cmd, &socket_name, &socket_type) {
                    Ok(response) => match response {
                        Response::Ok => println!("Ok"),
                        Response::Playback(response) => println!("{response}"),
                        Response::Error(e) => {
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
        trace!("No command specified, starting a daemon with GUI");

        #[cfg(not(feature = "gui"))]
        trace!("No command specified, starting the daemon");

        #[cfg(feature = "gui")]
        let identity = IDENTITY_GUI.to_string();

        #[cfg(not(feature = "gui"))]
        let identity = IDENTITY_HEADLESS.to_string();

        let conf = match chilen_daemon::Config::try_from_name(&identity, &socket_name) {
            Ok(conf) => conf,
            Err(e) => {
                error!("Could not create a config for the daemon: {e}");
                return Err(());
            }
        };
        let handle = thread::spawn(|| match chilen_daemon::start(conf) {
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
