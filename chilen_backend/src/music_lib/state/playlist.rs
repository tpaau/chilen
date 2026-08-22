use std::{collections::HashSet, hash::Hash, sync::Arc, time::Duration};

use log::warn;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::music_lib::indexer::covers::CacheMode;
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
    pub name: String,
    pub tracks: Vec<Arc<Track>>,
    pub duration: Duration,
    pub cover: Cover,
    pub(crate) unmatched: Vec<u64>,
}

impl Playlist {
    pub(super) fn load(lib: &MusicLibrary, loaded: ConfPlaylist, config: indexer::Config) -> Self {
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
            name: loaded.name,
            tracks: result.matched,
            duration,
            unmatched: result.unmatched,
            cover,
        }
    }

    pub(crate) fn remove_tracks(&mut self, mut ids: Vec<usize>) -> Result<(), Error> {
        ids.sort();
        let mut unique = ids.clone();
        unique.dedup();
        if unique != ids {
            return Err(Error::DuplicateItems);
        }
        if ids.iter().find(|i| i >= &&self.tracks.len()).is_some() {
            return Err(Error::IndexOutOfBounds);
        }
        let remove_set: HashSet<usize> = ids.into_iter().collect();
        self.tracks = self
            .tracks
            .drain(..)
            .enumerate()
            .filter_map(|(i, t)| {
                if remove_set.contains(&i) {
                    None
                } else {
                    Some(t)
                }
            })
            .collect();
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
            name: value.name,
            track_hashes,
        }
    }
}
