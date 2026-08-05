pub mod covers;
pub(crate) mod indexer;

pub mod state;
#[cfg(test)]
mod tests;

use std::{
    fs::{File, create_dir_all, read},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use log::{error, trace};
use m3u8_rs::{MediaPlaylist, MediaSegment};

use crate::{
    Error,
    music_lib::state::{MUSIC_LIBRARY, Track, save_library, unwrap_lib_mut, unwrap_lib_ref},
};

pub(crate) static DATA_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
pub(crate) static MUSIC_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
pub(crate) static CACHE_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);

fn init_dir(dir: &PathBuf) -> Result<(), Error> {
    if dir.is_dir() {
        let perms = match dir.metadata() {
            Ok(md) => md.permissions(),
            Err(e) => {
                error!("Could not read the metadata of {dir:?}: {e}");
                return Err(Error::PathInaccessible(dir.to_path_buf()));
            }
        };
        if perms.readonly() {
            error!("The directory {dir:?} is readonly");
            return Err(Error::PathReadonly(dir.to_path_buf()));
        }
        Ok(())
    } else {
        let exists = match dir.try_exists() {
            Ok(exists) => exists,
            Err(e) => {
                error!("Can't check whether {dir:?} exists: {e}");
                return Err(Error::PathInaccessible(dir.to_path_buf()));
            }
        };
        if exists {
            error!("The path is not a directory: {dir:?}");
            Err(Error::NotADirectory(dir.to_path_buf()))
        } else {
            trace!("The directory at {dir:?} does not exist. Attempting to create a new one");
            if let Err(e) = create_dir_all(dir) {
                error!("Could not create the directory: {e}");
                return Err(Error::DirectoryCreationFailed(
                    dir.to_path_buf(),
                    e.to_string(),
                ));
            }
            trace!("Created a new directory at {dir:?}");
            Ok(())
        }
    }
}

pub(crate) fn set_dirs(
    data_dir: PathBuf,
    cache_dir: PathBuf,
    music_dir: PathBuf,
) -> Result<(), Error> {
    init_dir(&cache_dir)?;
    init_dir(&data_dir)?;
    if music_dir.is_dir() {
        if let Err(e) = music_dir.metadata() {
            error!("Could not read the metadata of {:?}: {e}", music_dir);
            return Err(Error::PathInaccessible(music_dir));
        }
    } else {
        error!("The music library is not a directory or does not exist: {music_dir:?}");
        return Err(Error::NotADirectory(music_dir));
    }
    *DATA_DIR.write().unwrap() = Some(data_dir);
    *CACHE_DIR.write().unwrap() = Some(cache_dir);
    *MUSIC_DIR.write().unwrap() = Some(music_dir);

    trace!("Successfully set paths");

    Ok(())
}

/// Find indexed tracks in the music library by their paths.
///
/// # Fails
/// The function will fail if any of the provided paths are not present in the music library, and
/// the `allow_failure` argument is set to `false`.
///
/// If the `allow_failure` argument is `true`, the function will iterate over all the provided paths
/// and return only those that correspond to tracks in the music library. It might return an empty
/// vector if no paths could be matched.
///
/// It might also fail if the music library is not initialized, so you should never run `unwrap`
/// on the result of this function.
pub(crate) fn tracks_from_paths(
    track_paths: &[PathBuf],
    // Whether nonexistent paths should result in a failure.
    allow_failure: bool,
) -> Result<Vec<Track>, Error> {
    let guard = MUSIC_LIBRARY.read().unwrap();
    let lib = unwrap_lib_ref(guard.as_ref())?;
    let mut out = Vec::with_capacity(track_paths.len());

    for path in track_paths {
        if let Some(track) = lib.find_track_by_path(path) {
            out.push(track.as_ref().clone());
        } else if !allow_failure {
            return Err(Error::UnknownTrack);
        }
    }

    Ok(out)
}

