use std::{
    collections::HashSet,
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::BufReader,
    ops::Deref,
    path::PathBuf,
    sync::RwLock,
    time::Duration,
};

use lofty::{
    file::{AudioFile, TaggedFile, TaggedFileExt},
    tag::{Accessor, ItemValue, items::Timestamp},
};
use log::warn;
use lrc_rs::SyncedLyrics;
#[cfg(feature = "mpris")]
use mpris_server::TrackId;
use rodio::Decoder;

#[cfg(test)]
use crate::music_lib::indexer::covers::{self};
use crate::music_lib::{
    self,
    indexer::covers::{Cover, get_track_cover},
};

/// Lyrics data, can be either synced or unsynced.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lyrics {
    /// Synced lyrics parsed from the LRC format.
    Synced(Box<SyncedLyrics>),
    /// Unsynced lyrics as a string.
    Unsynced(String),
}

#[cfg_attr(test, derive(Default))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Track {
    /// Path to the audio file.
    pub path: PathBuf,
    /// Cover art for the track.
    pub cover: Cover,
    pub duration: Duration,
    pub title: Option<String>,
    pub artists: Option<Vec<String>>,
    pub genres: Option<Vec<String>>,
    /// Name of the album the track belongs to.
    pub album: Option<String>,
    pub lyrics: Option<Lyrics>,
    pub comment: Option<String>,
    /// Index of the track in the album.
    pub track: Option<u32>,
    /// Total number of tracks in the album.
    pub track_total: Option<u32>,
    pub disc: Option<u32>,
    pub disc_total: Option<u32>,
    pub date: Option<Timestamp>,
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            self.artists
                .clone()
                .unwrap_or(vec![String::from("Unknown")])
                .join(", "),
            self.title.clone().unwrap_or(String::from("Unknown")),
        )
    }
}

impl Track {
    pub(crate) fn new(
        path: PathBuf,
        tagged_file: &TaggedFile,
        config: &music_lib::Config,
        covers_lookup_set: &RwLock<HashSet<PathBuf>>,
    ) -> Result<Self, String> {
        let tag = match tagged_file.primary_tag() {
            Some(tag) => tag,
            None => {
                return Err(String::from("The provided tagged file had no tags"));
            }
        };

        let lyrics = match tag.get(lofty::tag::ItemKey::Lyrics) {
            Some(tag_item) => match tag_item.value() {
                ItemValue::Text(lyrics) => Some(lyrics),
                _ => None,
            },
            None => None,
        };

        let lyrics = if let Some(lyrics) = lyrics {
            match SyncedLyrics::parse(lyrics) {
                Ok(synced_lyrics) => Some(Lyrics::Synced(Box::new(synced_lyrics))),
                Err(_) => Some(Lyrics::Unsynced(lyrics.to_string())),
            }
        } else {
            None
        };

        let artists = {
            let artists: Vec<_> = tag.get_strings(lofty::tag::ItemKey::TrackArtists).collect();
            if artists.len() <= 1 {
                tag.artist().map(|a| {
                    a.split(|c| {
                        config
                            .value_separators
                            .artist
                            .iter()
                            .any(|s| s.chars().any(|sc| sc == c))
                    })
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
                })
            } else {
                Some(artists.into_iter().map(|s| s.trim().to_string()).collect())
            }
        };

        let genres: Option<Vec<_>> = tag.genre().map(|genre| {
            genre
                .split(|c| {
                    config
                        .value_separators
                        .genre
                        .iter()
                        .any(|s| s.chars().any(|sc| sc == c))
                })
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        });

        let cover = {
            #[cfg(not(test))]
            match get_track_cover(tag, config.indexer, covers_lookup_set) {
                Ok(cover) => cover,
                Err(e) => {
                    warn!("Could not get the cover image: {e}");
                    Cover::none()
                }
            }
            #[cfg(test)]
            if config.indexer.cache_mode != covers::CacheMode::Disabled {
                match get_track_cover(tag, config.indexer, covers_lookup_set) {
                    Ok(cover) => cover,
                    Err(e) => {
                        warn!("Could not get the cover image: {e}");
                        Cover::none()
                    }
                }
            } else {
                Cover::none()
            }
        };

        let track = Track {
            path,
            cover,
            duration: tagged_file.properties().duration(),
            artists,
            title: tag.title().map(|title| title.into()),
            album: tag.album().map(|album| album.into()),
            genres,
            lyrics,
            comment: tag.comment().map(|comment| comment.into()),
            track: tag.track(),
            track_total: tag.track_total(),
            disc: tag.disk(),
            disc_total: tag.disk_total(),
            date: tag.date(),
        };

        Ok(track)
    }

