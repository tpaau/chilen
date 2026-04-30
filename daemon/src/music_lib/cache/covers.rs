use std::{
    fs::{File, create_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::PathBuf,
    sync::LazyLock,
};

use lofty::{picture::PictureType, tag::Tag};
use log::error;
use serde::{Deserialize, Serialize};

use crate::{CACHE_DIR, music_lib::Track};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum CoverError {
    NoPicturesInTag,
    NoFrontCover,
    CoverWriteError(String),
}

impl std::fmt::Display for CoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPicturesInTag => write!(f, "Could not find any pictures in the tag"),
            Self::NoFrontCover => write!(f, "The tag contains pictures, but no front cover"),
            Self::CoverWriteError(e) => write!(f, "Could not write the cover image to cache: {e}"),
        }
    }
}

pub(crate) static COVERS_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut cache = CACHE_DIR.read().unwrap().clone().unwrap();
    cache.push("covers/");
    cache
});

pub(crate) fn get_track_cover(
    track: &mut Track,
    tag: &Tag,
    ignore_cache: bool,
) -> Result<(), CoverError> {
    let pic = tag
        .pictures()
        .iter()
        .find(|p| p.pic_type() == PictureType::CoverFront);

    // TODO: Revert to the old cover selection (that one worked :/)
    if let Some(front) = pic {
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

        if let Err(e) = file.write_all(front.data()) {
            error!("Could not write the cover image to the cache directory: {e}");
            return Err(CoverError::CoverWriteError(e.to_string()));
        }

        track.cover_path = Some(cover_path);

        Ok(())
    } else {
        Err(CoverError::NoPicturesInTag)
    }
}
