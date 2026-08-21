use std::{
    collections::HashSet,
    fs::{File, create_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
};

pub use image::ImageFormat;
use image::{ImageReader, imageops::resize};
use lofty::{
    picture::{Picture, PictureType},
    tag::Tag,
};
use log::{error, trace, warn};
use serde::{Deserialize, Serialize};

use crate::music_lib::{CACHE_DIR, indexer, state::Track};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quality {
    Low,
    #[default]
    Default,
    High,
    Lossless,
    Custom(u32),
}

impl Quality {
    pub(crate) fn resolution(self) -> Option<u32> {
        match self {
            Quality::Low => Some(256),
            Quality::Default => Some(512),
            Quality::High => Some(1024),
            Quality::Lossless => None,
            Quality::Custom(pixels) => Some(pixels),
        }
    }
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

#[cfg_attr(test, derive(Default))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cover {
    pub hires: Option<PathBuf>,
    pub thumbnail: Option<PathBuf>,
}

impl Cover {
    pub(crate) fn none() -> Self {
        Self {
            hires: None,
            thumbnail: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoverError {
    NoPictures,
    NoSuitablePictures,
    CoverWriteError { path: PathBuf, error: String },
    CacheDirError { path: PathBuf, error: String },
    UnknownFileFormat,
    DecodingError,
}

impl std::fmt::Display for CoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPictures => write!(f, "Could not find any pictures in the tag"),
            Self::NoSuitablePictures => write!(
                f,
                "The tag contains pictures, but none of them can be used as cover art replacement"
            ),
            Self::CoverWriteError { path, error } => write!(
                f,
                "Could not write the cover image to cache at {path:?}: {error}"
            ),
            Self::CacheDirError { path, error } => {
                write!(
                    f,
                    "Could not create the cache directory at {path:?}: {error}"
                )
            }
            CoverError::UnknownFileFormat => write!(f, "Could not guess the image format"),
            CoverError::DecodingError => write!(f, "Could not decode the image data"),
        }
    }
}

const COVER_FILTER: image::imageops::FilterType = image::imageops::FilterType::Triangle;

const FRONT_COVER_PRIORITY: [PictureType; 21] = [
    PictureType::CoverFront,
    PictureType::CoverBack,
    PictureType::Illustration,
    PictureType::Leaflet,
    PictureType::Media,
    PictureType::BandLogo,
    PictureType::Other,
    PictureType::Band,
    PictureType::ScreenCapture,
    PictureType::DuringPerformance,
    PictureType::DuringRecording,
    PictureType::RecordingLocation,
    PictureType::BrightFish,
    PictureType::LeadArtist,
    PictureType::Artist,
    PictureType::Composer,
    PictureType::Lyricist,
    PictureType::Conductor,
    PictureType::PublisherLogo,
    PictureType::Icon,
    PictureType::OtherIcon,
];

pub(crate) static TRACK_HIRES_COVER_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
    cache.push("covers");
    cache.push("tracks");
    cache.push("hires");
    cache
});

pub(crate) static TRACK_THUMBNAIL_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
    cache.push("covers");
    cache.push("tracks");
    cache.push("thumbnails");
    cache
});

pub(crate) static PLAYLIST_HIRES_COVER_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
    cache.push("covers");
    cache.push("playlists");
    cache.push("hires");
    cache
});

pub(crate) static PLAYLIST_THUMBNAIL_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
    cache.push("covers");
    cache.push("playlists");
    cache.push("thumbnails");
    cache
});

fn pick_front_cover_or_replacement(pictures: &[Picture]) -> Result<&Picture, CoverError> {
    if pictures.is_empty() {
        return Err(CoverError::NoPictures);
    }

    for pic_type in FRONT_COVER_PRIORITY {
        for pic in pictures {
            if pic.pic_type() == pic_type {
                return Ok(pic);
            }
        }
    }

    Err(CoverError::NoSuitablePictures)
}

