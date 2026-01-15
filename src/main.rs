mod argparse;
mod cli;
#[cfg(feature = "gui")]
mod gui;
// #[cfg(test)]
// mod tests;

use crate::{argparse::parse_args, cli::run_cli_command};

fn main() -> Result<(), ()> {
    run_cli_command(parse_args().command)
}
