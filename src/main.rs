mod argparse;
mod cli;
#[cfg(feature = "gui")]
mod gui;
#[cfg(test)]
mod tests;

use std::process::exit;

use crate::{argparse::parse_args, cli::run_cli_command};

fn main() -> Result<(), ()> {
    match run_cli_command(parse_args().command) {
        Ok(_) => exit(0),
        Err(_) => exit(1),
    }
}