pub(crate) fn get_track_cover(
    tag: &Tag,
    config: indexer::Config,
    covers_lookup_set: &RwLock<HashSet<PathBuf>>,
) -> Result<Cover, CoverError> {
    let hires_cache = TRACK_HIRES_COVER_CACHE_DIR.clone();
    let thumbnail_cache = TRACK_THUMBNAIL_CACHE_DIR.clone();

    let image_buf = pick_front_cover_or_replacement(tag.pictures())?.data();
    let mut hasher = DefaultHasher::new();
    image_buf.hash(&mut hasher);
    let hash = hasher.finish();

    let extension = config.extension();
    let mut hires_path = hires_cache.clone();
    hires_path.push(format!("{hash}.{extension}"));
    let mut thumbnail_path = thumbnail_cache.clone();
    thumbnail_path.push(format!("{hash}.{extension}"));

    let image =
        LazyLock::new(
            || match ImageReader::new(Cursor::new(image_buf)).with_guessed_format() {
                Ok(image) => match image.decode() {
                    Ok(i) => Ok(i),
                    Err(e) => {
                        warn!("Couldn't decode image data: {e}");
                        Err(CoverError::DecodingError)
                    }
                },
                Err(e) => {
                    warn!("{e}");
                    Err(CoverError::UnknownFileFormat)
                }
            },
        );

    let thumbnail = if config.cache_mode == CacheMode::UseCache
        && covers_lookup_set.read().unwrap().contains(&thumbnail_path)
    {
        Some(thumbnail_path)
    } else if config.cache_mode == CacheMode::UseCache && thumbnail_path.is_file() {
        covers_lookup_set
            .write()
            .unwrap()
            .insert(thumbnail_path.clone());
        Some(thumbnail_path)
    } else {
        if !thumbnail_cache.is_dir()
            && let Err(e) = create_dir_all(&thumbnail_cache)
        {
            error!("Could not create the cache directory {thumbnail_cache:?}: {e}");
            return Err(CoverError::CacheDirError {
                path: thumbnail_cache,
                error: e.to_string(),
            });
        }

        let image = match &*image {
            Ok(image) => image,
            Err(e) => {
                return Err(e.clone());
            }
        };

        let thumbnail = resize(
            image,
            config.thumbnail_resolution,
            config.thumbnail_resolution,
            COVER_FILTER,
        );

        match File::create(&thumbnail_path) {
            Ok(mut file) => match thumbnail.write_to(&mut file, config.format) {
                Ok(_) => {
                    covers_lookup_set
                        .write()
                        .unwrap()
                        .insert(thumbnail_path.clone());
                    Some(thumbnail_path)
                }
                Err(e) => {
                    warn!("Couldn't write thumbnail to {thumbnail_path:?}: {e}");
                    None
                }
            },
            Err(e) => {
                warn!("Couldn't write thumbnail to {thumbnail_path:?}: {e}");
                None
            }
        }
    };

    let hires = if config.cache_mode == CacheMode::UseCache
        && covers_lookup_set.read().unwrap().contains(&hires_path)
    {
        Some(hires_path)
    } else if config.cache_mode == CacheMode::UseCache && hires_path.is_file() {
        covers_lookup_set
            .write()
            .unwrap()
            .insert(hires_path.clone());
        Some(hires_path)
    } else {
        if !hires_cache.is_dir()
            && let Err(e) = create_dir_all(&hires_cache)
        {
            error!("Could not create the cache directory {hires_cache:?}: {e}");
            return Err(CoverError::CacheDirError {
                path: hires_cache,
                error: e.to_string(),
            });
        }

        let image = match image.as_ref() {
            Ok(image) => image,
            Err(e) => {
                return Err(e.clone());
            }
        };

        let cover = match config.cover_quality.resolution() {
            Some(res) => {
                if image.height() > res && image.height() > res {
                    resize(image, res, res, COVER_FILTER)
                } else {
                    image.clone().into()
                }
            }
            None => image.clone().into(),
        };

        match File::create(&hires_path) {
            Ok(mut file) => match cover.write_to(&mut file, config.format) {
                Ok(_) => {
                    covers_lookup_set
                        .write()
                        .unwrap()
                        .insert(hires_path.clone());
                    Some(hires_path)
                }
                Err(e) => {
                    warn!("Couldn't write thumbnail to {hires_path:?}: {e}");
                    None
                }
            },
            Err(e) => {
                warn!("Couldn't write thumbnail to {hires_path:?}: {e}");
                None
            }
        }
    };

    Ok(Cover { hires, thumbnail })
}

fn get_4_unique_covers(tracks: &[Arc<Track>]) -> Option<[&Cover; 4]> {
    let mut unique_tracks: HashSet<&Cover> = HashSet::new();
    for track in tracks {
        unique_tracks.insert(&track.cover);
    }
    let mut tracks = unique_tracks.into_iter();
    Some([
        tracks.next()?,
        tracks.next()?,
        tracks.next()?,
        tracks.next()?,
    ])
}

pub(crate) fn get_playlist_cover(name: &str, tracks: &[Arc<Track>]) -> Result<Cover, CoverError> {
    if tracks.is_empty() {
        Ok(Cover::none()) // No covers for empty playlists
    } else if tracks.len() >= 4
        && let Some(tracks) = get_4_unique_covers(tracks)
    {
        // And then for playlists with tracks with 4 or more unique covers, a custom cover is used

        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();

        let mut hires_cover_path = PLAYLIST_HIRES_COVER_CACHE_DIR.clone();
        hires_cover_path.push(hash.to_string());

        let mut thumbnail_path = PLAYLIST_THUMBNAIL_CACHE_DIR.clone();
        thumbnail_path.push(hash.to_string());

        let mut hires_cover = if hires_cover_path.is_file() {
            Some(hires_cover_path)
        } else {
            None
        };

        let mut thumbnail = if thumbnail_path.is_file() {
            Some(thumbnail_path)
        } else {
            None
        };

        Ok(Cover {
            hires: hires_cover,
            thumbnail,
        })
    } else {
        // If there are less than 4 unique covers, use the cover of the first track
        Ok(tracks[0].cover.clone())
    }
}
