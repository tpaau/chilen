use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use lofty::{file::TaggedFileExt, read_from_path};
use log::{info, trace, warn};
use walkdir::WalkDir;

use crate::track::Track;

pub fn index(music_dir: Option<PathBuf>) {
    trace!("Indexing the music directory");

    let tracks = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    let time_start = SystemTime::now();

    for result in WalkDir::new("/var/home/mikolaj/Music/").into_iter() {
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
            handles.push(thread::spawn(move || match read_from_path(entry.path()) {
                Ok(file) => {
                    match file.primary_tag() {
                        Some(tag) => {
                            let track = Track::from(tag);
                            lock.lock().unwrap().push(file);
                        }
                        None => {
                            warn!(
                                "Found an audio file with no tags: {:?}. Ignoring",
                                entry.path()
                            );
                        }
                    };
                }
                Err(e) => match e.kind() {
                    lofty::error::ErrorKind::UnknownFormat => {
                        info!("Likely not an audio file: {:?}", entry.path());
                    }
                    _ => {
                        warn!("Could not get tags from file {:?}: {e}", entry.path());
                    }
                },
            }));
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));

    trace!(
        "Finished indexing the music directory in {:.3}s, found {} audio files",
        time_elapsed.as_secs_f64(),
        tracks.lock().unwrap().len()
    );
}
