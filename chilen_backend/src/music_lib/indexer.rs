use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use lofty::{error::LoftyError, file::TaggedFile};
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    Error,
    music_lib::{
        MUSIC_DIR, Track,
        covers::{self, LoadMode},
    },
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValueSeparators {
    pub artist: Vec<String>,
    pub album: Vec<String>,
    pub genre: Vec<String>,
}

impl ValueSeparators {
    fn new(separators: Vec<String>) -> Self {
        Self {
            artist: separators.clone(),
            album: separators.clone(),
            genre: separators,
        }
    }
}

#[cfg_attr(test, derive(Default))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    pub value_separators: ValueSeparators,
    pub covers: covers::Config,
}

fn safe_read_from_path(path: &PathBuf) -> Result<TaggedFile, LoftyError> {
    let sleep_dur = Duration::from_millis(100);
    loop {
        match lofty::read_from_path(path) {
            Ok(file) => return Ok(file),
            Err(e) => {
                if let lofty::error::ErrorKind::Io(e) = e.kind()
                    && e.kind() == std::io::ErrorKind::TooManyOpenFiles
                {
                    thread::sleep(sleep_dur);
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

fn index_files(
    files: Vec<PathBuf>,
    load_mode: LoadMode,
    config: covers::Config,
) -> Result<Vec<Track>, Error> {
    let tracks = Arc::new(RwLock::new(Vec::with_capacity(files.len())));
    let mut handles = Vec::new();

    for file in files {
        let lock = tracks.clone();

        handles.push(thread::spawn(move || {
            let tagged_file = match safe_read_from_path(&file) {
                Ok(tagged_file) => tagged_file,
                Err(e) => {
                    match e.kind() {
                        lofty::error::ErrorKind::UnknownFormat => {
                            info!("Likely not an audio file: {file:?}");
                        }
                        _ => {
                            warn!("Could not get tags from file {file:?}: {e}");
                        }
                    }
                    return;
                }
            };

            let track = match Track::new(file, &tagged_file, &load_mode, &config) {
                Ok(track) => track,
                Err(e) => {
                    warn!("Could not create a track struct: {e}");
                    return;
                }
            };

            lock.write().unwrap().push(track)
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    tracks.write().unwrap().shrink_to_fit();
    Ok(Arc::into_inner(tracks).unwrap().into_inner().unwrap())
}

pub(crate) fn index(load_mode: LoadMode, config: covers::Config) -> Result<Vec<Track>, Error> {
    #[cfg(debug_assertions)]
    warn!(
        "\x1b[1mINDEXER RUNNING IN DEBUG MODE\x1b[0m, image decoding will be EXTREMELY SLOW. Consider running the program in release mode for the first indexing",
    );

    let music_dir = MUSIC_DIR.read().unwrap().clone().unwrap();

    trace!("Indexing directory: {music_dir:?}");

    let files: Vec<_> = WalkDir::new(music_dir)
        .into_iter()
        .filter_map(|result| {
            let entry = match result {
                Ok(entry) => entry,
                Err(e) => {
                    warn!("Error while trying to access the file: {e}");
                    return None;
                }
            };

            match entry.metadata() {
                Ok(meta) => {
                    if meta.is_file() {
                        Some(PathBuf::from(entry.path()))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    warn!("Could not get path metadata: {e}");
                    None
                }
            }
        })
        .collect();

    index_files(files, load_mode, config)
}
