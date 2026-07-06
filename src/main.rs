mod argparse;
mod gui;
pub mod music_lib;
mod playback;
#[cfg(test)]
mod tests;

use std::{env::home_dir, process::exit, thread};

use dirs::{cache_dir, data_dir};

use log::{error, info};

use crate::{
    argparse::parse_args,
    music_lib::{covers::LoadMode, set_dirs},
};

/// Error related to the daemon.
///
/// Can either originate from a [`Response`] or from a function in this crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// Quit requests from external clients are not allowed.
    QuitDisabled,
    /// The audio player is not connected.
    ///
    /// This may happen if the device doesn't have an audio device or none of the audio devices are
    /// marked as default.
    PlayerNotConnected,
    /// The playback state is not initialized.
    ///
    /// This error may occur when a [`PlaybackCommand`] is sent to the daemon too early, before the
    /// state is restored from cache.
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
    /// The daemon was not built with shuffle support.
    ShuffleNotSupported,
    /// No track at this index.
    NoTrackAtIndex(usize),
    /// The specified rate value was out of the allowed range.
    RateOutOfRange,
    /// The modification of the playback rate is not allowed.
    FixedRate,
    /// The player position could not be set because the duration provided was invalid.
    ///
    /// The player will additionally refuse to seek by 0s to prevent audio popping.
    InvalidDuration,
    /// Overflow detected while performing a seek operation.
    DurationOverflow,
    /// Could not complete the operation because a [playlist](library::Playlist) with the provided
    /// name already exists.
    PlaylistExists,
    /// Could not perform the operation because the [music library](MusicLibrary) is not
    /// initialized.
    ///
    /// This can happen if a command is sent to early and the music library is not yet initialized.
    LibraryNotInitialized,
    /// There is no [playlist](library::Playlist) in the [music library](MusicLibrary) with the
    /// provided name.
    UnknownPlaylist,
    /// The provided item index was out of bounds.
    IndexOutOfBounds,
    /// The provided list contained duplicate values.
    DuplicateItems,
    /// The provided track is not registered in the library.
    UnknownTrack,
    /// Could not read the contents of the library state file.
    StateNotReadable,
    /// Could not write the library state to a file.
    StateWriteFailed,
    /// The library state path is not a file.
    StateNotAFile,
    /// The provided path does not exist or access to it was denied.
    PathDoesNotExist,
    /// Could not check if the provided file exists.
    PathExistenceUnknown,
    /// Could not find any audio files in the provided directory path.
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
            Self::QuitDisabled => write!(f, "Quit requests from external clients are not allowed"),
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
            Self::ShuffleNotSupported => write!(f, "The daemon was not built with shuffle support"),
            Self::NoTrackAtIndex(index) => write!(f, "No track was found at index {index}"),
            Self::RateOutOfRange => {
                write!(f, "The specified rate value was out of the allowed range")
            }
            Self::FixedRate => write!(f, "The modification of the playback rate is disallowed"),
            Self::InvalidDuration => write!(
                f,
                "The player position could not be set because the duration provided was invalid"
            ),
            Self::DurationOverflow => {
                write!(f, "Overflow detected while performing a seek operation")
            }
            Self::UnknownTrack => {
                write!(f, "The provided track was not found in the music library")
            }
            Self::PlaylistExists => write!(f, "Playlist with this name already exists"),
            Self::LibraryNotInitialized => write!(f, "The music library is not initialized"),
            Self::UnknownPlaylist => write!(f, "There is no playlist with this name"),
            Self::IndexOutOfBounds => write!(f, "The provided item index was out of bounds"),
            Self::DuplicateItems => write!(f, "The provided vector contained duplicate values"),
            Self::StateNotReadable => {
                write!(f, "Could not read the contents of the library state file")
            }
            Self::StateWriteFailed => write!(f, "Could not write the library state to a file"),
            Self::StateNotAFile => write!(f, "The library state path is not a file"),
            Self::PathDoesNotExist => write!(
                f,
                "The provided path does not exist or access to it was denied"
            ),
            Self::PathExistenceUnknown => write!(f, "Could not check if the provided file exists"),
            Self::DirectoryWithNoTracks => write!(
                f,
                "Could not find any audio files in the provided directory path"
            ),
            Self::PlaylistParsingError => write!(f, "Could not parse the M3U8 playlist"),
            Self::PlaylistExportFailed => write!(f, "Could not export the playlist to M3U8"),
        }
    }
}

fn main() {
    let args = parse_args();

    let data_dir = match args.data_dir {
        Some(dir) => dir,
        None => match data_dir() {
            Some(dir) => dir,
            None => {
                error!("Could not get the path to the data directory");
                exit(1);
            }
        },
    };
    let cache_dir = match args.cache_dir {
        Some(dir) => dir,
        None => match cache_dir() {
            Some(dir) => dir,
            None => {
                error!("Could not get the path to the data directory");
                exit(1);
            }
        },
    };
    let music_dir = match args.music_dir {
        Some(dir) => dir,
        None => {
            let mut dir = match home_dir() {
                Some(home) => home,
                None => {
                    error!("Could not get the path to the home directory");
                    exit(1);
                }
            };
            dir.push("Music");
            dir
        }
    };

    thread::spawn(|| {
        if let Err(e) = set_dirs(data_dir, cache_dir, music_dir) {
            error!("Could not set the initial directories: {e}");
            exit(1)
        }
        if let Err(e) = music_lib::state::load(LoadMode::Load) {
            error!("Could not load the music library: {e}");
            exit(1)
        }
    });

    thread::spawn(|| {
        playback::init(
            #[cfg(feature = "mpris")]
            "Chilen".to_string(),
            #[cfg(feature = "mpris")]
            "dev.tpaau.Chilen".to_string(),
        );
    });

    match gui::start() {
        Ok(_) => info!("Main window closed, exiting"),
        Err(e) => {
            error!("GUI stopped unexpectedly: {e}");
            exit(1);
        }
    }
}
