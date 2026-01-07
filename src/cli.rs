use std::{
    io::{Write, stdin, stdout},
    thread,
};

use log::{error, info, trace};
use mpipc::{DaemonError, DaemonResponse, Playlist};

use crate::{
    argparse::{Command, DaemonCommand, GuiCommand, PlaylistCommand},
    daemon, gui,
};

fn display_playlists(playlists: &Vec<Playlist>, full: bool) {
    if playlists.is_empty() {
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

pub async fn run_cli_command(command: Option<Command>) -> Result<(), ()> {
    if let Some(command) = command {
        match command {
            Command::Daemon { command } => {
                if let DaemonCommand::Start = command {
                    match daemon::start().await {
                        Ok(status) => info!("Daemon exited: {status}"),
                        Err(e) => error!("Daemon failed: {e}"),
                    }
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
                        Ok(response) => info!("Got a response from the daemon: {response}"),
                        Err(e) => print_daemon_error(e),
                    }
                }
            }
            Command::Gui { command } => {
                if let GuiCommand::Start = command {
                    match gui::start() {
                        Ok(status) => info!("GUI exited: {status}"),
                        Err(e) => error!("GUI failed: {e}"),
                    }
                } else {
                    panic!("GUI command execution not supported!");
                }
            }
            Command::Playlist { command } => {
                let full = match command {
                    PlaylistCommand::List { full } => full,
                    _ => false,
                };
                match mpipc::exec_client_command(mpipc::ClientCommand::Playlist {
                    cmd: command.into(),
                }) {
                    Ok(response) => match response {
                        DaemonResponse::Ok => {
                            println!("Ok");
                        }
                        DaemonResponse::Playlists { playlists } => {
                            display_playlists(&playlists, full);
                        }
                        DaemonResponse::Error { error } => {
                            error!("{error}");
                        }
                    },
                    Err(e) => {
                        print_daemon_error(e);
                    }
                }
            }
        }
    } else {
        trace!("No command specified, starting a deamon with GUI");

        let handle = thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(daemon::start()) {
                Ok(status) => info!("Daemon exited: {status}"),
                Err(e) => error!("Daemon failed: {e}"),
            }
        });

        trace!("Starting GUI");
        match gui::start() {
            Ok(status) => info!("GUI exited: {status}"),
            Err(e) => error!("GUI failed: {e}"),
        };

        handle.join().unwrap();
    }
    Ok(())
}
