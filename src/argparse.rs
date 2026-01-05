use std::path::PathBuf;

use clap::{Parser, Subcommand};

use env_logger::Builder;
use log::{LevelFilter, error, trace};
use mpipc::ClientCommand;

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

    #[arg(long, short = 'v', default_value_t = String::from("Warn"))]
    /// The log filter level
    ///
    /// Used to configure the logger to filter certain log messages. Useful for debugging.
    ///
    /// Possible values are: `off`, `error`, `warn`, `info`, `debug`, and `trace`.
    ///
    /// Alternatively, you can use numbers from 0 to 5 to set the log filtering level.
    pub logger_verbosity: String,

    #[arg(long, short)]
    /// The directory with your audio files
    ///
    /// By default, only `~/Music/` will be searched. Use this option if you store your
    /// music outside this directory.
    pub music_dir: Option<String>,
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
    Playlist {
        #[command(subcommand)]
        command: PlaylistCommand,
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

impl From<DaemonCommand> for mpipc::DaemonCommand {
    fn from(value: DaemonCommand) -> Self {
        match value {
            DaemonCommand::Start => mpipc::DaemonCommand::Start,
            DaemonCommand::Stop => mpipc::DaemonCommand::Message {
                command: ClientCommand::Stop,
            },
            DaemonCommand::Restart => mpipc::DaemonCommand::Message {
                command: ClientCommand::Restart,
            },
            DaemonCommand::Status => mpipc::DaemonCommand::Message {
                command: ClientCommand::Status,
            },
        }
    }
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

#[derive(Subcommand)]
pub enum PlaylistCommand {
    /// Create a new playlist
    New {
        name: String,
        tracks: Option<Vec<PathBuf>>,
    },
    /// Import a playlist from an m3u8 file
    FromM3U8 { name: String, m3u8_file: PathBuf },
    /// Delete playlists from the library
    Delete { names: Vec<String> },
    /// List all playlists from the library
    List {
        #[arg(long, short, default_value_t = false)]
        /// Also list all the tracks in the playlists
        full: bool,
    },
}

impl From<PlaylistCommand> for mpipc::PlaylistCommand {
    fn from(value: PlaylistCommand) -> Self {
        match value {
            PlaylistCommand::New { name, tracks } => mpipc::PlaylistCommand::New { name, tracks },
            PlaylistCommand::FromM3U8 { name, m3u8_file } => {
                mpipc::PlaylistCommand::FromM3U8 { name, m3u8_file }
            }
            PlaylistCommand::Delete { names } => mpipc::PlaylistCommand::Delete { names },
            PlaylistCommand::List { full: _ } => mpipc::PlaylistCommand::List,
        }
    }
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

    let filter = match level_filter_from_string(&args.logger_verbosity) {
        Ok(log_level) => log_level,
        Err(e) => {
            error!("Failed parsing log level from arguments: {e}");
            LevelFilter::Info
        }
    };

    Builder::new()
        .filter_level(filter)
        .filter_module("lofty", LevelFilter::Warn)
        .init();

    trace!("Finished parsing command line arguments");

    args
}
