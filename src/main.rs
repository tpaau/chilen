mod argparse;
mod daemon;
mod gui;

use argparse::parse_args;
use log::{error, info};

use crate::argparse::{Command, DaemonCommand, GuiCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();

    if let Some(command) = args.command {
        match command {
            Command::Daemon { command } => {
                if let DaemonCommand::Start = command {
                    match daemon::start().await {
                        Ok(status) => {
                            info!("Daemon exited: {status}");
                            Ok(())
                        }
                        Err(e) => {
                            error!("Daemon failed: {e}");
                            Err(format!("Daemon failed: {e}").into())
                        }
                    }
                } else {
                    let cmd: mpipc::DaemonCommand = command.into();
                    let cmd = match cmd.try_into() {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            return Err(
                                format!("Could not send the command to the daemon: {e}").into()
                            );
                        }
                    };

                    match mpipc::exec_client_command(cmd) {
                        Ok(response) => {
                            info!("Got a response from daemon: {response}");
                            eprintln!("{response}");
                            Ok(())
                        }
                        Err(e) => {
                            error!("Failed executing daemon command: {e}");
                            Err(format!("Failed executing daemon command: {e}").into())
                        }
                    }
                }
            }
            Command::Gui { command } => {
                if let GuiCommand::Start = command {
                    match gui::start() {
                        Ok(status) => {
                            info!("GUI exited: {status}");
                            Ok(())
                        }
                        Err(e) => {
                            error!("GUI failed: {e}");
                            Err(format!("GUI failed: {e}").into())
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
