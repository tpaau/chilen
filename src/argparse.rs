use std::{path::PathBuf, time::Duration};

use clap::{ArgGroup, Parser, Subcommand, ValueHint};

use env_logger::Builder;
use log::{LevelFilter, error, trace};
use mpipc::ClientCommand;

#[derive(Subcommand, PartialEq, Eq)]
pub enum DaemonCommand {
    /// Start the daemon process
    Start {
        #[arg(long, short, value_hint = ValueHint::DirPath)]
        /// Set the cache runtime directory.
        cache_dir: Option<PathBuf>,
        #[arg(long, short, value_hint = ValueHint::DirPath)]
        /// Set the data runtime directory.
        data_dir: Option<PathBuf>,
        #[arg(long, short, value_hint = ValueHint::DirPath)]
        /// Set the directory with audio files.
        music_dir: Option<PathBuf>,
    },
    /// Stop the daemon process
    Stop,
    /// Stream events from the daemon. Causes the thread to stop accepting requests.
    EventStream {
        #[arg(long, short, default_value_t = false, conflicts_with = "pretty_json")]
        /// Show the output in JSON
        json: bool,

        #[arg(long, short, default_value_t = false, conflicts_with = "json")]
        /// Show the output in nicely formatted JSON
        pretty_json: bool,
    },
    /// Restart the daemon process
    Restart,
}

impl From<DaemonCommand> for mpipc::DaemonCommand {
    fn from(value: DaemonCommand) -> Self {
        match value {
            DaemonCommand::Start {
                cache_dir: _,
                music_dir: _,
                data_dir: _,
            } => mpipc::DaemonCommand::Start,
            DaemonCommand::Stop => mpipc::DaemonCommand::ClientCommand(ClientCommand::Shutdown),
            DaemonCommand::EventStream {
                json: _,
                pretty_json: _,
            } => mpipc::DaemonCommand::ClientCommand(ClientCommand::EventStream),
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
    /// List all playlists in the library
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
pub enum PlayCommand {
    /// Play the queue.
    Current,
    /// Put a list of tracks in the queue before playing it.
    Tracks {
        #[arg(value_parser = is_file, value_hint = ValueHint::FilePath)]
        tracks: Vec<PathBuf>,
    },
    /// Put the contents of a playlist in the queue before playing it.
    Playlist { name: String },
}

#[derive(Subcommand)]
pub enum LoopState {
    /// Do not loop.
    Off,
    /// Loop the current track.
    Track,
    /// Loop the current playlist.
    Playlist,
}

impl From<LoopState> for mpipc::LoopState {
    fn from(value: LoopState) -> Self {
        match value {
            LoopState::Off => mpipc::LoopState::Off,
            LoopState::Track => mpipc::LoopState::Track,
            LoopState::Playlist => mpipc::LoopState::Playlist,
        }
    }
}

#[derive(Subcommand)]
pub enum ShuffleState {
    /// Enable shuffle.
    On,
    /// Disable shuffle.
    Off,
}

impl From<ShuffleState> for mpipc::ShuffleState {
    fn from(value: ShuffleState) -> Self {
        match value {
            ShuffleState::On => mpipc::ShuffleState::On,
            ShuffleState::Off => mpipc::ShuffleState::Off,
        }
    }
}

#[derive(Subcommand)]
pub enum PlaybackCommand {
    /// Play the audio.
    Play,
    /// Set the queue to a playlist or a list of tracks.
    #[command(group(
        ArgGroup::new("source")
            .required(true)
            .args(&["tracks", "playlist"])
    ))]
    SetQueue {
        /// Paths of tracks to be set as the new queue.
        #[arg(long, short, value_parser = is_file, value_hint = ValueHint::FilePath, conflicts_with = "playlist")]
        tracks: Option<Vec<PathBuf>>,

        /// The name of the playlist to be set as the new queue.
        #[arg(long, short, conflicts_with = "tracks")]
        playlist: Option<String>,
    },
    /// Append a list of tracks to the queue.
    AppendToQueue {
        /// Paths of tracks to be appended to the queue.
        #[arg(value_parser = is_file, value_hint = ValueHint::FilePath)]
        tracks: Vec<PathBuf>,
    },
    /// Pause the audio.
    Pause,
    /// Skip to the next track.
    Next,
    /// Skip to the previous track.
    Previous,
    /// Set the loop state.
    SetLoopState {
        #[command(subcommand)]
        loop_state: LoopState,
    },
    /// Set the shuffle state.
    SetShuffleState {
        #[command(subcommand)]
        shuffle_state: ShuffleState,
    },
    /// Set the player position.
    SetPosition {
        #[arg()]
        position_secs: u64,
    },
}

impl From<PlaybackCommand> for mpipc::PlaybackCommand {
    fn from(value: PlaybackCommand) -> Self {
        match value {
            PlaybackCommand::Play => mpipc::PlaybackCommand::Play,
            PlaybackCommand::SetQueue { tracks, playlist } => {
                if let Some(tracks) = tracks {
                    return mpipc::PlaybackCommand::SetQueue(tracks);
                } else if let Some(playlist) = playlist {
                    return mpipc::PlaybackCommand::SetPlaylist(playlist);
                }
                panic!("This should never happen :)");
            }
            PlaybackCommand::AppendToQueue { tracks } => {
                mpipc::PlaybackCommand::AppendToQueue(tracks)
            }
            PlaybackCommand::Pause => mpipc::PlaybackCommand::Pause,
            PlaybackCommand::Next => mpipc::PlaybackCommand::Next,
            PlaybackCommand::Previous => mpipc::PlaybackCommand::Previous,
            PlaybackCommand::SetLoopState { loop_state } => {
                mpipc::PlaybackCommand::SetLoopState(loop_state.into())
            }
            PlaybackCommand::SetShuffleState { shuffle_state } => {
                mpipc::PlaybackCommand::SetShuffleState(shuffle_state.into())
            }
            PlaybackCommand::SetPosition { position_secs } => {
                mpipc::PlaybackCommand::SetPosition(Duration::from_secs(position_secs))
            }
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
    /// Control audio playback
    Playback {
        #[command(subcommand)]
        command: PlaybackCommand,
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

    let foreign_module_filter = LevelFilter::Error;

    Builder::new()
        .filter_level(filter)
        .filter_module("calloop", foreign_module_filter)
        .filter_module("cosmic_text", foreign_module_filter)
        .filter_module("iced_graphics", foreign_module_filter)
        .filter_module("iced_wgpu", foreign_module_filter)
        .filter_module("iced_winit", foreign_module_filter)
        .filter_module("lofty", foreign_module_filter)
        .filter_module("naga", foreign_module_filter)
        .filter_module("sctk", foreign_module_filter)
        .filter_module("tracing", foreign_module_filter)
        .filter_module("wgpu_core", foreign_module_filter)
        .filter_module("wgpu_hal", foreign_module_filter)
        .filter_module("winit", foreign_module_filter)
        .filter_module("zbus", foreign_module_filter)
        .filter_module("symphonia_core", foreign_module_filter)
        .filter_module("symphonia_bundle_mp3", foreign_module_filter)
        .init();

    trace!("Finished parsing command line arguments");

    args
}
