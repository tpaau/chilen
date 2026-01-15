use std::{hash::Hash, path::PathBuf};

use lofty::tag::{Accessor, ItemValue, Tag};

use crate::cache::{CacheError, covers::get_track_cover};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Track {
    pub path: PathBuf,
    pub cover_path: Option<PathBuf>,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub lyrics: Option<String>,
    pub comment: Option<String>,
    pub track: Option<u32>,
    pub track_total: Option<u32>,
    pub disk: Option<u32>,
    pub disk_total: Option<u32>,
    pub year: Option<u32>,
}

impl Into<mpipc::Track> for Track {
    fn into(self) -> mpipc::Track {
        mpipc::Track {
            path: self.path,
            cover_path: self.cover_path,
            artist: self.artist,
            title: self.title,
            album: self.album,
            genre: self.genre,
            comment: self.comment,
            track: self.track,
            track_total: self.track_total,
            disk: self.disk,
            disk_total: self.disk_total,
            year: self.year,
            lyrics: self.lyrics,
        }
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            self.artist.clone().unwrap_or(String::from("Unknown")),
            self.title.clone().unwrap_or(String::from("Unknown")),
        )
    }
}

impl From<&Tag> for Track {
    fn from(tag: &Tag) -> Self {
        let lyrics = match tag.get(&lofty::tag::ItemKey::Lyrics) {
            Some(tag_item) => match tag_item.value() {
                ItemValue::Text(lyrics) => Some(lyrics.clone()),
                _ => None,
            },
            None => None,
        };

        Track {
            path: PathBuf::new(),
            cover_path: None,
            artist: tag.artist().map(|artist| artist.into()),
            title: tag.title().map(|title| title.into()),
            album: tag.album().map(|album| album.into()),
            genre: tag.genre().map(|genre| genre.into()),
            lyrics,
            comment: tag.comment().map(|comment| comment.into()),
            track: tag.track(),
            track_total: tag.track_total(),
            disk: tag.disk(),
            disk_total: tag.disk_total(),
            year: tag.year(),
        }
    }
}

impl Track {
    pub fn get_cover(&mut self, tag: &Tag) -> Result<(), CacheError> {
        get_track_cover(self, tag, false)
    }

    pub fn extract_cover(&mut self, tag: &Tag) -> Result<(), CacheError> {
        get_track_cover(self, tag, true)
    }
}
