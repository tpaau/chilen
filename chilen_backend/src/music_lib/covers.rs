use std::{
    fs::{File, create_dir_all},
    hash::Hash,
    io::Cursor,
    path::PathBuf,
    sync::LazyLock,
    thread,
    time::Duration,
};

pub use image::ImageFormat;
use image::{GenericImageView, ImageReader, imageops::resize};
use lofty::{
    picture::{Picture, PictureType},
    tag::Tag,
};
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::music_lib::CACHE_DIR;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pixels(pub u32);

#[cfg_attr(test, derive(Default))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quality {
    #[cfg_attr(test, default)]
    Low,
    Default,
    High,
    Lossless,
    Custom(Pixels),
}

impl Quality {
    fn resolution(self) -> Option<Pixels> {
        match self {
            Quality::Low => Some(Pixels(256)),
            Quality::Default => Some(Pixels(512)),
            Quality::High => Some(Pixels(1024)),
            Quality::Lossless => None,
            Quality::Custom(pixels) => Some(pixels),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    pub format: ImageFormat,
    pub thumbnail_resolution: Pixels,
    pub cover_quality: Quality,
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            thumbnail_resolution: Pixels(40),
            cover_quality: Quality::default(),
        }
    }
}

impl Config {
    fn extension(&self) -> String {
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

/// Track cover art caching mode used when indexing the music library.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadMode {
    /// Don't cache cover arts.
    ///
    /// This should only be used for testing.
    #[cfg(test)]
    None,
    #[default]
    /// Use cached cover art images when possible.
    Load,
    /// Discard cached cover art images and extract them when indexing.
    Rebuild,
}

impl std::fmt::Display for LoadMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(test)]
            Self::None => write!(f, "None"),
            Self::Load => write!(f, "Load"),
            Self::Rebuild => write!(f, "Rebuild"),
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

pub(crate) static HIRES_COVER_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
    cache.push("covers/hires");
    cache
});

pub(crate) static THUMBNAIL_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
    cache.push("covers/thumbnails");
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

fn safe_file_create(path: &PathBuf) -> Result<File, std::io::Error> {
    let sleep_dur = Duration::from_millis(100);
    loop {
        match File::create(path) {
            Ok(handle) => return Ok(handle),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TooManyOpenFiles {
                    thread::sleep(sleep_dur);
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

// TODO: Add quality options
pub(crate) fn get_track_cover(
    track_hash: u64,
    tag: &Tag,
    load_mode: &LoadMode,
    config: Config,
) -> Result<Cover, CoverError> {
    let hires_cache = HIRES_COVER_CACHE_DIR.clone();
    let thumbnail_cache = THUMBNAIL_CACHE_DIR.clone();

    if !hires_cache.is_dir()
        && let Err(e) = create_dir_all(&hires_cache)
    {
        error!("Could not create the cache directory {hires_cache:?}: {e}");
        return Err(CoverError::CacheDirError {
            path: hires_cache,
            error: e.to_string(),
        });
    }

    if !thumbnail_cache.is_dir()
        && let Err(e) = create_dir_all(&thumbnail_cache)
    {
        error!("Could not create the cache directory {thumbnail_cache:?}: {e}");
        return Err(CoverError::CacheDirError {
            path: thumbnail_cache,
            error: e.to_string(),
        });
    }

    let buf = pick_front_cover_or_replacement(tag.pictures())?.data();
    let image = match ImageReader::new(Cursor::new(buf)).with_guessed_format() {
        Ok(image) => match image.decode() {
            Ok(i) => i,
            Err(e) => {
                warn!("Couldn't decode image data: {e}");
                return Err(CoverError::DecodingError);
            }
        },
        Err(e) => {
            warn!("{e}");
            return Err(CoverError::UnknownFileFormat);
        }
    };

    let extension = config.extension();
    let mut hires_path = hires_cache.clone();
    hires_path.push(format!("{track_hash}.{extension}"));
    let mut thumbnail_path = thumbnail_cache.clone();
    thumbnail_path.push(format!("{track_hash}.{extension}"));

    let thumbnail = if load_mode == &LoadMode::Load && thumbnail_path.is_file() {
        Some(thumbnail_path)
    } else {
        let thumbnail = resize(
            &image,
            config.thumbnail_resolution.0,
            config.thumbnail_resolution.0,
            COVER_FILTER,
        );

        match safe_file_create(&thumbnail_path) {
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

    let hires = if load_mode == &LoadMode::Load && hires_path.is_file() {
        Some(hires_path)
    } else {
        let cover = match config.cover_quality.resolution() {
            Some(res) => resize(&image, res.0, res.0, COVER_FILTER).buffer_like(),
            None => image.into(),
        };

        match safe_file_create(&hires_path) {
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

    Ok(Cover { hires, thumbnail })
}
