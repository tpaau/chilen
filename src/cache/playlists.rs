use std::{
    collections::HashSet,
    fs::{File, read},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::PathBuf,
    sync::{LazyLock, RwLock},
    time::{Duration, SystemTime},
};

use bincode::{Decode, Encode, config::standard, encode_to_vec};
use log::{error, trace};
use mpipc::MusicLibraryError;

use crate::{
    cache::{CACHE_DIR, CacheError},
    cache::indexer::{index, index_files},
    track::Track,
};

#[derive(Clone, Debug, Encode, Decode)]
struct ConfPlaylist {
    pub name: String,
    pub track_hashes: Vec<u64>,
}

impl From<Playlist> for ConfPlaylist {
    fn from(value: Playlist) -> Self {
        let mut track_hashes = Vec::new();
        for track in value.tracks {
            let mut hasher = DefaultHasher::new();
            track.hash(&mut hasher);
            track_hashes.push(hasher.finish());
        }
        Self {
            name: value.name,
            track_hashes,
        }
    }
}

#[derive(Clone, Debug, Encode, Decode)]
struct ConfMusicLibrary {
    pub playlists: Vec<ConfPlaylist>,
}

impl From<MusicLibrary> for ConfMusicLibrary {
    fn from(value: MusicLibrary) -> Self {
        let mut playlists = Vec::new();
        for playlist in value.playlists {
            playlists.push(playlist.into());
        }
        Self { playlists }
    }
}

#[derive(Clone, Debug)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
}

impl Into<mpipc::Playlist> for Playlist {
    fn into(self) -> mpipc::Playlist {
        let mut tracks = Vec::new();
        for track in self.tracks {
            tracks.push(track.into());
        }
        mpipc::Playlist {
            name: self.name,
            tracks,
        }
    }
}