    pub fn hash_self(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn hash_track(track: &Track) -> u64 {
        let mut hasher = DefaultHasher::new();
        track.hash(&mut hasher);
        hasher.finish()
    }

    pub fn hash_tracks<I, T>(tracks: I) -> Vec<u64>
    where
        I: IntoIterator<Item = T>,
        T: Deref<Target = Track>,
    {
        tracks
            .into_iter()
            .map(|track| Self::hash_track(track.deref()))
            .collect()
    }

    // TODO: Am I doing this right???
    #[cfg(feature = "mpris")]
    pub fn track_id(&self, position: usize) -> TrackId {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        position.hash(&mut hasher);
        let hash = hasher.finish();
        TrackId::try_from(format!("/org/mpris/MediaPlayer2/TrackList/queue/{hash}")).unwrap()
    }

    #[cfg(feature = "mpris")]
    pub fn get_meta(&self, position: usize) -> mpris_server::Metadata {
        use mpris_server::{Time, builder::MetadataBuilder};

        let builder = MetadataBuilder::default()
            .length(Time::from_nanos(
                self.duration.as_nanos().try_into().unwrap_or(i64::MAX),
            ))
            .url(self.path.to_string_lossy())
            .artist(
                self.artists
                    .clone()
                    .unwrap_or(vec!["Unknown artist".to_string()]),
            )
            .title(self.title.clone().unwrap_or_default())
            .album(self.album.clone().unwrap_or_default())
            .genre(
                self.genres
                    .clone()
                    .unwrap_or(vec!["Unknown genre".to_string()]),
            )
            .comment(self.comment.clone())
            .track_number(self.track.unwrap_or(0).try_into().unwrap_or(0))
            .disc_number(self.disc.unwrap_or(0).try_into().unwrap_or(0))
            .lyrics(match &self.lyrics {
                Some(lyrics) => {
                    use lrc_rs::LyricsAccess;

                    match lyrics {
                        Lyrics::Synced(synced) => synced.clone().to_unsynced(),
                        Lyrics::Unsynced(unsynced) => unsynced.to_string(),
                    }
                }
                None => String::new(),
            })
            .trackid(self.track_id(position));

        let builder = match &self.cover.hires {
            Some(cover) => builder.art_url(cover.to_string_lossy().to_string()),
            None => builder,
        };

        builder.build()
    }

    pub fn open_file(&self) -> std::io::Result<File> {
        match File::open(&self.path) {
            Ok(file) => Ok(file),
            Err(e) => Err(e),
        }
    }

    pub fn open_source(&self) -> Result<Decoder<BufReader<File>>, String> {
        let file = match self.open_file() {
            Ok(file) => file,
            Err(e) => return Err(e.to_string()),
        };
        match Decoder::try_from(file) {
            Ok(source) => Ok(source),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Returns a [Vec] of unique [Track] structs for testing.
    #[cfg(test)]
    pub fn unique_tracks(size: usize) -> Vec<Track> {
        let mut tracks = Vec::new();
        for i in 0..size {
            let track = Track {
                duration: Duration::from_secs(i.try_into().unwrap()),
                path: format!("/test/path/{i}").into(),
                ..Default::default()
            };
            tracks.push(track);
        }
        tracks
    }
}
