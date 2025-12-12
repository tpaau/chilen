mod argparse;
mod daemon;
mod gui;

use argparse::parse_args;
use log::{error, info};

use crate::argparse::{Command, DaemonCommand};

fn main() {
    let args = parse_args();

    if let Some(command) = args.command {
        match command {
            Command::Daemon { command } => match command {
                DaemonCommand::Start => match daemon::start() {
                    Ok(status) => {
                        info!("Daemon exited: {status}");
                    }
                    Err(e) => {
                        error!("Daemon failed: {e}");
                    }
                },
                _ => {
                    mpipc::exec_daemon_command(command.into());
                }
            },
            Command::Gui { command } => match gui::start(command) {
                Ok(status) => {
                    info!("GUI exited: {status}");
                }
                Err(e) => {
                    error!("GUI failed: {e}");
                }
            },
        }
    } else {
    }
}