impl Playlist {
    fn from_loaded_playlist(loaded: ConfPlaylist, tracks: &[Track]) -> Self {
        let wanted: HashSet<u64> = loaded.track_hashes.into_iter().collect();

        let tracks = tracks
            .iter()
            .filter_map(|track| {
                let mut hasher = DefaultHasher::new();
                track.hash(&mut hasher);
                let h = hasher.finish();
                if wanted.contains(&h) {
                    Some(track.clone())
                } else {
                    None
                }
            })
            .collect();

        Self {
            name: loaded.name,
            tracks,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct MusicLibrary {
    pub playlists: Vec<Playlist>,
    pub tracks: Vec<Track>,
}

impl MusicLibrary {
    fn from_loaded_lib(loaded: ConfMusicLibrary, tracks: Vec<Track>) -> Self {
        let mut playlists = Vec::new();
        for playlist in loaded.playlists {
            playlists.push(Playlist::from_loaded_playlist(playlist, &tracks));
        }
        Self { playlists, tracks }
    }

    pub fn get_playlist_with_name(&'_ self, name: &str) -> Option<&'_ Playlist> {
        self.playlists
            .iter()
            .find(|&playlist| playlist.name == name)
    }
}

static LIBRARY_CACHE_FILE: LazyLock<Result<PathBuf, CacheError>> =
    LazyLock::new(|| match CACHE_DIR.clone() {
        Ok(mut cache) => {
            cache.push("playlists");
            Ok(cache)
        }
        Err(e) => Err(e),
    });

static MUSIC_LIBRARY: RwLock<Option<MusicLibrary>> = RwLock::new(None);

/// Save the library state to a file.
pub fn save_library() -> Result<(), MusicLibraryError> {
    trace!("Saving the library state to library cache");

    let lib = MUSIC_LIBRARY.read().unwrap().clone();

    if let Some(lib) = lib {
        let lib = ConfMusicLibrary::from(lib);

        let library_cache = match LIBRARY_CACHE_FILE.clone() {
            Ok(cache) => cache,
            Err(e) => {
                error!("Could not get the path to the library cache: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        let data = match encode_to_vec(lib, standard()) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not serialize the library cache: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        let mut file = match File::create(library_cache) {
            Ok(file) => file,
            Err(e) => {
                error!("Could not open the library cache in write-only mode: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        match file.write_all(&data) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Could not write to the library cache: {e}");
                Err(MusicLibraryError::CacheError)
            }
        }
    } else {
        error!("Cannot save the library since it is uninitialized!");
        Err(MusicLibraryError::CacheError)
    }
}

/// Load the music library from the playlists file.
pub fn load() -> Result<(), MusicLibraryError> {
    trace!("Loading the music library");

    if MUSIC_LIBRARY.read().unwrap().is_some() {
        error!("Cannot load the music library, it is already initialized!");
        return Err(MusicLibraryError::CacheError);
    }

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

    trace!("Loading the library cache");

    let library_cache = match LIBRARY_CACHE_FILE.clone() {
        Ok(cache) => cache,
        Err(e) => {
            error!("Could not get the path to the library cache: {e}");
            return Err(MusicLibraryError::CacheError);
        }
    };

    let exists = match library_cache.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            error!("Could not check if the library cache exists: {e}");
            return Err(MusicLibraryError::CacheError);
        }
    };

    if exists {
        if library_cache.is_dir() {
            error!("The library cache at {library_cache:?} must not be a directory!");
            return Err(MusicLibraryError::CacheError);
        }

        let data = match read(library_cache) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not read the library cache: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        let lib_conf: ConfMusicLibrary = match bincode::decode_from_slice(&data, standard()) {
            Ok(data) => data.0,
            Err(e) => {
                error!("Could not decode the contents of the library cache: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        let lib = MusicLibrary::from_loaded_lib(lib_conf, tracks);
        let mut guard = MUSIC_LIBRARY.write().unwrap();
        *guard = Some(lib);
        drop(guard);
    } else {
        let mut guard = MUSIC_LIBRARY.write().unwrap();
        *guard = Some(MusicLibrary::default());
        drop(guard);
        save_library()?;
    }

    let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));
    trace!(
        "Done loading the music library in {:.2}s",
        time_elapsed.as_secs_f64()
    );

    Ok(())
}

pub fn get_library() -> Result<MusicLibrary, MusicLibraryError> {
    if let Some(lib) = MUSIC_LIBRARY.read().unwrap().clone() {
        Ok(lib)
    } else {
        error!("Tried to get the music library, but it was uninitialized!");
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}

pub fn create_playlist(
    name: String,
    tracks: &Option<Vec<PathBuf>>,
) -> Result<(), MusicLibraryError> {
    trace!("Creating a new playlist with name \"{name}\" from a list tracks");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = guard.as_mut() {
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

            let lib_set: HashSet<_> = lib.tracks.iter().collect();
            let intersecting_tracks: Vec<Track> = tracks
                .iter()
                .filter(|t| lib_set.contains(t))
                .cloned()
                .collect();

            lib.playlists.push(Playlist {
                name: name.clone(),
                tracks: intersecting_tracks,
            });

            drop(guard);
            save_library()?;
            trace!("Created a new playlist with the name \"{name}\"");
            Ok(())
        } else {
            error!("A playlist with the name \"{name}\" already exists");
            Err(MusicLibraryError::PlaylistExists)
        }
    } else {
        error!("Cannot modify an uninitialized library");
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

pub fn delete_playlist(name: &str, save_state: bool) -> Result<(), MusicLibraryError> {
    trace!("Deleting playlist with name \"{name}\"");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = guard.as_mut() {
        match lib
            .playlists
            .iter()
            .position(|playlist| playlist.name == name)
        {
            Some(pos) => {
                lib.playlists.remove(pos);
                if save_state {
                    drop(guard);
                    save_library()?;
                }
                trace!("Deleted playlist with name \"{name}\"");
                Ok(())
            }
            None => Err(MusicLibraryError::NoSuchPlaylist),
        }
    } else {
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}
