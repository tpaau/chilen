use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use lofty::read_from_path;
use log::{info, trace, warn};
use walkdir::WalkDir;

pub fn index() {
    trace!("Indexing the music directory");

    let tracks = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    let time_start = SystemTime::now();

    for result in WalkDir::new("/var/home/mikolaj/Music/").into_iter() {
        if let Ok(entry) = result {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    let lock = Arc::clone(&tracks);
                    handles.push(thread::spawn(move || match read_from_path(entry.path()) {
                        Ok(file) => {
                            lock.lock().unwrap().push(file);
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
        } else {
            warn!(
                "Error while trying to access the file: {}",
                result.unwrap_err()
            );
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
