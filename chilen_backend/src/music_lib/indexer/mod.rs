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

use image::ImageFormat;
use log::{info, trace, warn};
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    pub format: ImageFormat,
    pub thumbnail_resolution: u32,
    pub cover_quality: covers::Quality,
    pub cache_mode: covers::CacheMode,
    pub indexing_intensity: IndexingIntensity,
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            thumbnail_resolution: 40,
            cover_quality: covers::Quality::default(),
            indexing_intensity: IndexingIntensity::default(),
            cache_mode: covers::CacheMode::default(),
        }
    }
}

impl Config {
    pub(crate) fn extension(&self) -> String {
        match self.format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::WebP => "webp",
            ImageFormat::Pnm => "pnm",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Tga => "tga",
            ImageFormat::Dds => "dds",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Ico => "ico",
            ImageFormat::Hdr => "hdr",
            ImageFormat::OpenExr => "exr",
            ImageFormat::Farbfeld => "ff",
            ImageFormat::Avif => "avif",
            ImageFormat::Qoi => "qoi",
            _ => "img",
        }
        .to_string()
    }
}

fn index_files(files: Vec<PathBuf>, config: music_lib::Config) -> Result<Vec<Track>, Error> {
    let mut tracks = Mutex::new(Vec::with_capacity(files.len()));
    let i = AtomicUsize::new(0);
    let covers_lookup_set: RwLock<HashSet<PathBuf>> = RwLock::new(HashSet::new());

    std::thread::scope(|s| {
        let mult = config.indexer.indexing_intensity.multiplier();
        let available_threads = std::thread::available_parallelism()
            .map_or(1, |v| (NonZero::get(v) as f32 * mult).max(1.0) as u32);
        trace!("Starting {available_threads} indexing threads");

        for _ in 0..available_threads {
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

                    let track =
                        match Track::new(file.clone(), &tagged_file, &config, &covers_lookup_set) {
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

pub(crate) fn index(config: music_lib::Config) -> Result<Vec<Track>, Error> {
    #[cfg(debug_assertions)]
    {
        warn!(
            "\x1b[1mINDEXER RUNNING IN DEBUG MODE\x1b[0m, image decoding will be EXTREMELY SLOW. Consider running Chilen in release mode for the initial indexing",
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

    index_files(files, config)
}
