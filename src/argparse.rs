use std::{path::PathBuf, time::Duration};

use clap::{ArgGroup, Parser, Subcommand, ValueEnum, ValueHint};

use env_logger::Builder;
use log::{self, trace};
use mpipc::playback::SignedDuration;

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
        /// Allow clients to modify the playback rate of the player.
        #[arg(long, short, default_value_t = false)]
        allow_rate_modification: bool,
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

impl TryFrom<DaemonCommand> for mpipc::Command {
    type Error = String;
    fn try_from(value: DaemonCommand) -> Result<Self, Self::Error> {
        match value {
            DaemonCommand::Ping => Ok(mpipc::Command::Ping),
            _ => Err("Cannot convert variant {value:?} to a `mpipc::Command`".to_string()),
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

impl From<PlaylistCommand> for mpipc::library::LibraryCommand {
    fn from(value: PlaylistCommand) -> Self {
        match value {
            PlaylistCommand::New { name, tracks } => {
                mpipc::library::LibraryCommand::NewPlaylist { name, tracks }
            }
            PlaylistCommand::FromM3U8 { name, m3u8_file } => {
                mpipc::library::LibraryCommand::PlaylistFromM3U8 { name, m3u8_file }
            }
            PlaylistCommand::Delete { names } => {
                mpipc::library::LibraryCommand::DeletePlaylists { names }
            }
            PlaylistCommand::List { full: _, debug: _ } => {
                mpipc::library::LibraryCommand::GetLibrary
            }
            PlaylistCommand::AddTracks { name, tracks } => {
                mpipc::library::LibraryCommand::AddTracksToPlaylist { name, tracks }
            }
            PlaylistCommand::RemoveTracks { name, ids } => {
                mpipc::library::LibraryCommand::RemoveTracksFromPlaylist { name, ids }
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

impl From<LibraryCommand> for mpipc::library::LibraryCommand {
    fn from(value: LibraryCommand) -> Self {
        match value {
            LibraryCommand::Playlist { command } => command.into(),
            LibraryCommand::Reload => mpipc::library::LibraryCommand::Reload,
            LibraryCommand::Rebuild => mpipc::library::LibraryCommand::Rebuild,
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

impl From<LoopState> for mpipc::playback::LoopState {
    fn from(value: LoopState) -> Self {
        match value {
            LoopState::Off => mpipc::playback::LoopState::Off,
            LoopState::Track => mpipc::playback::LoopState::Track,
            LoopState::Playlist => mpipc::playback::LoopState::Playlist,
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

impl From<ShuffleState> for mpipc::playback::ShuffleState {
    fn from(value: ShuffleState) -> Self {
        match value {
            ShuffleState::On => mpipc::playback::ShuffleState::On,
            ShuffleState::Off => mpipc::playback::ShuffleState::Off,
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
}

impl From<PlaybackCommand> for mpipc::playback::PlaybackCommand {
    fn from(value: PlaybackCommand) -> Self {
        match value {
            PlaybackCommand::Play { index } => mpipc::playback::PlaybackCommand::Play(index),
            PlaybackCommand::Pause => mpipc::playback::PlaybackCommand::Pause,
            PlaybackCommand::Stop => mpipc::playback::PlaybackCommand::Stop,
            PlaybackCommand::TogglePlaying => mpipc::playback::PlaybackCommand::TogglePlaying,
            PlaybackCommand::GetPlaybackState => mpipc::playback::PlaybackCommand::GetPlaybackState,
            PlaybackCommand::SetQueue { tracks, playlist } => {
                if let Some(tracks) = tracks {
                    return mpipc::playback::PlaybackCommand::SetQueue(tracks);
                } else if let Some(playlist) = playlist {
                    return mpipc::playback::PlaybackCommand::SetPlaylist(playlist);
                }
                panic!("This should never happen :)");
            }
            PlaybackCommand::ClearQueue => mpipc::playback::PlaybackCommand::SetQueue(Vec::new()),
            PlaybackCommand::AppendToQueue { tracks, playlist } => {
                if let Some(tracks) = tracks {
                    return mpipc::playback::PlaybackCommand::AppendToQueue(tracks);
                } else if let Some(playlist) = playlist {
                    return mpipc::playback::PlaybackCommand::AppendPlaylist(playlist);
                }
                panic!("This should never happen :)");
            }
            PlaybackCommand::GetCurrentTrack => mpipc::playback::PlaybackCommand::GetCurrentTrack,
            PlaybackCommand::Next => mpipc::playback::PlaybackCommand::Next,
            PlaybackCommand::Previous => mpipc::playback::PlaybackCommand::Previous,
            PlaybackCommand::SetLoopState { loop_state } => {
                mpipc::playback::PlaybackCommand::SetLoopState(loop_state.into())
            }
            PlaybackCommand::GetLoopState => mpipc::playback::PlaybackCommand::GetLoopState,
            PlaybackCommand::SetRate { rate } => {
                mpipc::playback::PlaybackCommand::SetRate(rate.into())
            }
            PlaybackCommand::GetRate => mpipc::playback::PlaybackCommand::GetRate,
            PlaybackCommand::SetShuffleState { shuffle_state } => {
                mpipc::playback::PlaybackCommand::SetShuffleState(shuffle_state.into())
            }
            PlaybackCommand::GetShuffleState => mpipc::playback::PlaybackCommand::GetShuffleState,
            PlaybackCommand::SetPlayerPosition { position_secs } => {
                mpipc::playback::PlaybackCommand::SetPlayerPosition(Duration::from_secs(
                    position_secs,
                ))
            }
            PlaybackCommand::SeekForward { secs } => mpipc::playback::PlaybackCommand::Seek(
                SignedDuration::from_secs(secs.try_into().unwrap_or(i64::MAX)),
            ),
            PlaybackCommand::SeekBackward { secs } => mpipc::playback::PlaybackCommand::Seek(
                SignedDuration::from_secs(-secs.try_into().unwrap_or(i64::MAX)),
            ),
            PlaybackCommand::GetPlayerPosition => {
                mpipc::playback::PlaybackCommand::GetPlayerPosition
            }
            PlaybackCommand::SetVolume { volume } => {
                mpipc::playback::PlaybackCommand::SetPlayerVolume(
                    mpipc::playback::PlayerVolume::new(volume),
                )
            }
            PlaybackCommand::GetVolume => mpipc::playback::PlaybackCommand::GetPlayerVolume,
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

impl TryFrom<Command> for mpipc::Command {
    type Error = String;
    fn try_from(value: Command) -> Result<Self, Self::Error> {
        match value {
            Command::Daemon { command } => command.try_into(),
            #[cfg(feature = "gui")]
            Command::Gui { command: _ } => {
                Err("Cannot convert the `GuiCommand variant".to_string())
            }
            Command::Lib { command } => Ok(mpipc::Command::Library(command.into())),
            Command::Playback { command } => Ok(mpipc::Command::Playback(command.into())),
        }
    }
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
        .filter_module("symphonia_bundle_flac", foreign_module_filter)
        .init();

    trace!("Finished parsing command line arguments");

    args
}
