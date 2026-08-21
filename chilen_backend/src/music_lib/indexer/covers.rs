use std::{
    collections::HashSet,
    fs::{File, create_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    io::{Cursor, Read},
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
};

pub use image::ImageFormat;
use image::{ImageBuffer, ImageReader, Rgb, imageops::resize};
use lofty::{
    picture::{Picture, PictureType},
    tag::Tag,
};
use log::{error, warn};
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
    Playlist,
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
            Self::Playlist => write!(f, "Could not create a playlist cover image"),
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

pub(crate) static TRACK_HIRES_COVER_CACHE_DIR: LazyLock<Result<PathBuf, CoverError>> =
    LazyLock::new(|| {
        let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
        cache.push("covers");
        cache.push("tracks");
        cache.push("hires");
        if !cache.is_dir()
            && let Err(e) = create_dir_all(&cache)
        {
            error!("Could not create directory at {cache:?}!");
            Err(CoverError::CoverWriteError {
                path: cache,
                error: e.to_string(),
            })
        } else {
            Ok(cache)
        }
    });

pub(crate) static TRACK_THUMBNAIL_CACHE_DIR: LazyLock<Result<PathBuf, CoverError>> =
    LazyLock::new(|| {
        let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
        cache.push("covers");
        cache.push("tracks");
        cache.push("thumbnails");
        if !cache.is_dir()
            && let Err(e) = create_dir_all(&cache)
        {
            error!("Could not create directory at {cache:?}!");
            Err(CoverError::CoverWriteError {
                path: cache,
                error: e.to_string(),
            })
        } else {
            Ok(cache)
        }
    });

pub(crate) static PLAYLIST_HIRES_COVER_CACHE_DIR: LazyLock<Result<PathBuf, CoverError>> =
    LazyLock::new(|| {
        let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
        cache.push("covers");
        cache.push("playlists");
        cache.push("hires");
        if !cache.is_dir()
            && let Err(e) = create_dir_all(&cache)
        {
            error!("Could not create directory at {cache:?}!");
            Err(CoverError::CoverWriteError {
                path: cache,
                error: e.to_string(),
            })
        } else {
            Ok(cache)
        }
    });

pub(crate) static PLAYLIST_THUMBNAIL_CACHE_DIR: LazyLock<Result<PathBuf, CoverError>> =
    LazyLock::new(|| {
        let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
        cache.push("covers");
        cache.push("playlists");
        cache.push("thumbnails");
        if !cache.is_dir()
            && let Err(e) = create_dir_all(&cache)
        {
            error!("Could not create directory at {cache:?}!");
            Err(CoverError::CoverWriteError {
                path: cache,
                error: e.to_string(),
            })
        } else {
            Ok(cache)
        }
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
    let hires_cache = TRACK_HIRES_COVER_CACHE_DIR.clone()?;
    let thumbnail_cache = TRACK_THUMBNAIL_CACHE_DIR.clone()?;

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
        let image = match image.as_ref() {
            Ok(image) => image,
            Err(e) => {
                return Err(e.clone());
            }
        };

        let cover = match config.cover_quality.resolution() {
            Some(res) => {
                if image.height() > res || image.height() > res {
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

    Ok(Cover { hires, thumbnail })
}

fn get_unique_covers(tracks: &[Arc<Track>]) -> Vec<&Cover> {
    let mut covers: HashSet<&Cover> = HashSet::new();
    for track in tracks {
        covers.insert(&track.cover);
    }
    covers.into_iter().collect()
}

fn tile_2x2(images: [&ImageBuffer<Rgb<u8>, Vec<u8>>; 4]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let w = images.iter().map(|img| img.width()).max().unwrap();
    let h = images.iter().map(|img| img.height()).max().unwrap();

    let mut canvas = ImageBuffer::new(w * 2, h * 2);

    for (i, img) in images.iter().enumerate() {
        let x0 = if i % 2 == 0 { 0 } else { w };
        let y0 = if i / 2 == 0 { 0 } else { h };

        for y in 0..img.height() {
            for x in 0..img.width() {
                canvas.put_pixel(x0 + x, y0 + y, *img.get_pixel(x, y));
            }
        }
    }

    canvas
}

pub(crate) fn get_playlist_cover(
    name: &str,
    config: indexer::Config,
    tracks: &[Arc<Track>],
) -> Result<Cover, CoverError> {
    if tracks.is_empty() {
        Ok(Cover::none()) // No covers for empty playlists
    } else if tracks.len() >= 4 {
        let unique_covers = get_unique_covers(tracks);
        if unique_covers.len() >= 4 {
            // And then for playlists with tracks with 4 or more unique covers, a custom cover is used

            let mut hasher = DefaultHasher::new();
            name.hash(&mut hasher);
            let hash = hasher.finish();

            let extension = config.extension();
            let mut hires_path = PLAYLIST_HIRES_COVER_CACHE_DIR.clone()?;
            hires_path.push(format!("{hash}.{extension}"));
            let mut thumbnail_path = PLAYLIST_THUMBNAIL_CACHE_DIR.clone()?;
            thumbnail_path.push(format!("{hash}.{extension}"));

            let image = LazyLock::new(|| {
                let mut image_buffers = Vec::new();
                for cover in &unique_covers {
                    let image_buf = match cover.hires.as_ref() {
                        Some(path) => match File::open(path) {
                            Ok(mut handle) => {
                                let mut buf = Vec::new();
                                if let Err(e) = handle.read_to_end(&mut buf) {
                                    warn!("Couldn't read from file {path:?}: {e}")
                                }
                                buf
                            }
                            Err(e) => {
                                warn!("Couldn't open cover {path:?}: {e}");
                                continue;
                            }
                        },
                        None => continue,
                    };

                    match ImageReader::new(Cursor::new(image_buf)).with_guessed_format() {
                        Ok(image) => match image.decode() {
                            Ok(i) => image_buffers.push(i),
                            Err(e) => {
                                warn!("Couldn't decode image data: {e}");
                                continue;
                            }
                        },
                        Err(e) => {
                            warn!("{e}");
                            continue;
                        }
                    }

                    if image_buffers.len() == 4 {
                        break;
                    }
                }

                let buffers = if let (Some(a), Some(b), Some(c), Some(d)) = (
                    image_buffers.first(),
                    image_buffers.get(1),
                    image_buffers.get(2),
                    image_buffers.get(3),
                ) {
                    Some([&a.to_rgb8(), &b.to_rgb8(), &c.to_rgb8(), &d.to_rgb8()])
                } else {
                    None
                };

                buffers.map(tile_2x2)
            });

            let hires = if config.cache_mode == CacheMode::UseCache && hires_path.is_file() {
                Some(hires_path)
            } else {
                let image = match image.as_ref() {
                    Some(image) => image,
                    None => {
                        return Err(CoverError::Playlist);
                    }
                };

                let cover = match config.cover_quality.resolution() {
                    Some(res) => {
                        if image.height() > res && image.height() > res {
                            resize(image, res, res, COVER_FILTER)
                        } else {
                            image.clone()
                        }
                    }
                    None => image.clone(),
                };

                match File::create(&hires_path) {
                    Ok(mut file) => match cover.write_to(&mut file, config.format) {
                        Ok(_) => Some(hires_path),
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

            let thumbnail = if config.cache_mode == CacheMode::UseCache && thumbnail_path.is_file()
            {
                Some(thumbnail_path)
            } else {
                let image = match image.as_ref() {
                    Some(image) => image,
                    None => {
                        return Err(CoverError::Playlist);
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
                        Ok(_) => Some(thumbnail_path),
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

            Ok(Cover { hires, thumbnail })
        } else {
            Ok(tracks[0].cover.clone())
        }
    } else {
        Ok(tracks[0].cover.clone())
    }
}
