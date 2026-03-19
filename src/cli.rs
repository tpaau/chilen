use std::{
    env::home_dir,
    io::{BufReader, Write, stdin, stdout},
    thread,
};

use clap::crate_name;
use log::{error, info, trace};
use mpipc::{ClientCommand, DaemonError, DaemonResponse, Playlist, connect_to_daemon};
use rmp_serde::from_read;
use serde::Serialize;

use crate::argparse::{Command, DaemonCommand, PlaylistCommand};

#[cfg(feature = "gui")]
use crate::{argparse::GuiCommand, gui};

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

/// Asks the user the given question and returns their response:
/// - `true` -> Users' answer was positive (y/yes)
/// - `false` -> Users' answer was negative (n/no)
///
/// Returns an error if the users' input was invalid.
///
/// Valid input is either 'y' or 'yes' for yes and 'n' or 'no' for no.
///
/// Users' input is converted to lowercase internally so it doesn't matter
/// whether they respond with all uppercase, all lowercase, or mixed letters.
fn ask_user_yn(prompt: &str, default: bool) -> Result<bool, std::io::Error> {
    let mut input = String::new();

    if default {
        print!("{prompt} [Yes/no]: ");
    } else {
        print!("{prompt} [No/yes]: ");
    }

    let _ = stdout().flush();

    match stdin().read_line(&mut input) {
        Ok(_) => {
            let il = input.trim().to_lowercase();
            if il.is_empty() {
                Ok(default)
            } else if il == "y" || il == "yes" {
                Ok(true)
            } else if il == "n" || il == "no" {
                Ok(false)
            } else {
                error!("Invalid input: '{}'", input.trim());
                ask_user_yn(prompt, default)
            }
        }
        Err(e) => Err(e),
    }
}

fn print_daemon_error(error: DaemonError) {
    error!("Could not connect to the daemon: {error}");
    if let DaemonError::ConnectionError = error {
        eprintln!("You must first start the daemon to run this command!");
    }
}

fn event_stream(json: bool, pretty: bool) -> Result<(), ()> {
    let mut conn = match connect_to_daemon() {
        Ok(conn) => BufReader::new(conn),
        Err(e) => {
            error!("Could not start the event stream: {e}");
            return Err(());
        }
    };

    let mut data = Vec::new();
    if let Err(e) = ClientCommand::EventStream.serialize(&mut rmp_serde::Serializer::new(&mut data))
    {
        error!("Could not encode the client command: {e}");
        return Err(());
    }

    if let Err(e) = conn.get_mut().write_all(&data) {
        error!("Could not send the command to the daemon: {e}");
        return Err(());
    }

    loop {
        let response: DaemonResponse = match from_read(&mut conn) {
            Ok(response) => response,
            Err(e) => {
                error!("Failed decoding a daemon response: {e}");
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

pub fn run_cli_command(command: Option<Command>) -> Result<(), ()> {
    if let Some(command) = command {
        match command {
            Command::Daemon { command } => match command {
                DaemonCommand::Start {
                    cache_dir,
                    data_dir,
                    music_dir,
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
                    let config = daemon::Config::new(cache_dir, data_dir, music_dir);
                    match daemon::start(config) {
                        // TODO: Error handling and actual config
                        Ok(_) => info!("Daemon exited"),
                        Err(e) => {
                            error!("Daemon failed: {e}");
                            return Err(());
                        }
                    }
                }
                DaemonCommand::EventStream { json, pretty_json } => {
                    return event_stream(json || pretty_json, pretty_json);
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
                    match mpipc::exec_client_command(cmd) {
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
                match mpipc::exec_client_command(mpipc::ClientCommand::Playlist(command.into())) {
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
                match mpipc::exec_client_command(cmd) {
                    Ok(response) => match response {
                        DaemonResponse::Ok => {
                            println!("Ok");
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
        }
    } else {
        #[cfg(feature = "gui")]
        trace!("No command specified, starting a deamon with GUI");

        #[cfg(not(feature = "gui"))]
        trace!("No command specified, starting a deamon");

        let handle =
            thread::spawn(
                || match daemon::start(daemon::Config::try_default().unwrap()) {
                    // TODO: Error handling and actual config
                    Ok(_) => info!("Daemon exited"),
                    Err(e) => error!("Daemon failed: {e}"),
                },
            );

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
