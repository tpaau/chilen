use std::{
    io::{BufReader, Write, stdin, stdout},
    thread,
};

use bincode::{config::standard, decode_from_reader, encode_to_vec};
use log::{error, info, trace};
use mpipc::{ClientCommand, DaemonError, DaemonResponse, Playlist, connect_to_daemon};

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
    // TODO: Prompt timeout

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
    if let DaemonError::ConnectionError { error: _ } = error {
        eprintln!("You must first start the daemon to run this command!");
    }
}

fn event_stream() -> Result<(), ()> {
    let conn = match connect_to_daemon() {
        Ok(conn) => conn,
        Err(e) => {
            trace!("{e}");
            return Err(());
        }
    };

    let cmd = match encode_to_vec(ClientCommand::EventStream, standard()) {
        Ok(command) => command,
        Err(e) => {
            error!("Could not encode the client command: {e}");
            return Err(());
        }
    };

    let mut buf = BufReader::new(&conn);

    if let Err(e) = buf.get_mut().write_all(&cmd) {
        error!("Could not send the command to the daemon: {e}");
        return Err(());
    }

    loop {
        let response: DaemonResponse = match decode_from_reader(&mut buf, standard()) {
            Ok(response) => response,
            Err(e) => {
                trace!("{e}");
                return Err(());
            }
        };

        println!("{response:?}");
    }
}

pub fn run_cli_command(command: Option<Command>) -> Result<(), ()> {
    if let Some(command) = command {
        match command {
            Command::Daemon { command } => {
                if command == DaemonCommand::Start {
                    match daemon::start() {
                        Ok(_) => info!("Daemon exited"),
                        Err(e) => {
                            error!("Daemon failed: {e}");
                            return Err(());
                        }
                    }
                } else if command == DaemonCommand::EventStream {
                    return event_stream();
                } else {
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
            }
            #[cfg(feature = "gui")]
            Command::Gui { command } => {
                if let GuiCommand::Start = command {
                    match gui::start() {
                        Ok(status) => info!("GUI exited: {status}"),
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
                        DaemonResponse::Playlists(playlists) => {
                            display_playlists(&playlists, full, debug);
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

        let handle = thread::spawn(|| match daemon::start() {
            Ok(_) => info!("Daemon exited"),
            Err(e) => error!("Daemon failed: {e}"),
        });

        #[cfg(feature = "gui")]
        {
            trace!("Starting GUI");
            match gui::start() {
                Ok(status) => info!("GUI exited: {status}"),
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
