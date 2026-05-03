pub(crate) mod cache;
pub(crate) mod state;
#[cfg(test)]
mod tests;

use std::{
    collections::HashSet,
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::RwLock,
};

use log::{error, trace};
use mpipc::library::LibraryError;

use crate::{
    Error,
    music_lib::state::{
        MUSIC_LIBRARY, Playlist, Track, save_library, unwrap_lib_mut, unwrap_lib_ref,
    },
};

pub(crate) static DATA_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
pub(crate) static MUSIC_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
pub(crate) static CACHE_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);

fn init_dir(dir: &PathBuf) -> Result<(), String> {
    if dir.is_dir() {
        let perms = match dir.metadata() {
            Ok(md) => md.permissions(),
            Err(e) => {
                error!("Could not read the metadata of {dir:?}: {e}");
                return Err(format!("Could not read the metadata of {dir:?}: {e}"));
            }
        };
        if perms.readonly() {
            error!("The directory {dir:?} is readonly");
            return Err(format!("The directory {dir:?} is readonly"));
        }
        Ok(())
    } else {
        let exists = match dir.try_exists() {
            Ok(exists) => exists,
            Err(e) => {
                error!("Can't check whether {dir:?} exists: {e}");
                return Err(format!("Can't check whether {dir:?} exists: {e}"));
            }
        };
        if exists {
            error!("The path is not a directory: {dir:?}");
            Err(format!("The path is not a directory: {dir:?}"))
        } else {
            trace!("The directory at {dir:?} does not exist. Attempting to create a new one");
            if let Err(e) = create_dir_all(dir) {
                error!("Could not create the directory: {e}");
                return Err(format!("Could not create the directory: {e}"));
            }
            trace!("Created a new directory at {dir:?}");
            Ok(())
        }
    }
}

pub(crate) fn set_dirs(config: crate::Config) -> Result<(), Error> {
    if let Err(e) = init_dir(&config.cache_dir) {
        error!("Could not initialize the cache directory: {e}");
        return Err(Error::CacheDirError(e));
    }
    if let Err(e) = init_dir(&config.data_dir) {
        error!("Could not initialize the data directory: {e}");
        return Err(Error::DataDirError(e));
    }
    if config.music_dir.is_dir() {
        if let Err(e) = config.music_dir.metadata() {
            error!("Could not read the metadata of {:?}: {e}", config.music_dir);
            return Err(Error::LibraryNotAccessible);
        }
    } else {
        error!(
            "The music library path is not a directory or does not exist: {:?}",
            config.music_dir
        );
        return Err(Error::NoLibrary);
    }
    *DATA_DIR.write().unwrap() = Some(config.data_dir);
    *CACHE_DIR.write().unwrap() = Some(config.cache_dir);
    *MUSIC_DIR.write().unwrap() = Some(config.music_dir);

    trace!("Successfully set the paths from the daemon configuration");

    Ok(())
}

// TODO: Optimize this
pub(crate) fn tracks_from_paths(track_paths: &[PathBuf]) -> Result<Vec<Track>, LibraryError> {
    let guard = MUSIC_LIBRARY.read().unwrap();
    let lib = unwrap_lib_ref(guard.as_ref())?;
    let mut out = Vec::with_capacity(track_paths.len());

    for track in track_paths {
        if let Some(track) = lib.tracks.iter().find(|t| t.path == *track) {
            out.push(track.clone());
        } else {
            return Err(LibraryError::NoSuchTrack);
        }
    }

    Ok(out)
}

pub(crate) fn tracks_from_hashes(
    track_hashes: Vec<u64>,
    tracks: &HashSet<Track>,
) -> Result<Vec<Track>, LibraryError> {
    let wanted: HashSet<u64> = track_hashes.into_iter().collect();

    let tracks: Vec<Track> = tracks
        .iter()
        .filter(|t| {
            let h = t.hash_self();
            wanted.contains(&h)
        })
        .cloned()
        .collect();

    if tracks.len() != wanted.len() {
        return Err(LibraryError::NoSuchTrack);
    }

    Ok(tracks)
}

pub(crate) fn create_playlist(
    name: String,
    track_paths: &Option<Vec<PathBuf>>,
) -> Result<(), LibraryError> {
    trace!("Creating a new playlist with name \"{name}\" from a list of tracks");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    if lib.find_playlist(&name).is_some() {
        error!("A playlist with name \"{name}\" already exists");
        return Err(LibraryError::PlaylistExists);
    }

    // TODO: Optimize this
    let found_tracks = if let Some(tracks) = track_paths {
        tracks
            .iter()
            .map(|track_path| {
                lib.tracks
                    .iter()
                    .find(|t| t.path == *track_path)
                    .cloned()
                    .ok_or(LibraryError::NoSuchTrack)
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    lib.playlists.push(Playlist {
        name: name.clone(),
        tracks: found_tracks,
    });
    lib.playlists.sort_by_key(|p| p.name.clone());

    drop(guard);
    save_library()?;
    trace!("Created a new playlist with name \"{name}\"");
    Ok(())
}

pub(crate) fn import_playlist_from_m3u8(
    name: Option<String>,
    m3u8_file: &Path,
) -> Result<(), LibraryError> {
    let name = {
        if let Some(name) = name {
            name
        } else {
            match m3u8_file.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => {
                    error!(
                        "Could not determine the name for the imported playlist, the M3U8 file path had no final component!"
                    );
                    return Err(LibraryError::CacheError);
                }
            }
        }
    };
    let guard = MUSIC_LIBRARY.read().unwrap();
    let lib = unwrap_lib_ref(guard.as_ref())?;
    if lib.find_playlist(&name).is_none() {
        todo!("Importing playlists from M3U8 is not yet supported")
        // lib.playlists.sort_by_key(|p| p.name.clone());
    } else {
        Err(LibraryError::PlaylistExists)
    }
}

pub(crate) fn delete_playlists(names: Vec<String>) -> Result<(), LibraryError> {
    trace!("Deleting playlists: {names:?}");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    match lib.remove_playlists(names) {
        Ok(_) => {
            save_library()?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn add_tracks(name: &str, track_paths: Vec<PathBuf>) -> Result<(), LibraryError> {
    trace!("Adding tracks to playlist \"{name}\"");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    let playlist = match lib
        .playlists
        .iter()
        .position(|playlist| playlist.name == name)
    {
        Some(pos) => &mut lib.playlists[pos],
        None => return Err(LibraryError::NoSuchPlaylist),
    };

    // TODO: Optimize this
    let mut out = Vec::with_capacity(track_paths.len());
    for track_path in track_paths {
        if let Some(track) = lib.tracks.iter().find(|t| t.path == track_path) {
            out.push(track.clone());
        } else {
            return Err(LibraryError::NoSuchTrack);
        }
    }

    playlist.tracks.append(&mut out);
    drop(guard);
    save_library()?;
    Ok(())
}

/// Remove tracks by indices from a specific playlist
pub(crate) fn remove_tracks(name: &str, ids: Vec<usize>) -> Result<(), LibraryError> {
    trace!("Removing tracks from playlist \"{name}\"");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    let playlist = match lib
        .playlists
        .iter()
        .position(|playlist| playlist.name == name)
    {
        Some(pos) => &mut lib.playlists[pos],
        None => return Err(LibraryError::NoSuchPlaylist),
    };
    playlist.remove_tracks(ids)?;
    drop(guard);
    save_library()?;
    Ok(())
}
