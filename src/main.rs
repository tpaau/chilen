mod argparse;
mod daemon;
mod gui;

use argparse::parse_args;
use log::{error, info};

use crate::argparse::{Command, DaemonCommand, GuiCommand};

fn main() {
    let args = parse_args();

    if let Some(command) = args.command {
        match command {
            Command::Daemon { command } => {
                if let DaemonCommand::Start = command {
                    match daemon::start() {
                        Ok(status) => {
                            info!("Daemon exited: {status}");
                        }
                        Err(e) => {
                            error!("Daemon failed: {e}");
                        }
                    }
                } else {
                    match mpipc::exec_daemon_command(command.into()) {
                        Ok(response) => {}
                        Err(e) => {
                            error!("Failed executing daemon command: {e}")
                        }
                    }
                }
            }
            Command::Gui { command } => {
                if let GuiCommand::Start = command {
                    match gui::start() {
                        Ok(status) => {
                            info!("GUI exited: {status}");
                        }
                        Err(e) => {
                            error!("GUI failed: {e}");
                        }
                    }
                } else {
                    panic!("GUI command execution not supported!");
                }
            }
        }
    } else {
        panic!("Default command not yet supported!");
    }
}
