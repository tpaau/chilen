use std::{
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use lofty::{file::TaggedFileExt, read_from_path};
use log::{error, info, trace, warn};
use mpipc::library::LibraryError;
use walkdir::WalkDir;

use crate::music_lib::{MUSIC_DIR, Track, cache::covers::LoadMode};

pub(crate) fn index_files<T: Into<PathBuf> + Debug>(
    files: Vec<T>,
    load_mode: LoadMode,
) -> Result<Vec<Track>, LibraryError> {
    let tracks = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for file in files {
        let file = file.into();
        if !file.is_file() {
            error!("Not a file: {file:?}");
        }

        let lock = Arc::clone(&tracks);
        handles.push(thread::spawn(move || {
            let tagged_file = match read_from_path(&file) {
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

            let tag = match tagged_file.primary_tag() {
                Some(tag) => tag,
                None => {
                    warn!("Found an audio file with no tags: {file:?}. Ignoring");
                    return;
                }
            };

            let mut track = match Track::try_from(&tagged_file) {
                Ok(track) => track,
                Err(e) => {
                    warn!("Could not create a track struct: {e}");
                    return;
                }
            };
            if load_mode == LoadMode::Rebuild {
                if let Err(e) = track.extract_cover(tag) {
                    trace!("Could not extract a cover from file {file:?}: {e}")
                }
            } else if let Err(e) = track.get_cover(tag) {
                trace!("Could not obtain a cover from file {file:?}: {e}")
            }
            track.path = file;
            lock.lock().unwrap().push(track)
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(Arc::into_inner(tracks).unwrap().into_inner().unwrap())
}

// TODO: Crate a custom enum to replace the `Option` enum
pub(crate) fn index(
    music_dir: Option<PathBuf>,
    load_mode: LoadMode,
) -> Result<Vec<Track>, LibraryError> {
    let music_dir = music_dir.unwrap_or(MUSIC_DIR.read().unwrap().clone().unwrap());

    trace!("Indexing directory: {music_dir:?}");

    let mut files = Vec::new();

    for result in WalkDir::new(music_dir).into_iter() {
        let entry = match result {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Error while trying to access the file: {e}");
                continue;
            }
        };

        match entry.metadata() {
            Ok(meta) => {
                if meta.is_file() {
                    files.push(PathBuf::from(entry.path()));
                }
            }
            Err(e) => {
                warn!("Could not get `DirEntry` metadata: {e}");
                continue;
            }
        };
    }

    index_files(files, load_mode)
}
