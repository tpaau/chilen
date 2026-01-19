pub mod cache;
pub mod music_lib;

use std::{fs::create_dir_all, path::PathBuf, sync::RwLock};

use log::{error, trace};
use mpipc::DataError;

pub(crate) static DATA_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
pub(crate) static MUSIC_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
pub(crate) static CACHE_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);

fn check_or_init_dir(dir: &PathBuf) -> Result<(), DataError> {
    if dir.is_dir() {
        let perms = match dir.metadata() {
            Ok(md) => md.permissions(),
            Err(e) => {
                error!("Could not read the metadata of {dir:?}: {e}");
                return Err(DataError::PermissionError);
            }
        };
        if perms.readonly() {
            error!("The directory {dir:?} must be writable!");
            return Err(DataError::PermissionError);
        }
        Ok(())
    } else {
        let exists = match dir.try_exists() {
            Ok(exists) => exists,
            Err(e) => {
                error!("Can't check the existence of {dir:?}: {e}");
                return Err(DataError::PermissionError);
            }
        };
        if exists {
            error!("The directory path is not actually a directory: {dir:?}");
            Err(DataError::NotDirectory)
        } else {
            trace!("The directory at {dir:?} does not exist. Attempting to create a new one");
            if let Err(e) = create_dir_all(dir) {
                error!("Could not create the directory: {e}");
                return Err(DataError::PermissionError);
            }
            trace!("Created a new directory at {dir:?}");
            Ok(())
        }
    }
}

pub(crate) fn set_data_dirs(config: crate::Config) -> Result<(), DataError> {
    check_or_init_dir(&config.cache_dir)?;
    check_or_init_dir(&config.data_dir)?;
    if config.music_dir.is_dir() {
        if let Err(e) = config.music_dir.metadata() {
            error!("Could not read the metadata of {:?}: {e}", config.music_dir);
            return Err(DataError::PermissionError);
        }
    } else {
        error!("The music library path is not a directory or does not exist: {:?}", config.music_dir);
        return Err(DataError::NoMusicLibrary);
    }
    *DATA_DIR.write().unwrap() = Some(config.data_dir);
    *CACHE_DIR.write().unwrap() = Some(config.cache_dir);
    *MUSIC_DIR.write().unwrap() = Some(config.music_dir);

    trace!("Successfully set the paths from the daemon configuration");

    Ok(())
}
