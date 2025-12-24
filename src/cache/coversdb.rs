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
use rusqlite::Connection;

use crate::{
    cache::{CACHE_DIR, CacheError},
    track::Track,
};

pub static COVERS_DB: LazyLock<Result<PathBuf, CacheError>> =
    LazyLock::new(|| match CACHE_DIR.clone() {
        Ok(mut cache) => {
            cache.push("coversdb.sqlite");
            Ok(cache)
        }
        Err(e) => Err(e),
    });

pub static COVERS_CACHE_DIR: LazyLock<Result<PathBuf, CacheError>> =
    LazyLock::new(|| match CACHE_DIR.clone() {
        Ok(mut cache) => {
            cache.push("covers/");
            match create_dir_all(&cache) {
                Ok(_) => Ok(cache),
                Err(e) => Err(CacheError::DirError {
                    error: e.to_string(),
                }),
            }
        }
        Err(e) => Err(e),
    });

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

fn open_db() -> Result<Connection, CacheError> {
    let path = match COVERS_DB.clone() {
        Ok(path) => path,
        Err(e) => {
            return Err(e);
        }
    };

    match Connection::open(path) {
        Ok(conn) => Ok(conn),
        Err(e) => Err(CacheError::RusqliteError {
            error: e.to_string(),
        }),
    }
}

fn pick_front_cover_or_replacement(pictures: &[Picture]) -> Result<&Picture, CacheError> {
    if pictures.is_empty() {
        return Err(CacheError::NoPicturesInTag);
    }

    for pic_type in FRONT_COVER_PRIORITY {
        for pic in pictures {
            if pic.pic_type() == pic_type {
                return Ok(pic);
            }
        }
    }

    Err(CacheError::NoSuitablePicturesInTag)
}

pub fn get_track_cover(track: &mut Track, tag: &Tag) -> Result<(), CacheError> {
    let db = open_db()?;

    let front = pick_front_cover_or_replacement(tag.pictures())?;

    let mut hasher = DefaultHasher::new();
    track.hash(&mut hasher);
    let hash = hasher.finish();

    let cover_path = match COVERS_CACHE_DIR.clone() {
        Ok(mut covers_cache) => {
            covers_cache.push(hash.to_string());
            covers_cache
        }
        Err(e) => {
            return Err(e);
        }
    };

    let mut file = match File::create(cover_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Could not open the cover image file in the cache directory: {e}");
            return Err(CacheError::CoverWriteError);
        }
    };

    if let Err(e) = file.write_all(front.data()) {
        error!("Could not write the cover image to the cache directory: {e}");
        return Err(CacheError::CoverWriteError);
    }

    Ok(())
}
