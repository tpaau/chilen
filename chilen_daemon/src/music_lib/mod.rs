pub(crate) mod covers;
pub(crate) mod indexer;
pub(crate) mod state;
#[cfg(test)]
mod tests;

use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use log::{error, trace};

use crate::{
    Error,
    music_lib::state::{MUSIC_LIBRARY, Track, save_library, unwrap_lib_mut, unwrap_lib_ref},
};

pub(crate) static DATA_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
pub(crate) static MUSIC_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
pub(crate) static CACHE_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);

pub(crate) fn cleanup() {
    *DATA_DIR.write().unwrap() = None;
    *MUSIC_DIR.write().unwrap() = None;
    *CACHE_DIR.write().unwrap() = None;
}

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

pub(crate) fn tracks_from_paths(track_paths: &[PathBuf]) -> Result<Vec<Track>, chilen_ipc::Error> {
    let guard = MUSIC_LIBRARY.read().unwrap();
    let lib = unwrap_lib_ref(guard.as_ref())?;
    let mut out = Vec::with_capacity(track_paths.len());

    for path in track_paths {
        if let Some(track) = lib.find_track_by_path(path) {
            out.push(track.as_ref().clone());
        } else {
            return Err(chilen_ipc::Error::UnknownTrack);
        }
    }

    Ok(out)
}

pub(crate) fn tracks_from_hashes(hashes: Vec<u64>) -> Result<Vec<Arc<Track>>, chilen_ipc::Error> {
    let guard = MUSIC_LIBRARY.read().unwrap();
    let lib = unwrap_lib_ref(guard.as_ref())?;
    lib.tracks_from_hashes(hashes)
}

pub(crate) fn create_playlist(
    name: String,
    track_paths: &Option<Vec<PathBuf>>,
) -> Result<(), chilen_ipc::Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.create_playlist(name.clone(), track_paths)?;

    drop(guard);
    save_library()
}

pub(crate) fn import_playlist_from_m3u8(
    name: Option<String>,
    file: &Path,
) -> Result<(), chilen_ipc::Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.import_m3u8_playlist(file, name)?;

    drop(guard);
    save_library()
}

pub(crate) fn delete_playlists(playlists: Vec<String>) -> Result<(), chilen_ipc::Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.remove_playlists(playlists)?;

    drop(guard);
    save_library()
}

pub(crate) fn add_tracks(playlist: &str, tracks: Vec<PathBuf>) -> Result<(), chilen_ipc::Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.add_tracks(playlist, tracks)?;

    drop(guard);
    save_library()
}

/// Remove tracks by indices from a playlist
pub(crate) fn remove_tracks(playlist: &str, tracks: Vec<usize>) -> Result<(), chilen_ipc::Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.remove_tracks(playlist, tracks)?;

    drop(guard);
    save_library()?;
    Ok(())
}
