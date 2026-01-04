use std::{
    path::PathBuf,
    sync::RwLock,
    time::{Duration, SystemTime},
};

use bincode::{Decode, Encode};
use log::{error, trace};
use mpipc::MusicLibraryError;
use serde::{Deserialize, Serialize};

use crate::{
    indexer::{index, index_files},
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

static MUSIC_LIBRARY: RwLock<Option<MusicLibrary<'static>>> = RwLock::new(None);

/// Save the library state to a file.
fn save_library() -> Result<(), MusicLibraryError> {
    Ok(())
}

fn get_borrowed_tracks<'a>(tracks: Vec<Track>) -> Result<Vec<&'a Track>, MusicLibraryError> {
    Err(MusicLibraryError::ArcInnerError)
}

/// Load the music library from the playlists file.
pub fn load<'a>() -> Result<MusicLibrary<'a>, MusicLibraryError> {
    trace!("Loading the music library");

    let time_start = SystemTime::now();

    let tracks = match index(None) {
        Ok(tracks) => tracks,
        Err(e) => {
            return Err(e);
        }
    };

    let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));
    trace!(
        "Finished indexing the music directory in {:.2}s, found {} audio files",
        time_elapsed.as_secs_f64(),
        tracks.len()
    );

    trace!("Received tracks from the indexing thread, loading the playlist file");

    Ok(MusicLibrary {
        playlists: Vec::new(),
        tracks,
    })
}

pub fn create_playlist(name: String, tracks: &Option<Vec<PathBuf>>) -> Result<(), MusicLibraryError> {
    trace!("Creating a new playlist with name {name} from a list tracks");

    let lib = MUSIC_LIBRARY.write().unwrap().clone();
    if let Some(mut lib) = lib {
        if lib.get_playlist_with_name(&name).is_none() {
            let tracks = if let Some(tracks) = tracks {
                match index_files(tracks.to_vec()) {
                    Ok(tracks) => tracks,
                    Err(e) => {
                        error!("Got an error while indexing the provided files: {e}");
                        return Err(e);
                    }
                }
            } else {
                Vec::new()
            };

            let tracks = match get_borrowed_tracks(tracks) {
                Ok(tracks) => tracks,
                Err(e) => {
                    return Err(e);
                }
            };

            lib.playlists.push(Playlist { name, tracks });

            Ok(())
        } else {
            Err(MusicLibraryError::PlaylistExists)
        }
    } else {
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}

pub fn create_playlist_from_m3u8<T: Into<PathBuf>>(
    name: &str,
    m3u8_file: &T,
) -> Result<(), MusicLibraryError> {
    let lib = &*MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = lib {
        if lib.get_playlist_with_name(name).is_none() {
            todo!()
        } else {
            Err(MusicLibraryError::PlaylistExists)
        }
    } else {
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}

pub fn delete_playlist(name: &str) -> Result<(), MusicLibraryError> {
    let lib = &*MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = lib {
        if lib.get_playlist_with_name(name).is_some() {
            todo!()
        } else {
            Err(MusicLibraryError::NoSuchPlaylist)
        }
    } else {
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}
