use std::{
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock, mpsc},
    thread,
};

use icu::{
    collator::{Collator, CollatorBorrowed, options::CollatorOptions},
    locale::Locale,
};
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::{
    music_lib::{MusicLibrary, set_dirs},
    playback::state::PlayerState,
};

pub mod music_lib;
pub mod playback;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    /// A friendly name to identify the media player to users (eg: “VLC media player”).
    pub identity: String,
    /// The bus name suffix to be used with MPRIS.
    ///
    /// The resulting bus name will be `org.mpris.MediaPlayer2.<bus_name_suffix>`, where
    /// `<bus_name_suffix>` must be a unique identifier, such as one based on a UNIX process id.
    /// For example, this could be:
    ///
    /// - `org.mpris.MediaPlayer2.vlc.instance7389`
    ///
    /// **Note:** According to the D-Bus specification, the unique identifier “must only contain
    /// the ASCII characters \[A-Z\]\[a-z\]\[0-9\]_-” and “must not begin with a digit”.
    #[cfg(feature = "mpris")]
    pub identifier: String,
    pub cache_dir: PathBuf,
    // TODO: Support for multiple music library directories
    pub music_dir: PathBuf,
    pub data_dir: PathBuf,
    pub locale: Locale,
    pub playback: playback::Config,
    pub library: music_lib::Config,
}

#[cfg(test)]
pub(crate) fn testing_init_config() {
    use crate::music_lib::ValueSeparators;

    *crate::CONFIG.write().unwrap() = Some(Arc::new(Config {
        identity: "Chilen".to_string(),
        #[cfg(feature = "mpris")]
        identifier: "com.tpaau.Chilen".to_string(),
        cache_dir: PathBuf::new(),
        music_dir: PathBuf::new(),
        data_dir: PathBuf::new(),
        locale: "en-US".parse().unwrap(),
        playback: playback::Config {
            skip_previous_threshold: None,
        },
        library: crate::music_lib::Config {
            value_separators: ValueSeparators::default(),
            indexer: crate::music_lib::indexer::Config::default(),
        },
    }))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Raise,
    Quit,
    SetFullscreen(bool),
    PlayerStateChanged(PlayerState),
    LoadProgressChanged(music_lib::Progress),
    LibraryLoadFailed(String),
    LibraryChanged(Box<MusicLibrary>),
}

