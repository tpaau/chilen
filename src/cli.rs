use std::thread;

use log::{error, info, trace};
use mpipc::{DaemonResponse, Playlist};

use crate::{
    argparse::{Command, DaemonCommand, GuiCommand, PlaylistCommand},
    daemon, gui,
};

fn display_playlists(playlists: &Vec<Playlist>, full: bool) {
    if full {
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
                        Err(e) => error!("Failed executing the daemon command: {e}"),
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
                        _ => {
                            info!("Got a response from the daemon: {response}");
                        }
                    },
                    Err(e) => {
                        error!("Failed executing the daemon command: {e}");
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
