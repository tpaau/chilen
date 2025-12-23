use std::{path::PathBuf, sync::LazyLock};

use rusqlite::Connection;

use crate::cache::{CACHE_DIR, CacheError};

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
            Ok(cache)
        }
        Err(e) => Err(e),
    });

fn open() -> Result<Connection, CacheError> {
    let path = match COVERS_DB.clone() {
        Ok(path) => path,
        Err(e) => {
            return Err(e);
        }
    };

    match Connection::open(path) {
        Ok(conn) => Ok(conn),
        Err(e) => {
            Err(CacheError::RusqliteError { error: e.to_string() })
        }
    }
}
