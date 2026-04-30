use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

/// Struct representing a track from the [music library](crate::library::MusicLibrary).
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

/// Struct representing a playlist in the [music library](crate::library::MusicLibrary).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Playlist {
    /// The name of the playlist.
    ///
    /// Playlist names are unique.
    pub name: String,
    /// The content of the playlist.
    ///
    /// All tracks must already be in the [music library](crate::library::MusicLibrary). If a track
    /// is removed from the library (eg. by removing an audio file from the music directory and
    /// reloading the library), it will also be removed from all the playlists.
    pub tracks: Vec<Track>,
}

/// Struct representing the contents of the music library.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MusicLibrary {
    /// The list of [playlists](Playlist) in the music library.
    pub playlists: Vec<Playlist>,
    /// The list of all [tracks](Track) in the music library.
    pub tracks: Vec<Track>,
}

/// Subcommand of [Command](crate::Command) for managing the music library.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LibraryCommand {
    /// Create a new playlist, optionally with some tracks in it.
    ///
    /// The `daemon` will respond to this with [Response::Ok](crate::Response::Ok) if successful.
    NewPlaylist {
        /// The name for the new playlist, must not already exist in the music library.
        ///
        /// If a playlist with the specified name already exists
        /// [LibraryError::PlaylistExists] will be returned.
        name: String,
        /// Optional list of paths to tracks to be added to the playlist.
        ///
        /// **Note:** the `daemon` will return an error if any of the tracks are not registered in
        /// the music library or if the vector contains duplicates.
        tracks: Option<Vec<PathBuf>>,
    },
    /// Import a playlist from an M3U8 file.
    ///
    /// This is currently unimplemented and will cause the `daemon` to panic every time.
    PlaylistFromM3U8 {
        /// The name for the imported playlist, must not already exist in the music library.
        ///
        /// If left unspecified, it will be derived from the name of the imported file.
        ///
        /// If a playlist with the specified name already exists
        /// [LibraryError::PlaylistExists] will be returned.
        name: Option<String>,
        /// The path to the M3U8 file to import.
        m3u8_file: PathBuf,
    },
    /// Delete playlists from the music library.
    ///
    /// The `daemon` will respond to this with [Response::Ok](crate::Response::Ok) if successful.
    DeletePlaylists {
        /// List of the playlists to delete.
        ///
        /// If any of the provided playlists don't exist in the music library,
        /// [LibraryError::NoSuchPlaylist] will be returned, and no changes to the music
        /// library will be made.
        names: Vec<String>,
    },
    /// Add tracks to an already existing playlist.
    ///
    /// The `daemon` will respond to this with [Response::Ok](crate::Response::Ok) if successful.
    AddTracksToPlaylist {
        /// The name of the playlist to add tracks to.
        ///
        /// If a playlist with the specified name doesn't exist in the music library,
        /// [LibraryError::NoSuchPlaylist] will be returned.
        name: String,
        /// List of paths to tracks to add to the playlist.
        ///
        /// **Note:** the `daemon` will return an error if any of the tracks are not registered in
        /// the music library or if the vector contains duplicates.
        tracks: Vec<PathBuf>,
    },
    /// Remove tracks from a playlist.
    ///
    /// The `daemon` will respond to this with [Response::Ok](crate::Response::Ok) if successful.
    RemoveTracksFromPlaylist {
        /// The name of the playlist to remove tracks from.
        ///
        /// If a playlist with the specified name doesn't exist in the music library,
        /// [LibraryError::NoSuchPlaylist] will be returned.
        name: String,
        /// The list of track indices in the playlist to remove.
        ///
        /// Eg. to remove the first track you would pass `[0]`, to remove the first three
        /// `[0, 1, 2]`, etc.
        ///
        /// If one of the indices is out of range, the daemon will return
        /// [LibraryError::IndexOutOfBounds]. No changes will be made.
        ids: Vec<usize>,
    },
    /// Get the contents of the [music library](crate::library::MusicLibrary).
    ///
    /// The `daemon` will respond to this with [Response::Library](crate::Response::Library) if
    /// successful.
    GetLibrary,
    /// Reload the library and rebuild the cache ignoring already cached covers.
    ///
    /// Will take more time than just reloading the cache.
    ///
    /// The `daemon` will respond to this with [Response::Ok](crate::Response::Ok) if successful.
    Rebuild,
    /// Reload the library using cached data if possible.
    ///
    /// This can be used to discover newly added tracks.
    ///
    /// The `daemon` will respond to this with
    /// [Response::Ok](crate::Response::Ok)(crate::Response::Ok) if successful.
    Reload,
}

/// An error originating from the music library module of the `daemon`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LibraryError {
    /// Could not complete the operation because a [playlist](Playlist) with the provided name
    /// already exists.
    PlaylistExists,
    /// Could not perform the operation because the music library is not initialized.
    ///
    /// This can happen if a command is sent to early and the music library is not yet initialized.
    LibraryNotInitialized,
    /// There is not playlist in the music library with the provided name.
    NoSuchPlaylist,
    /// Could not get the path to the cache directory or the cache is unusable.
    CacheError,
    /// The provided item index was out of bounds.
    IndexOutOfBounds,
    /// The provided vector contained duplicate values.
    DuplicateItems,
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaylistExists => write!(f, "Playlist with this name already exists"),
            Self::LibraryNotInitialized => write!(f, "The music library is not initialized"),
            Self::NoSuchPlaylist => write!(f, "There is no playlist with this name"),
            Self::CacheError => write!(f, "Cache is unusable"),
            Self::IndexOutOfBounds => write!(f, "The provided item index was out of bounds"),
            Self::DuplicateItems => write!(f, "The provided vector contained duplicate values"),
        }
    }
}
