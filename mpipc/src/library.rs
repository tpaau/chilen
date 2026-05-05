use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

/// Struct representing a track from the [music library](MusicLibrary).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Track {
    /// The path to the audio file.
    pub path: PathBuf,
    /// The path to the extracted cover file.
    pub cover_path: Option<PathBuf>,
    /// The duration of the track.
    pub duration: Duration,
    // TODO: Add an option to split the tag with separators like ",", ";", "/", etc.
    /// The track artist.
    pub artist: Option<String>,
    /// The track title.
    pub title: Option<String>,
    /// The track album.
    pub album: Option<String>,
    // TODO: Same as with artist
    /// The track genre.
    pub genre: Option<String>,
    /// Possibly synchronized lyrics text.
    pub lyrics: Option<String>,
    /// Contents of the comment tag.
    pub comment: Option<String>,
    pub track: Option<u32>,
    pub track_total: Option<u32>,
    pub disc: Option<u32>,
    pub disc_total: Option<u32>,
    /// Release year.
    pub year: Option<u32>,
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} ({})",
            self.artist.clone().unwrap_or(String::from("Unknown")),
            self.title.clone().unwrap_or(String::from("Unknown")),
            self.path.to_string_lossy()
        )
    }
}

/// Struct representing a playlist in the [music library](MusicLibrary).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Playlist {
    /// The name of the playlist.
    ///
    /// Playlist names are unique.
    pub name: String,
    /// The content of the playlist.
    ///
    /// All tracks must already be in the [music library](MusicLibrary). If a track is removed from
    /// the library (eg. by removing an audio file from the music directory and reloading the
    /// library), it will also be removed from all the playlists.
    pub tracks: Vec<Track>,
}

/// Struct representing the contents of the music library.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MusicLibrary {
    /// The list of playlists in the music library.
    pub playlists: Vec<Playlist>,
    /// The list of all tracks in the music library.
    pub tracks: Vec<Track>,
}

/// Subcommand of [`Command`](crate::Command) for managing the music library.
///
/// The expected response may be different depending on the command sent. If it isn't specified in
/// the variant documentation, assume [`Response::Ok`](crate::Response::Ok) is the expected
/// response.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LibraryCommand {
    /// Create a new playlist, optionally with some tracks in it.
    NewPlaylist {
        /// The name for the new playlist. Must not already exist in the music library.
        ///
        /// If a playlist with the specified name already exists,
        /// [`LibraryError::PlaylistExists`] will be returned, and no changes to the
        /// [music library](MusicLibrary) will be made.
        name: String,
        /// Optional list of paths to tracks to be added to the playlist.
        ///
        /// This command will fail if any of the tracks are not registered in the
        /// [music library](MusicLibrary), or if the list contains duplicates.
        tracks: Option<Vec<PathBuf>>,
    },
    /// Import a playlist from an M3U8 file.
    ///
    /// This is currently unimplemented and will cause the `daemon` to panic.
    PlaylistFromM3U8 {
        /// The name for the imported playlist. Must not already exist in the music library.
        ///
        /// If a playlist with the specified name already exists,
        /// [`LibraryError::PlaylistExists`] will be returned, and no changes to the
        /// [music library](MusicLibrary) will be made.
        name: String,
        /// The path to the M3U8 file to import.
        m3u8_file: PathBuf,
    },
    /// Delete playlists from the music library.
    DeletePlaylists {
        /// List of the playlists to delete.
        ///
        /// If any of the provided playlists don't exist in the music library,
        /// [`LibraryError::NoSuchPlaylist`] will be returned, and no changes to the
        /// [music library](MusicLibrary) will be made.
        names: Vec<String>,
    },
    /// Add tracks to an already existing playlist.
    AddTracksToPlaylist {
        /// The name of the playlist to add tracks to.
        ///
        /// If a playlist with the specified name doesn't exist in the music library,
        /// [`LibraryError::NoSuchPlaylist`] will be returned, and no changes to the
        /// [music library](MusicLibrary) will be made.
        name: String,
        /// List of paths to tracks to add to the playlist.
        ///
        /// **Note:** the `daemon` will return an error if any of the tracks are not registered in
        /// the music library or if the list contains duplicates.
        tracks: Vec<PathBuf>,
    },
    /// Remove tracks from a playlist.
    RemoveTracksFromPlaylist {
        /// The name of the playlist to remove tracks from.
        ///
        /// If a playlist with the specified name doesn't exist in the music library,
        /// [`LibraryError::NoSuchPlaylist`] will be returned, and no changes to the
        /// [music library](MusicLibrary) will be made.
        name: String,
        /// The list of track indices in the playlist to remove.
        ///
        /// Eg. to remove the first track you would pass `[0]`, to remove the first three
        /// `[0, 1, 2]`, etc.
        ///
        /// If one or more of the indices is out of range, [`LibraryError::IndexOutOfBounds`]
        /// will be returned, and no changes to the [music library](MusicLibrary) will be made.
        ids: Vec<usize>,
    },
    /// Get the contents of the [music library](MusicLibrary).
    ///
    /// The `daemon` will respond to this with [`Response::Library`](crate::Response::Library) if
    /// successful.
    GetLibrary,
    /// Reload the library and rebuild the cache ignoring already cached covers.
    ///
    /// Will take more time than just reloading the cache.
    Rebuild,
    /// Reload the library using cached data if possible.
    ///
    /// This can be used to discover newly added tracks.
    Reload,
}

/// An error originating from the music library module of the `daemon`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LibraryError {
    /// Could not complete the operation because a [playlist](Playlist) with the provided name
    /// already exists.
    PlaylistExists,
    /// Could not perform the operation because the [music library](MusicLibrary) is not
    /// initialized.
    ///
    /// This can happen if a command is sent to early and the music library is not yet initialized.
    LibraryNotInitialized,
    /// There is no [playlist](Playlist) in the [music library](MusicLibrary) with the provided
    /// name.
    NoSuchPlaylist,
    /// The provided item index was out of bounds.
    IndexOutOfBounds,
    /// The provided list contained duplicate values.
    DuplicateItems,
    /// The provided track is not registered in the library.
    NoSuchTrack,
    /// Could not read the contents of the library state file.
    StateNotReadable,
    /// Could not write the library state to a file.
    StateWriteFailed,
    /// The library state path is not a file.
    StateNotAFile,
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaylistExists => write!(f, "Playlist with this name already exists"),
            Self::LibraryNotInitialized => write!(f, "The music library is not initialized"),
            Self::NoSuchPlaylist => write!(f, "There is no playlist with this name"),
            Self::IndexOutOfBounds => write!(f, "The provided item index was out of bounds"),
            Self::NoSuchTrack => {
                write!(f, "The provided track is not registered in the library")
            }
            Self::DuplicateItems => write!(f, "The provided vector contained duplicate values"),
            Self::StateNotReadable => {
                write!(f, "Could not read the contents of the library state file")
            }
            Self::StateWriteFailed => write!(f, "Could not write the library state to a file"),
            Self::StateNotAFile => write!(f, "The library state path is not a file"),
        }
    }
}
