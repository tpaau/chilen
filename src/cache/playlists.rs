use std::{path::PathBuf, sync::RwLock};

use bincode::{Decode, Encode};
use log::trace;
use mpipc::PlaylistError;
use serde::{Deserialize, Serialize};

use crate::{
    indexer::{IndexingError, index},
    track::Track,
};

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
struct ConfPlaylist {
    pub name: String,
    pub track_hashes: Vec<i32>,
}

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
struct ConfMusicLibrary {
    pub playlists: Vec<ConfPlaylist>,
}

pub trait Playable {}

#[derive(Clone, Debug)]
pub struct Playlist<'a> {
    pub name: String,
    pub tracks: Vec<&'a Track>,
}

#[derive(Clone, Debug)]
pub struct MusicLibrary<'a> {
    pub playlists: Vec<Playlist<'a>>,
    pub tracks: Vec<Track>,
}

impl MusicLibrary<'_> {
    pub fn get_playlist_with_name(&'_ self, name: &str) -> Option<&'_ Playlist> {
        self.playlists
            .iter()
            .find(|&playlist| playlist.name == name)
    }
}

/// Error indicating an issue with loading or handling the music library.
pub enum MusicLibraryError {
    IndexingError { e: IndexingError },
}

static MUSIC_LIBRARY: RwLock<Option<MusicLibrary<'static>>> = RwLock::new(None);

/// Save the library state to a file.
fn save_library() -> Result<(), MusicLibraryError> {
    Ok(())
}

/// Load the music library from the playlists file.
pub fn load<'a>() -> Result<MusicLibrary<'a>, MusicLibraryError> {
    trace!("Loading the music library");

    let tracks = match index::<String>(None) {
        Ok(tracks) => tracks,
        Err(e) => {
            return Err(MusicLibraryError::IndexingError { e });
        }
    };

    trace!("Received tracks from the indexing thread, loading the playlist file");

    Ok(MusicLibrary {
        playlists: Vec::new(),
        tracks,
    })
}

pub fn create_playlist(name: &str, tracks: &Option<Vec<PathBuf>>) -> Result<(), PlaylistError> {
    let lib = &*MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = lib {
        if lib.get_playlist_with_name(name).is_none() {
            todo!()
        } else {
            Err(PlaylistError::PlaylistExists)
        }
    } else {
        Err(PlaylistError::LibraryNotInitialized)
    }
}

pub fn create_playlist_from_m3u8<T: Into<PathBuf>>(
    name: &str,
    m3u8_file: &T,
) -> Result<(), PlaylistError> {
    let lib = &*MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = lib {
        if lib.get_playlist_with_name(name).is_none() {
            todo!()
        } else {
            Err(PlaylistError::PlaylistExists)
        }
    } else {
        Err(PlaylistError::LibraryNotInitialized)
    }
}

pub fn delete_playlist(name: &str) -> Result<(), PlaylistError> {
    let lib = &*MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = lib {
        if lib.get_playlist_with_name(name).is_some() {
            todo!()
        } else {
            Err(PlaylistError::NoSuchPlaylist)
        }
    } else {
        Err(PlaylistError::LibraryNotInitialized)
    }
}