/// Chilen error type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    PathInaccessible(PathBuf),
    PathReadonly(PathBuf),
    NotADirectory(PathBuf),
    DirectoryCreationFailed(PathBuf, String),
    EmptyName,
    /// The audio player is not connected.
    ///
    /// This may happen if the device doesn't have an audio device or none of the audio devices are
    /// marked as default.
    PlayerNotConnected,
    /// The playback state is not initialized.
    StateNotInitialized,
    /// The queue is empty.
    QueueEmpty,
    /// The audio file could not be opened, has an unsupported format or is corrupt.
    SourceError,
    /// The player is already playing.
    PlayerPlaying,
    /// The player is already paused.
    PlayerPaused,
    /// Thrown when a client attempts to stop the player when it was already stopped or when a
    /// client attempts to seek while the player is stopped.
    PlayerStopped,
    /// Seek is not supported for the current audio source.
    SeekNotSupported,
    /// Cannot go to the previous track.
    ///
    /// This means that the current track is first in the queue and the
    /// [loop state](playback::LoopState) is set to [`LoopState::Off`](playback::LoopState::Off).
    CannotGoPrevious,
    /// Cannot go to the next track.
    ///
    /// This means that the current track is last in the queue and the
    /// [loop state](playback::LoopState) is set to [`LoopState::Off`](playback::LoopState).
    CannotGoNext,
    /// No track at this index.
    NoTrackAtIndex(usize),
    /// The player position could not be set because the duration provided was invalid.
    ///
    /// The player will additionally refuse to seek by 0s to prevent audio popping.
    InvalidDuration,
    /// Overflow detected while performing a seek operation.
    DurationOverflow,
    /// Could not complete the operation because a [playlist](music_lib::state::Playlist) with the
    /// provided name already exists.
    PlaylistExists,
    /// Could not perform the operation because the [music library](music_lib::state::MusicLibrary)
    /// is not initialized.
    ///
    /// This can happen if a command is sent to early and the music library is not yet initialized.
    LibraryNotInitialized,
    /// There is no [playlist](music_lib::state::Playlist) in the
    /// [music library](music_lib::state::MusicLibrary) with the provided name.
    UnknownPlaylist(String),
    /// The item index was out of bounds.
    IndexOutOfBounds,
    /// The list contained duplicate values.
    DuplicateItems,
    /// The track was not found in the music library.
    UnknownTrackPath(PathBuf),
    /// Could not read the contents of the library state file.
    StateNotReadable,
    /// Could not write the library state to a file.
    StateWriteFailed,
    /// The library state path is not a file.
    StateNotAFile,
    /// The path does not exist or access to it was denied.
    PathDoesNotExist,
    /// Could not find any audio files in the directory path.
    DirectoryWithNoTracks,
    /// Could not parse the M3U8 playlist.
    ///
    /// Please make sure that the playlist has the correct format and is not corrupted.
    PlaylistParsingError,
    /// Could not export the playlist to M3U8.
    ///
    /// This likely either means that the specified file path doesn't exists or is not writable.
    PlaylistExportFailed,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayerNotConnected => write!(f, "The audio player is not connected"),
            Self::StateNotInitialized => write!(f, "The playback state is not initialized"),
            Self::QueueEmpty => write!(f, "The queue is empty"),
            Self::SourceError => write!(
                f,
                "The audio file could not be opened, has an unsupported format or is corrupt"
            ),
            Self::PlayerPlaying => write!(f, "The player is already playing"),
            Self::PlayerPaused => write!(f, "The player is already paused"),
            Self::PlayerStopped => write!(f, "The player is stopped"),
            Self::SeekNotSupported => write!(f, "Seek is not supported"),
            Self::CannotGoPrevious => write!(f, "Cannot go to the previous track"),
            Self::CannotGoNext => write!(f, "Cannot go to the next track"),
            Self::NoTrackAtIndex(index) => write!(f, "No track was found at index {index}"),
            Self::InvalidDuration => write!(
                f,
                "The player position could not be set because the duration was invalid"
            ),
            Self::DurationOverflow => {
                write!(f, "Overflow detected while performing a seek operation")
            }
            Self::UnknownTrackPath(path) => {
                write!(
                    f,
                    "The track track doesn't exist the music library: {path:?}"
                )
            }
            Self::PlaylistExists => write!(f, "Playlist with this name already exists"),
            Self::LibraryNotInitialized => write!(f, "The music library is not initialized"),
            Self::UnknownPlaylist(name) => {
                write!(f, "There is no playlist with the name \"{name}\"")
            }
            Self::IndexOutOfBounds => write!(f, "The item index was out of bounds"),
            Self::DuplicateItems => write!(f, "The vector contained duplicate values"),
            Self::StateNotReadable => {
                write!(f, "Could not read the contents of the library state file")
            }
            Self::StateWriteFailed => write!(f, "Could not write the library state to a file"),
            Self::StateNotAFile => write!(f, "The library state path is not a file"),
            Self::PathDoesNotExist => {
                write!(f, "The path does not exist or access to it was denied")
            }
            Self::DirectoryWithNoTracks => {
                write!(f, "Could not find any audio files in the directory path")
            }
            Self::PlaylistParsingError => write!(f, "Could not parse the M3U8 playlist"),
            Self::PlaylistExportFailed => write!(f, "Could not export the playlist to M3U8"),
            Error::PathInaccessible(path_buf) => write!(f, "Path inaccessible: {path_buf:?}"),
            Error::PathReadonly(path_buf) => write!(f, "Path is readonly: {path_buf:?}"),
            Error::NotADirectory(path_buf) => {
                write!(f, "Expected this path to be a directory: {path_buf:?}")
            }
            Error::DirectoryCreationFailed(path_buf, e) => {
                write!(f, "Could not create a directory at {path_buf:?}: {e}")
            }
            Error::EmptyName => write!(f, "The name is empty"),
        }
    }
}

fn send_event(event: Event) {
    if let Some(sender) = EVENT_SENDER.read().unwrap().as_ref() {
        if let Err(e) = sender.send(event) {
            error!("Could not send the event: {e}");
        }
    } else {
        warn!(
            "The event sender is not initialized - this is expected during testing, in production this would be a serious bug though"
        )
    }
}

pub(crate) static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static CONFIG: RwLock<Option<Arc<Config>>> = RwLock::new(None);
pub(crate) static COLLATOR: RwLock<Option<Arc<CollatorBorrowed<'_>>>> = RwLock::new(None);

pub(crate) fn get_config() -> Arc<Config> {
    let guard = CONFIG.read().unwrap();
    guard.as_ref().unwrap().clone()
}

// TODO: Unused cached cover art cleanup here somewhere
pub fn init(config: Config) -> Result<mpsc::Receiver<Event>, Error> {
    let (sender, receiver) = mpsc::channel();
    *EVENT_SENDER.write().unwrap() = Some(sender);

    thread::spawn(|| {
        *CONFIG.write().unwrap() = Some(Arc::new(config.clone()));
        *COLLATOR.write().unwrap() =
            match Collator::try_new(config.locale.clone().into(), CollatorOptions::default()) {
                Ok(c) => Some(Arc::new(c)),
                Err(e) => {
                    error!("Could not initialize the collator, things will be unordered: {e}");
                    None
                }
            };

        if let Err(e) = set_dirs(config.data_dir, config.cache_dir, config.music_dir) {
            error!("Could not set the initial directories: {e}");
            send_event(Event::LoadProgressChanged(music_lib::Progress::Failed(e)));
            return;
        }
        if let Err(e) = music_lib::load(config.library) {
            error!("Could not load the music library: {e}");
            send_event(Event::LoadProgressChanged(music_lib::Progress::Failed(e)));
            return;
        }
        playback::init(
            #[cfg(feature = "mpris")]
            config.identity,
            #[cfg(feature = "mpris")]
            config.identifier,
        )
    });

    Ok(receiver)
}
