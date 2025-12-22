use std::{
    env::home_dir,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use lofty::{file::TaggedFileExt, read_from_path};
use log::{info, trace, warn};
use walkdir::WalkDir;

use crate::track::Track;

pub enum IndexingError {
    HomeDirNotFound,
    ArcInnerError,
}

impl std::fmt::Display for IndexingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HomeDirNotFound => {
                write!(f, "Could not obtain the home directory to index.")
            }
            Self::ArcInnerError => {
                write!(f, "Could not get the underlying `Arc` data.")
            }
        }
    }
}

pub fn index<T: Into<PathBuf>>(music_dir: Option<T>) -> Result<Vec<Track>, IndexingError>
where
    PathBuf: From<T>,
{
    let music_dir = match music_dir {
        Some(dir) => PathBuf::from(dir),
        None => match home_dir() {
            Some(mut dir) => {
                dir.push("Music/");
                dir
            }
            None => {
                return Err(IndexingError::HomeDirNotFound);
            }
        },
    };

    trace!("Indexing directory: {music_dir:?}");

    let tracks = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    let time_start = SystemTime::now();

    for result in WalkDir::new(music_dir).into_iter() {
        let entry = match result {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Error while trying to access the file: {e}");
                continue;
            }
        };

        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(e) => {
                warn!("Could not get `DirEntry` metadata: {e}");
                continue;
            }
        };

        if meta.is_file() {
            let lock = Arc::clone(&tracks);
            handles.push(thread::spawn(move || {
                let file = match read_from_path(entry.path()) {
                    Ok(file) => file,
                    Err(e) => {
                        match e.kind() {
                            lofty::error::ErrorKind::UnknownFormat => {
                                info!("Likely not an audio file: {:?}", entry.path());
                            }
                            _ => {
                                warn!("Could not get tags from file {:?}: {e}", entry.path());
                            }
                        }
                        return;
                    }
                };

                let tag = match file.primary_tag() {
                    Some(tag) => tag,
                    None => {
                        warn!(
                            "Found an audio file with no tags: {:?}. Ignoring",
                            entry.path()
                        );
                        return;
                    }
                };

                let mut track = match Track::with_cover_from_tag(tag) {
                    Ok(track) => track,
                    Err(e) => {
                        trace!("{:?}", e.error);
                        e.track
                    }
                };
                let mut path = PathBuf::new();
                path.push(entry.path());
                track.path = path;
                lock.lock().unwrap().push(track)
            }));
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    match Arc::into_inner(tracks) {
        Some(lock) => {
            let vec = lock.into_inner().unwrap();
            let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));
            trace!(
                "Finished indexing the music directory in {:.3}s, found {} audio files",
                time_elapsed.as_secs_f64(),
                vec.len()
            );
            Ok(vec)
        }
        None => Err(IndexingError::ArcInnerError),
    }
}
