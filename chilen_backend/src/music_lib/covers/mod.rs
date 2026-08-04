mod infer_image;

use std::{
    fs::{File, create_dir_all},
    hash::Hash,
    io::Write,
    path::PathBuf,
    sync::LazyLock,
    thread,
    time::Duration,
};

use lofty::{
    picture::{Picture, PictureType},
    tag::Tag,
};
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::music_lib::CACHE_DIR;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoverError {
    NoPictures,
    NoSuitablePictures,
    CoverWriteError(String),
    CacheDirError(String),
    UnknownFileType,
}

impl std::fmt::Display for CoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPictures => write!(f, "Could not find any pictures in the tag"),
            Self::NoSuitablePictures => write!(
                f,
                "The tag contains pictures, but none of them can be used as cover art replacement"
            ),
            Self::CoverWriteError(e) => write!(f, "Could not write the cover image to cache: {e}"),
            Self::CacheDirError(e) => write!(f, "Could not create the cache directory: {e}"),
            CoverError::UnknownFileType => write!(f, "Could not determine the image type"),
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

pub(crate) static COVERS_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
    cache.push("covers/");
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
) -> Result<PathBuf, CoverError> {
    let cover_cache = COVERS_CACHE_DIR.clone();
    if !cover_cache.is_dir()
        && let Err(e) = create_dir_all(&cover_cache)
    {
        error!("Could not create the cache directory {cover_cache:?}: {e}");
        return Err(CoverError::CacheDirError(e.to_string()));
    }

    let pic = pick_front_cover_or_replacement(tag.pictures())?.data();
    let extension = if let Some(extension) = infer_image::extension(pic) {
        extension
    } else {
        warn!("Could not determine the image type for track {track_hash}");
        return Err(CoverError::UnknownFileType);
    };

    let mut cover_path = cover_cache.clone();
    cover_path.push(format!("{track_hash}.{extension}"));

    if load_mode == &LoadMode::Load && cover_path.is_file() {
        return Ok(cover_path);
    }

    let mut file = match safe_file_create(&cover_path) {
        Ok(file) => file,
        Err(e) => {
            error!(
                "Could not open the cover image file in the cache directory in {cover_path:?}: {e}"
            );
            return Err(CoverError::CoverWriteError(e.to_string()));
        }
    };

    if let Err(e) = file.write_all(pic) {
        error!("Could not write the cover image to the cache directory: {e}");
        return Err(CoverError::CoverWriteError(e.to_string()));
    }

    Ok(cover_path)
}
