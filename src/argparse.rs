use std::{path::PathBuf, time::Duration};

use clap::{ArgAction, ArgGroup, Parser, Subcommand, ValueEnum, ValueHint};

use chilen_ipc::playback::SignedDuration;
use env_logger::Builder;
use log::{self, trace};

#[derive(Debug, Subcommand, PartialEq, Eq)]
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
        /// Allow clients to modify the playback rate of the player.
        #[arg(long, short, default_value_t = false)]
        allow_rate_modification: bool,
        /// Whether the daemon can receive raise requests from clients.
        #[arg(long, short = 'r', num_args = 1, default_value_t = true)]
        can_raise: bool,
        /// Whether clients can request the daemon to toggle fullscreen mode of the user interface.
        #[cfg(feature = "dev-commands")]
        #[arg(long, short = 'f', num_args = 1, default_value_t = true)]
        can_set_fullscreen: bool,
        /// Whether clients can request the daemon to quit.
        #[arg(long, short = 'q', num_args = 1, default_value_t = true)]
        can_quit: bool,
    },
    /// Stop the daemon process
    Quit,
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
    /// Check whether the daemon's user interface can be brought to the front.
    GetCanRaise,
    /// Bring the user interface to the front.
    Raise,
    /// Check whether clients can toggle fullscreen mode of the user interface.
    GetCanSetFullscreen,
    /// Toggle fullscreen mode of the user interface.
    SetFullscreen {
        #[arg(action = ArgAction::Set)]
        fullscreen: bool,
    },
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
    Import {
        /// The path to the M3U8 file to import the playlist from
        #[arg(value_parser = is_file, value_hint = ValueHint::FilePath)]
        m3u8_file: PathBuf,
        /// The name of the playlist
        ///
        /// If left unspecified, it will be derived from the name set in the M3U8 playlist.
        name: Option<String>,
    },
    /// Export a playlist as an M3U8 file.
    Export {
        /// The name of the playlist to export
        name: String,
        /// The path to export the playlist to
        path: PathBuf,
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

impl From<PlaylistCommand> for chilen_ipc::library::LibraryCommand {
    fn from(value: PlaylistCommand) -> Self {
        match value {
            PlaylistCommand::New { name, tracks } => {
                chilen_ipc::library::LibraryCommand::NewPlaylist { name, tracks }
            }
            PlaylistCommand::Import { name, m3u8_file } => {
                chilen_ipc::library::LibraryCommand::PlaylistFromM3U8 { name, m3u8_file }
            }
            PlaylistCommand::Export { name, path } => {
                chilen_ipc::library::LibraryCommand::ExportPlaylistToM3U { name, path }
            }
            PlaylistCommand::Delete { names } => {
                chilen_ipc::library::LibraryCommand::DeletePlaylists { names }
            }
            PlaylistCommand::List { full: _, debug: _ } => {
                chilen_ipc::library::LibraryCommand::GetLibrary
            }
            PlaylistCommand::AddTracks { name, tracks } => {
                chilen_ipc::library::LibraryCommand::AddTracksToPlaylist { name, tracks }
            }
            PlaylistCommand::RemoveTracks { name, ids } => {
                chilen_ipc::library::LibraryCommand::RemoveTracksFromPlaylist { name, ids }
            }
        }
    }
}

#[derive(Subcommand)]
pub enum LibraryCommand {
    /// Manage playlists in the library.
    Playlist {
        #[command(subcommand)]
        command: PlaylistCommand,
    },
    /// Reload the music library using already cached covers if possible.
    ///
    /// This can be used for discovering newly added tracks.
    Reload,
    /// Reload the music library and rebuild the cache.
    ///
    /// This does not reset user-generated data like playlists.
    Rebuild,
}

impl From<LibraryCommand> for chilen_ipc::library::LibraryCommand {
    fn from(value: LibraryCommand) -> Self {
        match value {
            LibraryCommand::Playlist { command } => command.into(),
            LibraryCommand::Reload => chilen_ipc::library::LibraryCommand::Reload,
            LibraryCommand::Rebuild => chilen_ipc::library::LibraryCommand::Rebuild,
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

impl From<LoopState> for chilen_ipc::playback::LoopState {
    fn from(value: LoopState) -> Self {
        match value {
            LoopState::Off => chilen_ipc::playback::LoopState::Off,
            LoopState::Track => chilen_ipc::playback::LoopState::Track,
            LoopState::Playlist => chilen_ipc::playback::LoopState::Playlist,
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

impl From<ShuffleState> for chilen_ipc::playback::ShuffleState {
    fn from(value: ShuffleState) -> Self {
        match value {
            ShuffleState::On => chilen_ipc::playback::ShuffleState::On,
            ShuffleState::Off => chilen_ipc::playback::ShuffleState::Off,
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
    /// Pause the audio.
    Pause,
    /// Stop the player.
    Stop,
    /// Toggle between play/pause.
    TogglePlaying,
    /// Get the playback state (Playing, Paused, Stopped) of the player.
    GetPlaybackState,
    /// Set the queue to a playlist or a list of tracks.
    #[command(group(
        ArgGroup::new("source")
            .required(true)
            .args(&["tracks", "playlist"])
    ))]
    SetQueue {
        /// Paths of tracks to be set as the new queue.
        #[arg(
            long,
            short,
            value_parser = is_file,
            value_hint = ValueHint::FilePath,
            conflicts_with = "playlist"
        )]
        tracks: Option<Vec<PathBuf>>,

        /// The name of the playlist to be set as the new queue.
        #[arg(long, short, conflicts_with = "tracks")]
        playlist: Option<String>,
    },
    /// Remove all tracks from the queue.
    ClearQueue,
    /// Append a tracks to the queue.
    AppendToQueue {
        /// Paths of tracks to be appended to the queue.
        #[arg(value_parser = is_file, value_hint = ValueHint::FilePath)]
        tracks: Option<Vec<PathBuf>>,

        /// The name of the playlist to append to the queue.
        #[arg(long, short, conflicts_with = "tracks")]
        playlist: Option<String>,
    },
    /// Get the current track.
    GetCurrentTrack,
    /// Skip to the next track.
    Next,
    /// Skip to the previous track.
    Previous,
    /// Set the loop state.
    SetLoopState {
        #[command(subcommand)]
        loop_state: LoopState,
    },
    /// Get the loop state of the player.
    GetLoopState,
    /// Set the playback rate of the player.
    SetRate {
        #[arg()]
        rate: f32,
    },
    /// Get the playback rate of the player.
    GetRate,
    /// Set the shuffle state.
    SetShuffleState {
        #[command(subcommand)]
        shuffle_state: ShuffleState,
    },
    /// Get the shuffle state of the player.
    GetShuffleState,
    /// Set the player position.
    SetPlayerPosition {
        #[arg()]
        position_secs: u64,
    },
    /// Seek the player forward by a number of seconds.
    SeekForward {
        #[arg()]
        secs: u64,
    },
    /// Seek the player backward by a number of seconds.
    SeekBackward {
        #[arg()]
        secs: u64,
    },
    /// Get the position of the player.
    GetPlayerPosition,
    /// Set the volume of the player.
    SetVolume {
        #[arg()]
        volume: f64,
    },
    /// Get the volume of the player.
    GetVolume,
    OpenUri {
        #[arg()]
        uri: String,
    },
}

impl From<PlaybackCommand> for chilen_ipc::playback::PlaybackCommand {
    fn from(value: PlaybackCommand) -> Self {
        match value {
            PlaybackCommand::Play { index } => chilen_ipc::playback::PlaybackCommand::Play(index),
            PlaybackCommand::Pause => chilen_ipc::playback::PlaybackCommand::Pause,
            PlaybackCommand::Stop => chilen_ipc::playback::PlaybackCommand::Stop,
            PlaybackCommand::TogglePlaying => chilen_ipc::playback::PlaybackCommand::TogglePlaying,
            PlaybackCommand::GetPlaybackState => {
                chilen_ipc::playback::PlaybackCommand::GetPlaybackState
            }
            PlaybackCommand::SetQueue { tracks, playlist } => {
                if let Some(tracks) = tracks {
                    return chilen_ipc::playback::PlaybackCommand::SetQueue(tracks);
                } else if let Some(playlist) = playlist {
                    return chilen_ipc::playback::PlaybackCommand::SetPlaylist(playlist);
                }
                panic!("This should never happen :)");
            }
            PlaybackCommand::ClearQueue => {
                chilen_ipc::playback::PlaybackCommand::SetQueue(Vec::new())
            }
            PlaybackCommand::AppendToQueue { tracks, playlist } => {
                if let Some(tracks) = tracks {
                    return chilen_ipc::playback::PlaybackCommand::AppendToQueue(tracks);
                } else if let Some(playlist) = playlist {
                    return chilen_ipc::playback::PlaybackCommand::AppendPlaylist(playlist);
                }
                panic!("This should never happen :)");
            }
            PlaybackCommand::GetCurrentTrack => {
                chilen_ipc::playback::PlaybackCommand::GetCurrentTrack
            }
            PlaybackCommand::Next => chilen_ipc::playback::PlaybackCommand::Next,
            PlaybackCommand::Previous => chilen_ipc::playback::PlaybackCommand::Previous,
            PlaybackCommand::SetLoopState { loop_state } => {
                chilen_ipc::playback::PlaybackCommand::SetLoopState(loop_state.into())
            }
            PlaybackCommand::GetLoopState => chilen_ipc::playback::PlaybackCommand::GetLoopState,
            PlaybackCommand::SetRate { rate } => {
                chilen_ipc::playback::PlaybackCommand::SetRate(rate.into())
            }
            PlaybackCommand::GetRate => chilen_ipc::playback::PlaybackCommand::GetRate,
            PlaybackCommand::SetShuffleState { shuffle_state } => {
                chilen_ipc::playback::PlaybackCommand::SetShuffleState(shuffle_state.into())
            }
            PlaybackCommand::GetShuffleState => {
                chilen_ipc::playback::PlaybackCommand::GetShuffleState
            }
            PlaybackCommand::SetPlayerPosition { position_secs } => {
                chilen_ipc::playback::PlaybackCommand::SetPlayerPosition(Duration::from_secs(
                    position_secs,
                ))
            }
            PlaybackCommand::SeekForward { secs } => chilen_ipc::playback::PlaybackCommand::Seek(
                SignedDuration::from_secs(secs.try_into().unwrap_or(i64::MAX)),
            ),
            PlaybackCommand::SeekBackward { secs } => chilen_ipc::playback::PlaybackCommand::Seek(
                SignedDuration::from_secs(-secs.try_into().unwrap_or(i64::MAX)),
            ),
            PlaybackCommand::GetPlayerPosition => {
                chilen_ipc::playback::PlaybackCommand::GetPlayerPosition
            }
            PlaybackCommand::SetVolume { volume } => {
                chilen_ipc::playback::PlaybackCommand::SetPlayerVolume(
                    chilen_ipc::playback::PlayerVolume::new(volume),
                )
            }
            PlaybackCommand::GetVolume => chilen_ipc::playback::PlaybackCommand::GetPlayerVolume,
            PlaybackCommand::OpenUri { uri } => chilen_ipc::playback::PlaybackCommand::OpenURI(uri),
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
    /// Manage the library
    Lib {
        #[command(subcommand)]
        command: LibraryCommand,
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

impl From<SocketType> for chilen_ipc::SocketType {
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
        .filter_module("symphonia_bundle_flac", foreign_module_filter)
        .init();

    trace!("Finished parsing command line arguments");

    args
}
