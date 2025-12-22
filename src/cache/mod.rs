mod coversdb;

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
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HomeError => write!(f, "Could not get the home directory"),
            Self::DirError { error } => {
                write!(f, "The cache directory could not be created: {error}")
            }
        }
    }
}
