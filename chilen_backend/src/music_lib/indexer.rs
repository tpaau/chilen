use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use lofty::{file::TaggedFileExt, read_from_path};
use log::{info, trace, warn};
use walkdir::WalkDir;

use crate::{
    Error,
    music_lib::{MUSIC_DIR, Track, covers::LoadMode},
};

// FIX: Too many open files (os error 24) on large music libraries
fn index_files(files: Vec<PathBuf>, load_mode: LoadMode) -> Result<Vec<Track>, Error> {
    let tracks = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for file in files {
        let lock = tracks.clone();

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

            match load_mode {
                #[cfg(test)]
                LoadMode::None => {}
                LoadMode::Load => {
                    if let Err(e) = track.get_cover(tag) {
                        trace!("Could not obtain a cover from file {file:?}: {e}")
                    }
                }
                LoadMode::Rebuild => {
                    if let Err(e) = track.extract_cover(tag) {
                        trace!("Could not extract a cover from file {file:?}: {e}")
                    }
                }
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

pub(crate) fn index(load_mode: LoadMode) -> Result<Vec<Track>, Error> {
    let music_dir = MUSIC_DIR.read().unwrap().clone().unwrap();

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
                warn!("Could not get path metadata: {e}");
                continue;
            }
        };
    }

    index_files(files, load_mode)
}
