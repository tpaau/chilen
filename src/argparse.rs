use std::{path::PathBuf, time::Duration};

use clap::{ArgGroup, Parser, Subcommand, ValueEnum, ValueHint};

use env_logger::Builder;
use log::{self, trace};
use mpipc::ClientCommand;

#[derive(Subcommand, PartialEq, Eq)]
pub enum DaemonCommand {
    /// Start the daemon process
    Start {
        /// Set the cache runtime directory.
        #[arg(long, short, value_hint = ValueHint::DirPath)]
        cache_dir: Option<PathBuf>,
        /// Set the data runtime directory.
        #[arg(long, short, value_hint = ValueHint::DirPath)]
        data_dir: Option<PathBuf>,
        /// Set the directory with audio files.
        #[arg(long, short, value_hint = ValueHint::DirPath)]
        music_dir: Option<PathBuf>,
    },
    /// Stop the daemon process
    Stop,
    /// Stream events from the daemon. Causes the thread to stop accepting requests.
    EventStream {
        #[arg(long, short, default_value_t = false, conflicts_with = "pretty_json")]
        /// Show the output in JSON
        json: bool,

        /// Show the output in nicely formatted JSON
        #[arg(long, short, default_value_t = false, conflicts_with = "json")]
        pretty_json: bool,
    },
    /// Ping the daemon (used for debugging)
    Ping,
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
            DaemonCommand::Ping => mpipc::DaemonCommand::ClientCommand(ClientCommand::Ping),
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
        /// The list of tracks to add.
        #[arg(value_parser = is_file, value_hint = ValueHint::FilePath)]
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
        /// Also list all the tracks in the playlists
        #[arg(long, short, default_value_t = false, conflicts_with = "debug")]
        full: bool,

        /// Print all the info about the playlists
        #[arg(long, short, default_value_t = false, conflicts_with = "full")]
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
    Play {
        /// Play the track at this index.
        #[arg(long, short)]
        index: Option<usize>,
    },
    /// Set the queue to a playlist or a list of tracks.
    #[command(group(
        ArgGroup::new("source")
            .required(true)
            .args(&["tracks", "playlist"])
    ))]
    /// Append the queue.
    SetQueue {
        /// Paths of tracks to be set as the new queue.
        #[arg(long, short, value_parser = is_file, value_hint = ValueHint::FilePath, conflicts_with = "playlist")]
        tracks: Option<Vec<PathBuf>>,

        /// The name of the playlist to be set as the new queue.
        #[arg(long, short, conflicts_with = "tracks")]
        playlist: Option<String>,
    },
    /// Append a tracks to the queue.
    AppendToQueue {
        /// Paths of tracks to be appended to the queue.
        #[arg(value_parser = is_file, value_hint = ValueHint::FilePath)]
        tracks: Option<Vec<PathBuf>>,

        /// The name of the playlist to append to the queue.
        #[arg(long, short, conflicts_with = "tracks")]
        playlist: Option<String>,
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
    SetPlayerPosition {
        #[arg()]
        position_secs: u64,
    },
}

impl From<PlaybackCommand> for mpipc::PlaybackCommand {
    fn from(value: PlaybackCommand) -> Self {
        match value {
            PlaybackCommand::Play { index } => mpipc::PlaybackCommand::Play(index),
            PlaybackCommand::SetQueue { tracks, playlist } => {
                if let Some(tracks) = tracks {
                    return mpipc::PlaybackCommand::SetQueue(tracks);
                } else if let Some(playlist) = playlist {
                    return mpipc::PlaybackCommand::SetPlaylist(playlist);
                }
                panic!("This should never happen :)");
            }
            PlaybackCommand::AppendToQueue { tracks, playlist } => {
                if let Some(tracks) = tracks {
                    return mpipc::PlaybackCommand::AppendToQueue(tracks);
                } else if let Some(playlist) = playlist {
                    return mpipc::PlaybackCommand::AppendPlaylist(playlist);
                }
                panic!("This should never happen :)");
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
            PlaybackCommand::SetPlayerPosition { position_secs } => {
                mpipc::PlaybackCommand::SetPlayerPosition(Duration::from_secs(position_secs))
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

#[derive(Default, ValueEnum, Clone, Copy)]
pub enum SocketType {
    NamespacedOnly,
    #[default]
    NamespacedOrFilesystem,
    FilesystemOnly,
}

impl std::fmt::Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NamespacedOnly => write!(f, "namespaced-only"),
            Self::NamespacedOrFilesystem => write!(f, "namespaced-or-filesystem"),
            Self::FilesystemOnly => write!(f, "filesystem-only"),
        }
    }
}

impl From<SocketType> for mpipc::SocketType {
    fn from(value: SocketType) -> Self {
        match value {
            SocketType::NamespacedOnly => Self::NamespacedOnly,
            SocketType::NamespacedOrFilesystem => Self::NamespacedOrFilesystem,
            SocketType::FilesystemOnly => Self::FilesystemOnly,
        }
    }
}

#[derive(Default, ValueEnum, Clone, Copy)]
pub enum LevelFilter {
    Off,
    Error,
    #[default]
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LevelFilter> for log::LevelFilter {
    fn from(level: LevelFilter) -> Self {
        match level {
            LevelFilter::Off => log::LevelFilter::Off,
            LevelFilter::Error => log::LevelFilter::Error,
            LevelFilter::Warn => log::LevelFilter::Warn,
            LevelFilter::Info => log::LevelFilter::Info,
            LevelFilter::Debug => log::LevelFilter::Debug,
            LevelFilter::Trace => log::LevelFilter::Trace,
        }
    }
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

    /// Specify the socket name to use.
    #[arg(long, short)]
    pub socket_name: Option<String>,

    /// Specify the socket type to use.
    #[arg(long, short = 'n', default_value_t = SocketType::default())]
    pub socket_type: SocketType,

    #[arg(long, short)]
    /// Set the log filter level
    pub log_filter: LevelFilter,
}

fn is_file(path: &str) -> Result<PathBuf, String> {
    let track = PathBuf::from(path);
    if !track.is_file() {
        return Err(format!("Not a file: {path}"));
    }
    Ok(track)
}

pub fn parse_args() -> Args {
    let args = Args::parse();

    let foreign_module_filter = log::LevelFilter::Error;

    Builder::new()
        .filter_level(args.log_filter.into())
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
