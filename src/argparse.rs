use clap::{Parser, Subcommand};

use env_logger::Builder;
use log::{LevelFilter, error, trace};

#[derive(Parser)]
#[command(
    version,
    author = "tpaau <tpaau-17DB@tutamail.com>",
    help_template = "{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}
{all-args}{after-help}
"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, short = 'v', default_value_t = String::from("Info"))]
    /// Set the log filter level
    pub logger_verbosity: String,
}

#[derive(Subcommand)]
pub enum Command {
    #[command()]
    /// Manage the daemon
    Daemon {
        #[command(subcommand)]
        /// Command for the daemon
        command: DaemonCommand,
    },
    #[command()]
    /// Manage the GUI
    Gui {
        #[command(subcommand)]
        /// Command for the GUI
        command: GuiCommand,
    },
}

#[derive(Subcommand)]
pub enum DaemonCommand {
    /// Start the daemon process
    Start,
    /// Stop the daemon process
    Stop,
    /// Restart the daemon process
    Restart,
    /// Show daemon status
    Status,
}

#[derive(Subcommand)]
pub enum GuiCommand {
    /// Start the GUI process
    Start,
    /// Stop the GUI process
    Stop,
    /// Restart the GUI process
    Restart,
    /// Show GUI status
    Status,
}

fn level_filter_from_string(filter_string: &str) -> Result<LevelFilter, String> {
    match filter_string.to_lowercase().as_str() {
        "0" | "off" => Ok(LevelFilter::Off),
        "1" | "error" => Ok(LevelFilter::Error),
        "2" | "warn" => Ok(LevelFilter::Warn),
        "3" | "info" => Ok(LevelFilter::Info),
        "4" | "debug" => Ok(LevelFilter::Debug),
        "5" | "trace" => Ok(LevelFilter::Trace),
        _ => Err(format!(
            "No log level for the provided string: '{filter_string}'"
        )),
    }
}

pub fn parse_args() -> Args {
    let args = Args::parse();

    match level_filter_from_string(&args.logger_verbosity) {
        Ok(log_level) => {
            Builder::new().filter_level(log_level).init();
        }
        Err(e) => {
            Builder::new().filter_level(LevelFilter::Info).init();
            error!("Failed parsing log level from arguments: {e}");
        }
    }

    trace!("Finished parsing command line arguments");

    args
}
