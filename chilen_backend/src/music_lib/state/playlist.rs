use std::{hash::Hash, sync::Arc, time::Duration};

use log::warn;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::music_lib::indexer::CacheMode;
use crate::{
    Error,
    music_lib::{
        indexer::{
            self,
            covers::{Cover, get_playlist_cover},
        },
        state::{MusicLibrary, Track},
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Playlist {
    pub name: Box<str>,
    pub tracks: Box<[Arc<Track>]>,
    pub duration: Duration,
    pub cover: Cover,
    /// Unique per-session ID for identifying the playlist in the library.
    ///
    /// This ID is not dependent on the contents of the playlist or it's name, and once a playlist
    /// with a specific ID has been created, no new playlist can reclaim it.
    pub id: u64,
    pub(crate) unmatched: Box<[u64]>,
}

impl Playlist {
    pub(super) fn load(
        lib: &mut MusicLibrary,
        loaded: ConfPlaylist,
        config: indexer::Config,
    ) -> Self {
        let result = lib.tracks_from_hashes(loaded.track_hashes);
        if !result.unmatched.is_empty() {
            warn!(
                "{} missing tracks in playlist {}",
                result.unmatched.len(),
                loaded.name
            );
        }
        let duration = result.matched.iter().map(|t| t.duration).sum();

        #[cfg(not(test))]
        let cover =
            get_playlist_cover(&loaded.name, config, &result.matched).unwrap_or(Cover::none());
        #[cfg(test)]
        let cover = if config.cache_mode != CacheMode::Disabled {
            get_playlist_cover(&loaded.name, config, &result.matched).unwrap_or(Cover::none())
        } else {
            Cover::none()
        };

        Self {
            name: loaded.name.into(),
            tracks: result.matched.into(),
            duration,
            unmatched: result.unmatched.into(),
            id: lib.get_playlist_id(),
            cover,
        }
    }

    pub(crate) fn remove_tracks(&mut self, mut ids: Vec<usize>) -> Result<(), Error> {
        ids.sort_unstable();

        if ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(Error::DuplicateItems);
        }
        if ids.iter().any(|&i| i >= self.tracks.len()) {
            return Err(Error::IndexOutOfBounds);
        }

        let mut ids = ids.into_iter().peekable();
        let mut tracks = std::mem::take(&mut self.tracks).into_vec();
        let mut index = 0;

        tracks.retain(|_| {
            let remove = ids.peek() == Some(&index);
            if remove {
                ids.next();
            }
            index += 1;
            !remove
        });

        self.tracks = tracks.into_boxed_slice();

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ConfPlaylist {
    name: String,
    track_hashes: Vec<u64>,
}

impl From<Playlist> for ConfPlaylist {
    fn from(value: Playlist) -> Self {
        let mut track_hashes: Vec<_> = value.tracks.iter().map(|t| t.hash_self()).collect();
        track_hashes.extend(value.unmatched);
        Self {
            name: value.name.to_string(),
            track_hashes,
        }
    }
}
