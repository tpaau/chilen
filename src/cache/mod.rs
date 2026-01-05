pub mod covers;
pub mod playlists;
mod indexer;

use clap::crate_name;
use std::{env::home_dir, fs::create_dir_all, path::PathBuf, sync::LazyLock};

pub static CACHE_DIR: LazyLock<Result<PathBuf, CacheError>> = LazyLock::new(|| {
    if let Some(mut cache) = home_dir() {
        cache.push(format!(".cache/{}/", crate_name!()));
        match create_dir_all(&cache) {
            Ok(_) => Ok(cache),
            Err(e) => Err(CacheError::DirError {
                error: e.to_string(),
            }),
        }
    } else {
        Err(CacheError::HomeError)
    }
});

#[derive(Debug, Clone)]
pub enum CacheError {
    HomeError,
    DirError { error: String },
    NoPicturesInTag,
    NoSuitablePicturesInTag,
    CoverWriteError,
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HomeError => write!(f, "Could not get the path to the home directory"),
            Self::DirError { error } => {
                write!(f, "The cache directory could not be created: {error}")
            }
            Self::NoPicturesInTag => write!(f, "The provided tag did not contain any pictures"),
            Self::NoSuitablePicturesInTag => write!(
                f,
                "The provided tag contained some pictures, but none of them were suitable"
            ),
            Self::CoverWriteError => write!(f, "Could not write the cover image to the cache"),
        }
    }
}
