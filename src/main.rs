mod argparse;
mod cli;
#[cfg(feature = "gui")]
mod gui;

use std::process::exit;

use crate::{argparse::parse_args, cli::run_cli_command};

fn main() {
    let args = parse_args();
    match run_cli_command(args.command, args.socket_type.into(), args.socket_name) {
        Ok(_) => exit(0),
        Err(_) => exit(1),
    }
}
