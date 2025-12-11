mod argparse;
mod daemon;
mod gui;

use argparse::parse_args;

use crate::argparse::Command;

fn main() {
    let args = parse_args();

    if let Some(command) = args.command {
        match command {
            Command::Daemon { command } => {

            }
            Command::Gui { command } => {

            }
        }
    }
    else {

    }
}
