use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};

use env_logger::Builder;
use log::{LevelFilter, error, trace};
use mpipc::ClientCommand;

#[derive(Subcommand, PartialEq, Eq)]
pub enum DaemonCommand {
    /// Start the daemon process
    Start,
    /// Stop the daemon process
    Stop,
    /// Stream events from the daemon. Causes the thread to stop accepting requests.
    EventStream,
    /// Restart the daemon process
    Restart,
}

impl From<DaemonCommand> for mpipc::DaemonCommand {
    fn from(value: DaemonCommand) -> Self {
        match value {
            DaemonCommand::Start => mpipc::DaemonCommand::Start,
            DaemonCommand::Stop => mpipc::DaemonCommand::ClientCommand(ClientCommand::Shutdown),
            DaemonCommand::EventStream => {
                mpipc::DaemonCommand::ClientCommand(ClientCommand::EventStream)
            }
            DaemonCommand::Restart => mpipc::DaemonCommand::ClientCommand(ClientCommand::Restart),
        }
    }
}

#[cfg(feature = "gui")]
#[derive(Subcommand)]
pub enum GuiCommand {
    /// Start the GUI process
    Start,
    /// Stop the GUI process
    Stop,
    /// Restart the GUI process
    Restart,
}

#[derive(Subcommand)]
pub enum PlaylistCommand {
    /// Create a new playlist
    New {
        /// The name of the playlist
        name: String,
        /// The list of tracks to add to the new playlist
        #[arg(value_parser = is_file, value_hint = ValueHint::FilePath)]
        tracks: Option<Vec<PathBuf>>,
    },
    /// Import a playlist from an M3U8 file
    FromM3U8 {
        // The path to the M3U8 file to import the playlist from
        #[arg(value_parser = is_file, value_hint = ValueHint::FilePath)]
        m3u8_file: PathBuf,
        /// The name of the playlist
        ///
        /// If this is not specified, the name of the playlist will be derived from the
        /// name of the M3U8 file.
        ///
        name: Option<String>,
    },
    /// Delete playlist(s) from the library
    Delete { names: Vec<String> },
    /// Add tracks to an already existing playlist.
    AddTracks {
        /// The name of the playlist to operate on.
        name: String,
        #[arg(value_parser = is_file, value_hint = ValueHint::FilePath)]
        /// The list of tracks to add.
        tracks: Vec<PathBuf>,
    },
    /// Remove tracks from an already existing playlist.
    RemoveTracks {
        /// The name of the playlist to operate on.
        name: String,
        /// The list of IDs of tracks to remove.
        ids: Vec<usize>,
    },
    /// List all playlists from the library
    List {
        #[arg(long, short, default_value_t = false, conflicts_with = "debug")]
        /// Also list all the tracks in the playlists
        full: bool,

        #[arg(long, short, default_value_t = false, conflicts_with = "full")]
        /// Print all the info about the playlists
        debug: bool,
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
            PlaylistCommand::List { full: _, debug: _ } => mpipc::PlaylistCommand::List,
            PlaylistCommand::AddTracks { name, tracks } => {
                mpipc::PlaylistCommand::AddTracks { name, tracks }
            }
            PlaylistCommand::RemoveTracks { name, ids } => {
                mpipc::PlaylistCommand::RemoveTracks { name, ids }
            }
        }
    }
}

#[derive(Subcommand)]
pub enum CacheCommand {
    /// Reinitialize the music library to find newly added tracks.
    Reload,
    /// Rebuild the cache. May resolve some issues with badly extracted covers.
    Rebuild,
}

impl From<CacheCommand> for mpipc::CacheCommand {
    fn from(value: CacheCommand) -> Self {
        match value {
            CacheCommand::Reload => mpipc::CacheCommand::Reload,
            CacheCommand::Rebuild => mpipc::CacheCommand::Rebuild,
        }
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage the daemon
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    /// Manage the GUI
    #[cfg(feature = "gui")]
    Gui {
        #[command(subcommand)]
        command: GuiCommand,
    },
    /// Manage playlists
    Playlist {
        #[command(subcommand)]
        command: PlaylistCommand,
    },
    /// Manage cache
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
}

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
    /// Set the log filter level
    ///
    /// Possible values are: `off`, `error`, `warn`, `info`, `debug`, and `trace`.
    ///
    /// You can also use numbers from 0 to 5 to set the log filtering level.
    pub logger_verbosity: String,

    #[cfg(feature = "landlock")]
    #[arg(long, short, default_value_t = false)]
    /// Disable landlock sandboxing
    pub no_landlock: bool,
}

fn is_file(path: &str) -> Result<PathBuf, String> {
    let track = PathBuf::from(path);
    if !track.is_file() {
        return Err(format!("Not a file: {path}"));
    }
    Ok(track)
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
