use std::{
    collections::HashSet,
    num::NonZero,
    path::PathBuf,
    sync::{
        Mutex, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
};

use log::{info, trace, warn};
use walkdir::WalkDir;

use crate::{
    Error,
    music_lib::{self, MUSIC_DIR, Track, covers::LoadMode},
};

fn index_files(
    files: Vec<PathBuf>,
    load_mode: LoadMode,
    config: music_lib::Config,
) -> Result<Vec<Track>, Error> {
    let mut tracks = Mutex::new(Vec::with_capacity(files.len()));
    let i = AtomicUsize::new(0);
    let covers_lookup_set: RwLock<HashSet<PathBuf>> = RwLock::new(HashSet::new());

    std::thread::scope(|s| {
        for _ in 0..std::thread::available_parallelism().map_or(1, NonZero::get) {
            s.spawn(|| {
                while let Some(file) = files.get(i.fetch_add(1, Ordering::Relaxed)) {
                    let tagged_file = match lofty::read_from_path(file) {
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
                            continue;
                        }
                    };

                    let track = match Track::new(
                        file.clone(),
                        &tagged_file,
                        &load_mode,
                        &config,
                        &covers_lookup_set,
                    ) {
                        Ok(track) => track,
                        Err(e) => {
                            warn!("Could not create a track struct: {e}");
                            continue;
                        }
                    };

                    tracks.lock().unwrap().push(track);
                }
            });
        }
    });

    tracks.get_mut().unwrap().shrink_to_fit();
    Ok(tracks.into_inner().unwrap())
}

pub(crate) fn index(load_mode: LoadMode, config: music_lib::Config) -> Result<Vec<Track>, Error> {
    #[cfg(debug_assertions)]
    {
        let guard = crate::CONFIG.read().unwrap();
        let identity = &guard.as_ref().unwrap().identity;
        warn!(
            "\x1b[1mINDEXER RUNNING IN DEBUG MODE\x1b[0m, image decoding will be EXTREMELY SLOW. Consider running {identity} in release mode for the initial indexing",
        );
    }

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
