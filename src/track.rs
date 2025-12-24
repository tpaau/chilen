use std::{hash::Hash, path::PathBuf};

use cxx_qt_lib::QString;
use lofty::tag::{Accessor, Tag};
use serde::{Deserialize, Serialize};

use crate::cache::{CacheError, coversdb::get_track_cover};

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, path)]
        #[qproperty(QString, cover_path)]
        #[qproperty(QString, thumbnail_path)]
        #[qproperty(QString, artist)]
        #[qproperty(QString, title)]
        #[qproperty(QString, album)]
        #[qproperty(QString, genre)]
        #[qproperty(QString, comment)]
        #[qproperty(i32, track)]
        #[qproperty(i32, track_total)]
        #[qproperty(i32, disk)]
        #[qproperty(i32, disk_total)]
        #[qproperty(i32, year)]
        #[namespace = "track"]
        type QTrack = super::RQTrack;
    }
}

#[derive(Default)]
pub struct RQTrack {
    pub path: QString,
    pub cover_path: QString,
    pub thumbnail_path: QString,
    pub artist: QString,
    pub title: QString,
    pub album: QString,
    pub genre: QString,
    pub comment: QString,
    pub track: i32,
    pub track_total: i32,
    pub disk: i32,
    pub disk_total: i32,
    pub year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub path: PathBuf,
    pub cover_path: Option<PathBuf>,
    pub thumbnail_path: Option<PathBuf>,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub comment: Option<String>,
    pub track: Option<u32>,
    pub track_total: Option<u32>,
    pub disk: Option<u32>,
    pub disk_total: Option<u32>,
    pub year: Option<u32>,
}

impl From<RQTrack> for Track {
    fn from(v: RQTrack) -> Self {
        Track {
            path: PathBuf::from(String::from(&v.path)),
            cover_path: if !v.path.is_empty() && !v.path.is_null() {
                Some(PathBuf::from(String::from(v.cover_path)))
            } else {
                None
            },
            thumbnail_path: if !v.path.is_empty() && !v.path.is_null() {
                Some(PathBuf::from(String::from(v.thumbnail_path)))
            } else {
                None
            },
            artist: if !v.artist.is_empty() && !v.artist.is_null() {
                Some(String::from(v.artist))
            } else {
                None
            },
            title: if !v.title.is_empty() && !v.title.is_null() {
                Some(String::from(v.title))
            } else {
                None
            },
            album: if !v.album.is_empty() && !v.album.is_null() {
                Some(String::from(v.album))
            } else {
                None
            },
            genre: if !v.genre.is_empty() && !v.genre.is_null() {
                Some(String::from(v.genre))
            } else {
                None
            },
            comment: if !v.comment.is_empty() && !v.comment.is_null() {
                Some(String::from(v.comment))
            } else {
                None
            },
            track: if v.track >= 0 {
                Some(v.track.try_into().unwrap())
            } else {
                None
            },
            track_total: if v.track_total >= 0 {
                Some(v.track_total.try_into().unwrap())
            } else {
                None
            },
            disk: if v.disk >= 0 {
                Some(v.disk.try_into().unwrap())
            } else {
                None
            },
            disk_total: if v.disk_total >= 0 {
                Some(v.disk_total.try_into().unwrap())
            } else {
                None
            },
            year: if v.year >= 0 {
                Some(v.year.try_into().unwrap())
            } else {
                None
            },
        }
    }
}

impl From<&Tag> for Track {
    fn from(tag: &Tag) -> Self {
        Track {
            path: PathBuf::new(),
            cover_path: None,
            thumbnail_path: None,
            artist: tag.artist().map(|artist| artist.into()),
            title: tag.title().map(|title| title.into()),
            album: tag.album().map(|album| album.into()),
            genre: tag.genre().map(|genre| genre.into()),
            comment: tag.comment().map(|comment| comment.into()),
            track: tag.track(),
            track_total: tag.track_total(),
            disk: tag.disk(),
            disk_total: tag.disk_total(),
            year: tag.year(),
        }
    }
}

impl Hash for Track {
    /// Hashes everything but the paths, ie. only the metadata.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.artist.hash(state);
        self.title.hash(state);
        self.album.hash(state);
        self.genre.hash(state);
        self.comment.hash(state);
        self.track.hash(state);
        self.track_total.hash(state);
        self.disk.hash(state);
        self.disk_total.hash(state);
        self.year.hash(state);
    }
}

impl Track {
    pub fn get_cover(&mut self, tag: &Tag) -> Result<(), CacheError> {
        match get_track_cover(self, tag) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn with_cover_from_tag(tag: &Tag) -> Result<Track, Box<(Track, CacheError)>> {
        let mut track = Track::from(tag);
        match track.get_cover(tag) {
            Ok(_) => Ok(track),
            Err(e) => Err(Box::new((track, e))),
        }
    }
}
