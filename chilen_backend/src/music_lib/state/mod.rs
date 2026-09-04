mod library;
mod playlist;
mod track;

use std::{
    fs::{File, read},
    io::Write,
    path::PathBuf,
    sync::{LazyLock, RwLock},
    time::{Duration, SystemTime},
};

use log::{error, trace};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use crate::{
    Error, Event,
    music_lib::{
        self, DATA_DIR,
        indexer::{self},
    },
};

pub use library::*;
pub use playlist::*;
pub use track::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Progress {
    /// First step of the loading process.
    ///
    /// The indexer finds files in the music library to index.
    ///
    /// This step usually takes under 100ms.
    FindingTracks,

    /// Second step of the loading process.
    ///
    /// The indexer goes through all of the found files and attempts to gather audio metadata from
    /// them. During this process track covers are also cached.
    ///
    /// This is the longest step of the loading process. The time it takes to complete this step
    /// heavily depends on on whether track covers are already cached (warm start) or not (cold
    /// start), and how many tracks are in the library.
    ///
    /// Eg. for my 908-track music library, cold start takes about 4.24s, and warm start just 0.26s.
    Indexing { progress: f32 },

    /// Third step of the loading process.
    ///
    /// After all the tracks have been gathered, the backend rebuilds the virtual library from the
    /// track tags. It rebuilds artists, albums and genres, as well as hash maps used to quickly
    /// find items in the music library.
    ///
    /// Usually takes under a 100ms, depending on library size.
    RebuildingLibrary,

    /// Fourth step of the loading process.
    ///
    /// The playlists are restored from disk.
    ///
    /// **NOTE**: This step is skipped if the `playlists` file is not present.
    RestoringState,

    /// The loading process has finished.
    Done,

    /// The loading process has failed irrecoverably and will not restart.
    Failed(Error),
}

static LIBRARY_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data = DATA_DIR.read().unwrap().clone().unwrap();
    data.push("playlists");
    data
});

pub(crate) static MUSIC_LIBRARY: RwLock<Option<MusicLibrary>> = RwLock::new(None);

pub(crate) fn unwrap_lib_ref(maybe_lib: Option<&MusicLibrary>) -> Result<&MusicLibrary, Error> {
    match maybe_lib {
        Some(lib) => Ok(lib),
        None => Err(Error::LibraryNotInitialized),
    }
}

pub(crate) fn unwrap_lib_mut(
    maybe_lib: Option<&mut MusicLibrary>,
) -> Result<&mut MusicLibrary, Error> {
    match maybe_lib {
        Some(lib) => Ok(lib),
        None => Err(Error::LibraryNotInitialized),
    }
}

#[cfg(test)]
pub(crate) fn get_library() -> Result<MusicLibrary, Error> {
    let guard = MUSIC_LIBRARY.read().unwrap();
    unwrap_lib_ref(guard.as_ref()).cloned()
}

/// Save the library state to a file.
pub(crate) fn save_library() -> Result<(), Error> {
    trace!("Saving the library state");

    let lib = MUSIC_LIBRARY.read().unwrap().clone();

    if let Some(lib_data) = lib {
        let lib = ConfMusicLibrary::from(lib_data.clone());

        let library_state = LIBRARY_FILE.clone();

        let mut data = Vec::new();
        lib.serialize(&mut Serializer::new(&mut data)).unwrap();

        let mut file = match File::create(library_state) {
            Ok(file) => file,
            Err(e) => {
                error!("Could not open the library state in write-only mode: {e}");
                return Err(Error::StateWriteFailed);
            }
        };

        match file.write_all(&data) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Could not write to the library: {e}");
                Err(Error::StateWriteFailed)
            }
        }
    } else {
        error!("Cannot save the library since it is uninitialized!");
        Err(Error::LibraryNotInitialized)
    }
}

// FIX: This function hangs if if the `MusicLibrary` struct changes (and can't be deserialized)
/// Load the music library from the playlists file.
pub(crate) fn load(config: music_lib::Config) -> Result<(), Error> {
    trace!("Loading the music library");

    let time_start = SystemTime::now();

    let tracks = match indexer::index(config.clone()) {
        Ok(tracks) => tracks,
        Err(e) => {
            crate::send_event(Event::LibraryLoadFailed(e.to_string()));
            return Err(e);
        }
    };

    let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));
    trace!(
        "Finished indexing the music directory in {:.2}s, found {} audio files",
        time_elapsed.as_secs_f64(),
        tracks.len()
    );

    let library_state = LIBRARY_FILE.clone();

    trace!("Loading the library state from {library_state:?}");

    let exists = match library_state.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            error!("Could not check if the library exists: {e}");
            crate::send_event(Event::LibraryLoadFailed(e.to_string()));
            return Err(Error::StateNotReadable);
        }
    };

    if exists {
        trace!("Restoring the library state");

        if !library_state.is_file() {
            error!("The item at {library_state:?} must be a file!");
            crate::send_event(Event::LibraryLoadFailed(format!(
                "The item at {library_state:?} must be a file!"
            )));
            return Err(Error::StateNotAFile);
        }

        let data = match read(library_state) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not read the library: {e}");
                crate::send_event(Event::LibraryLoadFailed(e.to_string()));
                return Err(Error::StateNotReadable);
            }
        };

        let lib = match ConfMusicLibrary::deserialize(&mut Deserializer::from_read_ref(&data)) {
            Ok(lib) => {
                MusicLibrary::load(lib, tracks.clone().into_iter().collect(), config.indexer)
            }
            Err(e) => {
                error!("Could not decode the contents of the library state file: {e}");
                trace!("Creating a new library");
                MusicLibrary::new(tracks.into_iter().collect())
            }
        };

        *MUSIC_LIBRARY.write().unwrap() = Some(lib);
    } else {
        trace!("The library file does not exist, creating a new library");
        *MUSIC_LIBRARY.write().unwrap() = Some(MusicLibrary::new(tracks));
    }
    crate::send_event(Event::LibraryChanged(Box::new(
        MUSIC_LIBRARY.read().unwrap().as_ref().unwrap().clone(),
    )));
    save_library()?;

    let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));
    trace!(
        "Done loading the music library in {:.2}s",
        time_elapsed.as_secs_f64()
    );

    crate::send_event(Event::LoadProgressChanged(Progress::Done));

    Ok(())
}
