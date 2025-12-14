mod argparse;
mod daemon;
mod gui;

use std::{
    thread::{self, sleep},
    time::Duration,
};

use argparse::parse_args;
use log::{error, info, trace};

use crate::argparse::{Command, DaemonCommand, GuiCommand};

#[tokio::main]
async fn main() {
    let args = parse_args();

    if let Some(command) = args.command {
        match command {
            Command::Daemon { command } => {
                if let DaemonCommand::Start = command {
                    match daemon::start().await {
                        Ok(status) => {
                            info!("Daemon exited: {status}");
                            eprintln!("Daemon exited: {status}");
                        }
                        Err(e) => error!("Daemon failed: {e}"),
                    }
                } else {
                    let cmd: mpipc::DaemonCommand = command.into();
                    let cmd = match cmd.try_into() {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            error!("Could not send the command to the daemon: {e}");
                            return;
                        }
                    };
                    match mpipc::exec_client_command(cmd) {
                        Ok(response) => {
                            info!("Got a response from the daemon: {response}");
                            eprintln!("Got a response from the daemon: {response}");
                        }
                        Err(e) => error!("Failed executing the daemon command: {e}"),
                    }
                }
            }
            Command::Gui { command } => {
                if let GuiCommand::Start = command {
                    match gui::start() {
                        Ok(status) => {
                            info!("GUI exited: {status}");
                            eprintln!("GUI exited: {status}")
                        }
                        Err(e) => error!("GUI failed: {e}"),
                    }
                } else {
                    panic!("GUI command execution not supported!");
                }
            }
        }
    } else {
        trace!("No command specified, starting a deamon with GUI");

        let handle = thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(daemon::start()) {
                Ok(status) => {
                    info!("Daemon exited: {status}");
                    eprintln!("Daemon exited: {status}");
                }
                Err(e) => error!("Daemon failed: {e}"),
            }
        });

        // For testing purposes only to give the daemon time
        sleep(Duration::from_secs(1));

        trace!("Starting GUI");
        match gui::start() {
            Ok(status) => {
                info!("GUI exited: {status}");
                eprintln!("GUI exited: {status}")
            }
            Err(e) => error!("GUI failed: {e}"),
        };

        handle.join().unwrap();
    }
}
