mod argparse;
mod cli;
#[cfg(feature = "gui")]
mod gui;
// #[cfg(test)]
// mod tests;

use crate::{cli::run_cli_command, argparse::parse_args};

fn main() -> Result<(), ()> {
    run_cli_command(parse_args().command)
}
