use std::{
    fs::{File, create_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::PathBuf,
    sync::LazyLock,
};

use lofty::{
    picture::{Picture, PictureType},
    tag::Tag,
};
use log::error;
use serde::{Deserialize, Serialize};

use crate::{CACHE_DIR, music_lib::Track};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum CoverError {
    NoPictures,
    NoSuitablePictures,
    CoverWriteError(String),
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
pub(crate) fn get_track_cover(
    track: &mut Track,
    tag: &Tag,
    ignore_cache: bool,
) -> Result<(), CoverError> {
    let pic = pick_front_cover_or_replacement(tag.pictures())?;

    let mut hasher = DefaultHasher::new();
    track.hash(&mut hasher);
    let hash = hasher.finish();

    let cover_cache = COVERS_CACHE_DIR.clone();
    if !cover_cache.is_dir()
        && let Err(e) = create_dir_all(&cover_cache)
    {
        error!("Could not create the cover cache directory {cover_cache:?}: {e}");
    }

    let mut cover_path = cover_cache.clone();
    cover_path.push(hash.to_string());

    if !ignore_cache && cover_path.is_file() {
        track.cover_path = Some(cover_path);
        return Ok(());
    }

    let mut file = match File::create(&cover_path) {
        Ok(file) => file,
        Err(e) => {
            error!(
                "Could not open the cover image file in the cache directory in {cover_path:?}: {e}"
            );
            return Err(CoverError::CoverWriteError(e.to_string()));
        }
    };

    if let Err(e) = file.write_all(pic.data()) {
        error!("Could not write the cover image to the cache directory: {e}");
        return Err(CoverError::CoverWriteError(e.to_string()));
    }

    track.cover_path = Some(cover_path);

    Ok(())
}
