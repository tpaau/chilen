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
            Command::Daemon { command } => {
                match command {
                    DaemonCommand::Start => {
                        match daemon::start() {
                            Ok(status) => {
                                info!("Daemon exited: {status}");
                            }
                            Err(e) => {
                                error!("Daemon failed to start: {e}");
                            }
                        }
                    }
                    DaemonCommand::Stop => {

                    }
                    DaemonCommand::Restart => {

                    }
                    DaemonCommand::Status => {

                    }
                }
            }
            Command::Gui { command } => {

            }
        }
    }
    else {

    }
}
