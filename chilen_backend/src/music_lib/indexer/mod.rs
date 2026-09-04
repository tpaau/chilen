pub mod covers;

use std::{
    collections::HashSet,
    num::NonZero,
    path::PathBuf,
    sync::{
        Mutex, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
};

use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    Error,
    music_lib::{self, MUSIC_DIR, Track},
};

/// Defines the intensity level for resource allocation during indexing.
///
/// The indexer will always allocate at least one CPU core.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndexingIntensity {
    /// Use all available CPU cores.
    #[default]
    Fast,
    /// Use half the available CPU cores.
    Balanced,
    /// Use a quarter of the available CPU cores.
    Lightweight,
}

/// Track cover art caching mode used when indexing the music library.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheMode {
    /// Don't attempt to obtain cover art images for indexed tracks.
    ///
    /// This should only be used for testing.
    ///
    #[cfg_attr(test, default)]
    #[cfg(test)]
    Disabled,
    #[cfg_attr(not(test), default)]
    /// Use cached cover art images when possible.
    UseCache,
    /// Discard cached cover art images and extract them when indexing.
    RebuildCache,
}

impl IndexingIntensity {
    /// Returns the percentage of available CPU cores the indexer should use.
    fn multiplier(&self) -> f32 {
        match self {
            IndexingIntensity::Fast => 1.0,
            IndexingIntensity::Balanced => 0.5,
            IndexingIntensity::Lightweight => 0.25,
        }
    }
}

#[cfg_attr(test, derive(Default))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    pub cache_mode: CacheMode,
    pub indexing_intensity: IndexingIntensity,
    pub covers: covers::Config,
}

fn index_files(files: Vec<PathBuf>, config: music_lib::Config) -> Vec<Track> {
    let total = files.len();
    let mut tracks = Mutex::new(Vec::with_capacity(files.len()));
    let next_index = AtomicUsize::new(0);
    let completed = AtomicUsize::new(0);
    let last_sent = AtomicUsize::new(0);

    let report_every = (total / 100).max(1);
    let covers_lookup_set: RwLock<HashSet<PathBuf>> = RwLock::new(HashSet::new());

    crate::send_event(crate::Event::LoadProgressChanged(
        music_lib::state::Progress::Indexing { progress: 0.0 },
    ));

    std::thread::scope(|s| {
        let mult = config.indexer.indexing_intensity.multiplier();
        let available_threads = std::thread::available_parallelism().map_or(1, |v| {
            (NonZero::get(v) as f32 * mult)
                .max(1.0)
                .min(files.len() as f32) as u32
        });
        trace!("Starting {available_threads} indexing threads");

        for _ in 0..available_threads {
            s.spawn(|| {
                while let Some(file) = files.get(next_index.fetch_add(1, Ordering::Relaxed)) {
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

                    let track =
                        match Track::new(file.clone(), &tagged_file, &config, &covers_lookup_set) {
                            Ok(track) => track,
                            Err(e) => {
                                warn!("Could not create a track struct: {e}");
                                continue;
                            }
                        };

                    let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    let should_send = last_sent
                        .try_update(Ordering::Relaxed, Ordering::Relaxed, |last| {
                            if done >= last + report_every {
                                Some(done)
                            } else {
                                None
                            }
                        })
                        .is_ok();

                    if should_send {
                        let progress = done as f32 / total as f32;

                        crate::send_event(crate::Event::LoadProgressChanged(
                            music_lib::state::Progress::Indexing { progress },
                        ));
                    }

                    tracks.lock().unwrap().push(track);
                }
            });
        }
    });

    tracks.get_mut().unwrap().shrink_to_fit();
    tracks.into_inner().unwrap()
}

pub(crate) fn index(config: music_lib::Config) -> Result<Vec<Track>, Error> {
    #[cfg(debug_assertions)]
    {
        warn!(
            "\x1b[1mINDEXER RUNNING IN DEBUG MODE\x1b[0m, image decoding will be EXTREMELY SLOW. Consider running Chilen in release mode for the initial indexing",
        );
    }

    let music_dir = MUSIC_DIR.read().unwrap().clone().unwrap();

    match music_dir.is_dir() {
        true => {
            trace!("Indexing directory: {music_dir:?}");
        }
        false => {
            error!("Music library path is not a directory!");
            return Err(Error::NotADirectory(music_dir));
        }
    }

    crate::send_event(crate::Event::LoadProgressChanged(
        music_lib::state::Progress::FindingTracks,
    ));

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

    Ok(index_files(files, config))
}
