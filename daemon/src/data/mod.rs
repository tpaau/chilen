// TODO: Instead of this module I could have `music_lib` as the root module, and `covers` and `cache`
// as submodules

pub(crate) mod cache;
pub mod music_lib;
#[cfg(test)]
mod tests;

use std::{fs::create_dir_all, path::PathBuf, sync::RwLock};

use log::{error, trace};

use crate::Error;

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
        error!("Coult not initialize the cache directory: {e}");
        return Err(Error::CacheDirError(e));
    }
    if let Err(e) = init_dir(&config.data_dir) {
        error!("Could not initialize the data directory: {e}");
        return Err(Error::DataDirError(e));
    }
    if config.music_dir.is_dir() {
        if let Err(e) = config.music_dir.metadata() {
            error!("Could not read the metadata of {:?}: {e}", config.music_dir);
            return Err(Error::MusicLibraryNotAccessible);
        }
    } else {
        error!(
            "The music library path is not a directory or does not exist: {:?}",
            config.music_dir
        );
        return Err(Error::NoMusicLibrary);
    }
    *DATA_DIR.write().unwrap() = Some(config.data_dir);
    *CACHE_DIR.write().unwrap() = Some(config.cache_dir);
    *MUSIC_DIR.write().unwrap() = Some(config.music_dir);

    trace!("Successfully set the paths from the daemon configuration");

    Ok(())
}
