mod argparse;
mod cache;
mod cli;
mod cxxqt_object;
mod daemon;
mod gui;
#[cfg(test)]
mod tests;
mod track;

use argparse::parse_args;

use crate::cli::run_cli_command;

fn main() -> Result<(), ()> {
    smol::block_on(async { run_cli_command(parse_args().command).await })
}