pub(crate) fn tracks_from_hashes(hashes: Vec<u64>) -> Result<Vec<Arc<Track>>, Error> {
    let guard = MUSIC_LIBRARY.read().unwrap();
    let lib = unwrap_lib_ref(guard.as_ref())?;
    lib.tracks_from_hashes(hashes)
}

// TEST: Check if import works correctly
pub(crate) fn tracks_from_m3u8(path: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    trace!("Loading an M3U8 playlist from {path:?}");
    let data = match read(path) {
        Ok(data) => data,
        Err(e) => {
            error!("Could not read the playlist file at {path:?}: {e}");
            return Err(Error::PathDoesNotExist);
        }
    };
    let content = String::from_utf8_lossy(&data);
    match m3u8_rs::parser::parse_media_playlist(&content) {
        Ok((_, pl)) => {
            let base_path = path.parent().unwrap_or(Path::new("./"));
            let track_paths: Vec<_> = pl
                .segments
                .into_iter()
                .map(|s| {
                    let mut path = PathBuf::from(base_path);
                    path.push(s.uri.components());
                    path
                })
                .collect();
            Ok(track_paths)
        }
        Err(e) => {
            error!("Could not parse the M3U playlist: {e}");
            Err(Error::PlaylistParsingError)
        }
    }
}

// TEST: Check if export work correctly
pub fn export_playlist_to_m3u8(name: String, path: &Path) -> Result<(), Error> {
    trace!("Exporting a playlist to an M3U8 file in {path:?}");
    let guard = MUSIC_LIBRARY.read().unwrap();
    let lib = unwrap_lib_ref(guard.as_ref())?;
    let pl = match lib.find_playlist(&name) {
        Some(pl) => pl,
        None => return Err(Error::UnknownPlaylist(name)),
    };
    let music_dir = MUSIC_DIR.read().unwrap().as_ref().unwrap().clone();
    let segments = pl
        .tracks
        .iter()
        .map(|t| MediaSegment {
            uri: {
                if path.parent() == Some(&music_dir) {
                    match t.path.strip_prefix(music_dir.clone()) {
                        Ok(path) => path.to_path_buf(),
                        Err(_) => path.to_path_buf(),
                    }
                } else {
                    t.path.clone()
                }
            },
            duration: t.duration,
            title: t.title.clone(),
        })
        .collect();
    drop(guard);
    let media_playlist = MediaPlaylist { segments };
    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            error!("Could not open the m3u8 export file for writing: {e}");
            return Err(Error::PlaylistExportFailed);
        }
    };
    match file.write_all(media_playlist.serialize().as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Could not write the playlist contents to a file: {e}");
            Err(Error::PlaylistExportFailed)
        }
    }
}

pub fn create_playlist(name: String, track_paths: &Option<Vec<PathBuf>>) -> Result<(), Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    let name = name.trim();
    let name = if name.is_empty() {
        lib.get_default_playlist_name()
    } else {
        name.to_string()
    };
    lib.create_playlist(name, track_paths)?;

    drop(guard);
    save_library()
}

pub fn rename_playlist(source: &str, target: &str) -> Result<(), Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.rename_playlist(source, target)?;
    drop(guard);
    save_library()
}

pub fn import_playlist_from_m3u8(name: Option<String>, file: &PathBuf) -> Result<(), Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.import_m3u8_playlist(file, name)?;

    drop(guard);
    save_library()
}

pub fn delete_playlists(playlists: Vec<String>) -> Result<(), Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.remove_playlists(playlists)?;

    drop(guard);
    save_library()
}

pub fn add_tracks(playlist: &str, tracks: Vec<PathBuf>) -> Result<(), Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.add_tracks(playlist, tracks)?;

    drop(guard);
    save_library()
}

/// Remove tracks by indices from a playlist.
pub fn remove_tracks(playlist: &str, tracks: Vec<usize>) -> Result<(), Error> {
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    let lib = unwrap_lib_mut(guard.as_mut())?;
    lib.remove_tracks(playlist, tracks)?;

    drop(guard);
    save_library()?;
    Ok(())
}
