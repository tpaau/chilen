use std::{
    collections::{HashMap, HashSet},
    fs::{File, read},
    hash::{DefaultHasher, Hash, Hasher},
    io::{BufReader, Write},
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, RwLock},
    time::{Duration, SystemTime},
};

use icu::collator::CollatorBorrowed;
use lofty::{
    file::{AudioFile, TaggedFile, TaggedFileExt},
    tag::{Accessor, ItemValue, items::Timestamp},
};
use log::{error, trace, warn};
use lrc_rs::SyncedLyrics;
#[cfg(feature = "mpris")]
use mpris_server::TrackId;
use rmp_serde::{Deserializer, Serializer};
use rodio::Decoder;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::music_lib::indexer::covers::{self, CacheMode};
use crate::{
    COLLATOR, Error, Event, get_config,
    music_lib::indexer::covers::get_playlist_cover,
    music_lib::{
        self, DATA_DIR,
        indexer::{
            self,
            covers::{Cover, get_track_cover},
        },
        tracks_from_m3u8,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Progress {
    /// First step of the loading process.
    ///
    /// The indexer finds files in the music library to index.
    ///
    /// This step usually takes under 100ms.
    FindingTracks,

    /// Second step of the loading process.
    ///
    /// The indexer goes through all of the found files and attempts to gather audio metadata from
    /// them. During this process track covers are also cached.
    ///
    /// This is the longest step of the loading process. The time it takes to complete this step
    /// heavily depends on on whether track covers are already cached (warm start) or not (cold
    /// start), and how many tracks are in the library.
    ///
    /// Eg. for my 908-track music library, cold start takes about 4.24s, and warm start just 0.26s.
    Indexing,

    /// Third step of the loading process.
    ///
    /// After all the tracks have been gathered, the backend rebuilds the virtual library from the
    /// track tags. It rebuilds artists, albums and genres, as well as hash maps used to quickly
    /// find items in the music library.
    ///
    /// Usually takes under a 100ms, depending on library size.
    RebuildingLibrary,

    /// Fourth step of the loading process.
    ///
    /// The playlists are restored from disk.
    ///
    /// **NOTE**: This step is skipped if the `playlists` file is not present.
    RestoringState,

    /// The loading process has finished.
    Done,

    /// The loading process has failed irrecoverably and will not restart.
    Failed(Error),
}

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
    pub path: PathBuf,
    pub cover: Cover,
    pub duration: Duration,
    pub title: Option<String>,
    pub artists: Option<Vec<String>>,
    pub genres: Option<Vec<String>>,
    pub album: Option<String>,
    pub lyrics: Option<Lyrics>,
    pub comment: Option<String>,
    pub track: Option<u32>,
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Album {
    pub title: String,
    pub cover: Cover,
    pub artists: Vec<String>,
    pub tracks: Vec<Arc<Track>>,
    pub date: Option<Timestamp>,
    pub duration: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Artist {
    pub name: String,
    pub cover: Cover,
    pub tracks: Vec<Arc<Track>>,
    pub albums: Vec<Arc<Album>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Genre {
    pub name: String,
    pub cover: Cover,
    pub artists: Vec<Arc<Artist>>,
    pub albums: Vec<Arc<Album>>,
    pub tracks: Vec<Arc<Track>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Arc<Track>>,
    pub duration: Duration,
    pub cover: Cover,
    #[cfg(not(test))]
    unmatched: Vec<u64>,
    #[cfg(test)]
    pub unmatched: Vec<u64>,
}

impl Playlist {
    fn load(lib: &MusicLibrary, loaded: ConfPlaylist, config: indexer::Config) -> Self {
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
struct ConfPlaylist {
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

#[derive(Clone, Debug, PartialEq)]
pub struct MusicLibrary {
    pub playlists: HashSet<Arc<Playlist>>,
    pub tracks: Vec<Arc<Track>>,
    pub albums: Vec<Arc<Album>>,
    pub artists: Vec<Arc<Artist>>,
    pub genres: Vec<Arc<Genre>>,
    tracks_by_path: HashMap<String, Arc<Track>>,
    tracks_by_hash: HashMap<u64, Arc<Track>>,
    playlists_by_name: HashMap<String, Arc<Playlist>>,
    artists_by_name: HashMap<String, Arc<Artist>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HashMatchingResult {
    pub matched: Vec<Arc<Track>>,
    pub unmatched: Vec<u64>,
}

impl MusicLibrary {
    fn sort_tracks(tracks: &mut [Arc<Track>], collator: Option<&Arc<CollatorBorrowed<'_>>>) {
        if let Some(collator) = collator {
            tracks.sort_by(|t1, t2| {
                collator.compare(
                    t1.title.as_deref().unwrap_or(""),
                    t2.title.as_deref().unwrap_or(""),
                )
            });
        }
    }

    fn sort_albums(albums: &mut [Arc<Album>], collator: Option<&Arc<CollatorBorrowed<'_>>>) {
        if let Some(collator) = collator {
            albums.sort_by(|a1, a2| collator.compare(&a1.title, &a2.title));
        }
    }

    fn sort_artists(artists: &mut [Arc<Artist>], collator: Option<&Arc<CollatorBorrowed<'_>>>) {
        if let Some(collator) = collator {
            artists.sort_by(|a1, a2| collator.compare(&a1.name, &a2.name));
        }
    }

    fn new(tracks: Vec<Track>) -> Self {
        crate::send_event(Event::LoadProgressChanged(Progress::RebuildingLibrary));

        let mut tracks: Vec<_> = tracks.into_iter().map(Arc::new).collect();

        let guard = COLLATOR.read().unwrap();
        let collator = guard.as_ref();
        Self::sort_tracks(&mut tracks, collator);

        let album_titles: HashSet<_> = tracks.iter().flat_map(|t| &t.album).collect();
        let artist_names: HashSet<_> = tracks
            .iter()
            .flat_map(|t| t.artists.as_ref().into_iter().flatten())
            .collect();
        let genre_names: HashSet<_> = tracks
            .iter()
            .flat_map(|t| t.genres.as_ref().into_iter().flatten())
            .collect();

        let mut tracks_by_artist: HashMap<&String, HashSet<Arc<Track>>> =
            HashMap::with_capacity(artist_names.len());
        let mut tracks_by_album: HashMap<&String, HashSet<Arc<Track>>> =
            HashMap::with_capacity(album_titles.len());
        let mut tracks_by_genre: HashMap<&String, HashSet<Arc<Track>>> =
            HashMap::with_capacity(genre_names.len());

        for track in &tracks {
            if let Some(artists) = &track.artists {
                for artist in artists {
                    if let Some(val) = tracks_by_artist.get_mut(artist) {
                        val.insert(track.clone());
                    } else {
                        tracks_by_artist.insert(artist, [track.clone()].into_iter().collect());
                    }
                }
            }
            if let Some(album) = &track.album {
                if let Some(val) = tracks_by_album.get_mut(album) {
                    val.insert(track.clone());
                } else {
                    tracks_by_album.insert(album, [track.clone()].into_iter().collect());
                }
            }
            if let Some(genres) = &track.genres {
                for genre in genres {
                    if let Some(val) = tracks_by_genre.get_mut(genre) {
                        val.insert(track.clone());
                    } else {
                        tracks_by_genre.insert(genre, [track.clone()].into_iter().collect());
                    }
                }
            }
        }

        let mut albums: Vec<_> = album_titles
            .into_iter()
            .map(|title| {
                let mut tracks: Vec<_> = tracks_by_album[title].clone().into_iter().collect();
                Self::sort_tracks(&mut tracks, collator);

                let mut artists: Vec<_> = tracks
                    .iter()
                    .flat_map(|t| t.artists.clone().into_iter().flatten())
                    .collect::<HashSet<String>>()
                    .into_iter()
                    .collect();
                if let Some(collator) = collator {
                    artists.sort_by(|a1, a2| collator.compare(a1, a2));
                }

                let mut counts: HashMap<Cover, usize> = HashMap::with_capacity(tracks.len());
                for track in &tracks {
                    *counts.entry(track.cover.clone()).or_insert(0) += 1;
                }
                let commonest = counts.into_iter().max_by_key(|(_, c)| *c).map(|(k, _)| k);
                let cover = commonest.unwrap_or(tracks[0].cover.clone());

                let date = tracks.iter().filter_map(|t| t.date).max();
                let duration = tracks.iter().map(|t| t.duration).sum();

                Arc::new(Album {
                    title: title.to_string(),
                    cover,
                    tracks,
                    artists,
                    date,
                    duration,
                })
            })
            .collect();
        Self::sort_albums(&mut albums, collator);

        let mut albums_by_artist: HashMap<&String, HashSet<Arc<Album>>> =
            HashMap::with_capacity(artist_names.len());
        for album in &albums {
            for artist in &album.artists {
                if let Some(val) = albums_by_artist.get_mut(&artist) {
                    val.insert(album.clone());
                } else {
                    albums_by_artist.insert(artist, [album.clone()].into_iter().collect());
                }
            }
        }

        let mut artists: Vec<_> = artist_names
            .into_iter()
            .map(|name| {
                let mut tracks: Vec<_> = tracks_by_artist[name].clone().into_iter().collect();
                Self::sort_tracks(&mut tracks, collator);

                let mut albums = if let Some(albums) = albums_by_artist.get(name) {
                    albums.iter().cloned().collect()
                } else {
                    Vec::new()
                };
                Self::sort_albums(&mut albums, collator);

                let cover = tracks
                    .iter()
                    .max_by_key(|t| t.date)
                    .unwrap_or(&tracks[0])
                    .cover
                    .clone();

                Arc::new(Artist {
                    name: name.to_string(),
                    cover,
                    tracks,
                    albums,
                })
            })
            .collect();
        Self::sort_artists(&mut artists, collator);

        let mut artists_by_album: HashMap<&String, HashSet<Arc<Artist>>> =
            HashMap::with_capacity(albums.len());
        for artist in &artists {
            for album in &artist.albums {
                if let Some(val) = artists_by_album.get_mut(&album.title) {
                    val.insert(artist.clone());
                } else {
                    artists_by_album.insert(&album.title, [artist.clone()].into_iter().collect());
                }
            }
        }

        let mut albums_by_genre: HashMap<&String, HashSet<Arc<Album>>> =
            HashMap::with_capacity(albums.len());
        for album in &albums {
            for track in &album.tracks {
                if let Some(genres) = &track.genres {
                    for genre in genres {
                        if let Some(val) = albums_by_genre.get_mut(genre) {
                            val.insert(album.clone());
                        } else {
                            albums_by_genre.insert(genre, [album.clone()].into_iter().collect());
                        }
                    }
                }
            }
        }

        let mut artists_by_genre: HashMap<&String, HashSet<Arc<Artist>>> =
            HashMap::with_capacity(artists.len());
        for artist in &artists {
            for track in &artist.tracks {
                if let Some(genres) = &track.genres {
                    for genre in genres {
                        if let Some(val) = artists_by_genre.get_mut(genre) {
                            val.insert(artist.clone());
                        } else {
                            artists_by_genre.insert(genre, [artist.clone()].into_iter().collect());
                        }
                    }
                }
            }
        }

        let mut genres: Vec<_> = genre_names
            .into_iter()
            .map(|name| {
                let mut tracks: Vec<_> = tracks_by_genre[name].clone().into_iter().collect();
                Self::sort_tracks(&mut tracks, collator);

                let mut counts: HashMap<Cover, usize> = HashMap::with_capacity(tracks.len());
                for track in &tracks {
                    *counts.entry(track.cover.clone()).or_insert(0) += 1;
                }
                let commonest = counts.into_iter().max_by_key(|(_, c)| *c).map(|(k, _)| k);
                let cover = commonest.unwrap_or(tracks[0].cover.clone());

                let mut artists: Vec<_> = artists_by_genre[name].clone().into_iter().collect();
                Self::sort_artists(&mut artists, collator);

                let mut albums: Vec<_> = albums_by_genre[name].clone().into_iter().collect();
                Self::sort_albums(&mut albums, collator);

                Arc::new(Genre {
                    name: name.to_string(),
                    cover,
                    tracks,
                    albums,
                    artists,
                })
            })
            .collect();

        if let Some(collator) = collator {
            genres.sort_by(|g1, g2| collator.compare(&g1.name, &g2.name));
        }

        let mut track_path_map: HashMap<_, _> = HashMap::with_capacity(tracks.len());
        for t in tracks.iter() {
            track_path_map.insert(t.path.to_string_lossy().to_string(), t.clone());
        }

        let mut track_hash_map: HashMap<_, _> = HashMap::with_capacity(tracks.len());
        for t in tracks.iter() {
            track_hash_map.insert(t.hash_self(), t.clone());
        }

        let artists_by_name: HashMap<String, Arc<Artist>> = artists
            .iter()
            .map(|a| (a.name.clone(), a.clone()))
            .collect();

        Self {
            playlists: HashSet::new(),
            tracks,
            albums,
            artists,
            genres,
            tracks_by_path: track_path_map,
            tracks_by_hash: track_hash_map,
            playlists_by_name: HashMap::new(),
            artists_by_name,
        }
    }

    fn load(loaded: ConfMusicLibrary, tracks: Vec<Track>, config: indexer::Config) -> Self {
        crate::send_event(Event::LoadProgressChanged(Progress::RestoringState));

        let mut lib = Self::new(tracks);
        for p in loaded.playlists {
            let playlist = Arc::new(Playlist::load(&lib, p, config));
            lib.playlists.insert(playlist.clone());
            lib.playlists_by_name
                .insert(playlist.name.clone(), playlist);
        }

        lib
    }

    #[cfg(test)]
    pub(crate) fn new_testing(tracks: Vec<Track>) -> Self {
        Self::new(tracks)
    }

    fn check_name(&self, name: &str) -> Result<(), Error> {
        let name = name.trim();
        if name.is_empty() {
            return Err(Error::EmptyName);
        }
        if self.find_playlist(name).is_some() {
            error!("A playlist with name \"{name}\" already exists");
            return Err(Error::PlaylistExists);
        }
        Ok(())
    }

    pub fn find_artist(&self, name: &str) -> Option<&Arc<Artist>> {
        self.artists_by_name.get(name)
    }

    pub fn find_playlist(&self, name: &str) -> Option<&Arc<Playlist>> {
        self.playlists_by_name.get(name)
    }

    pub fn find_track_by_path(&self, path: &Path) -> Option<Arc<Track>> {
        self.tracks_by_path
            .get(&path.to_string_lossy().to_string())
            .cloned()
    }

    /// Returns the default playlist name ("New Playlist").
    ///
    /// If a playlist with the default name exists, then a number will be added to the end of the
    /// playlist name so it's unique, eg. "New Playlist 1", "New Playlist 2", etc.
    pub fn get_default_playlist_name(&self) -> String {
        let mut i = 0;
        let mut playlist_name = DEFAULT_PLAYLIST_NAME.to_string();
        while self.find_playlist(&playlist_name).is_some() {
            i += 1;
            playlist_name = format!("{DEFAULT_PLAYLIST_NAME} {i}");
        }
        playlist_name
    }

    /// Retrieves tracks corresponding to a list of provided hashes.
    pub fn tracks_from_hashes(&self, hashes: Vec<u64>) -> HashMatchingResult {
        // In most cases all tracks will match
        let mut tracks = Vec::with_capacity(hashes.len());
        let mut unmatched = Vec::new();
        for hash in hashes {
            if let Some(track) = self.tracks_by_hash.get(&hash) {
                tracks.push(track.clone());
            } else {
                unmatched.push(hash);
            }
        }
        tracks.shrink_to_fit();
        HashMatchingResult {
            matched: tracks,
            unmatched,
        }
    }

    pub fn remove_playlists(&mut self, mut playlists: Vec<String>) -> Result<(), Error> {
        playlists.sort();
        let mut unique = playlists.clone();
        unique.dedup();
        if unique != playlists {
            return Err(Error::DuplicateItems);
        }

        for name in &playlists {
            if let Some(playlist) = self.find_playlist(name).cloned() {
                self.playlists.remove(&playlist);
                self.playlists_by_name.remove(name);
            } else {
                return Err(Error::UnknownPlaylist(name.to_string()));
            }
        }

        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    pub fn create_playlist(
        &mut self,
        name: String,
        track_paths: &Option<Vec<PathBuf>>,
    ) -> Result<(), Error> {
        trace!("Creating a new playlist \"{name}\" from a list of tracks");

        let name = name.trim();
        self.check_name(name)?;

        let tracks = if let Some(tracks) = track_paths {
            let mut out = Vec::with_capacity(tracks.len());
            for path in tracks {
                if let Some(track) = self.find_track_by_path(path) {
                    out.push(track);
                } else {
                    error!("The track {path:?} was not found in the music library");
                    return Err(Error::UnknownTrackPath(path.to_path_buf()));
                }
            }
            out
        } else {
            Vec::new()
        };

        let duration = tracks.iter().map(|t| t.duration).sum();
        let config = get_config();

        #[cfg(not(test))]
        let cover =
            get_playlist_cover(name, config.library.indexer, &tracks).unwrap_or(Cover::none());
        #[cfg(test)]
        let cover = if config.library.indexer.cache_mode != CacheMode::Disabled {
            get_playlist_cover(name, config.library.indexer, &tracks).unwrap_or(Cover::none())
        } else {
            Cover::none()
        };

        let playlist = Arc::new(Playlist {
            name: name.to_string(),
            tracks,
            duration,
            unmatched: Vec::new(),
            cover,
        });
        self.playlists.insert(playlist.clone());
        self.playlists_by_name.insert(name.to_string(), playlist);
        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    pub fn rename_playlist(&mut self, source: &str, target: &str) -> Result<(), Error> {
        let source = source.trim();
        let target = target.trim();
        trace!("Renaming playlist \"{source}\" to \"{target}\"!");
        self.check_name(target)?;

        let source_playlist = if let Some(source) = self.find_playlist(source) {
            source.clone()
        } else {
            return Err(Error::UnknownPlaylist(source.to_string()));
        };
        let mut playlist = source_playlist.as_ref().clone();
        playlist.name = target.to_string();

        self.playlists.remove(&source_playlist);
        self.playlists_by_name.remove(source);
        let playlist = Arc::new(playlist);
        self.playlists.insert(playlist.clone());
        self.playlists_by_name.insert(target.to_string(), playlist);

        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    pub fn add_tracks(&mut self, name: &str, tracks: Vec<PathBuf>) -> Result<(), Error> {
        let name = name.trim();
        trace!("Adding tracks to playlist \"{name}\"");

        let mut playlist = match self.find_playlist(name) {
            Some(playlist) => playlist.clone(),
            None => return Err(Error::UnknownPlaylist(name.to_string())),
        }
        .as_ref()
        .clone();

        self.playlists.remove(&playlist);
        self.playlists_by_name.remove(&playlist.name);

        let mut out = Vec::with_capacity(tracks.len());
        for path in tracks {
            if let Some(track) = self.find_track_by_path(&path) {
                out.push(track.clone());
            } else {
                return Err(Error::UnknownTrackPath(path.to_path_buf()));
            }
        }

        playlist.tracks.append(&mut out);

        let playlist = Arc::new(playlist);
        self.playlists.insert(playlist.clone());
        self.playlists_by_name
            .insert(playlist.name.clone(), playlist);

        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    pub fn remove_tracks(&mut self, name: &str, tracks: Vec<usize>) -> Result<(), Error> {
        let name = name.trim();
        trace!("Removing tracks from playlist \"{name}\"");

        let mut playlist = match self.find_playlist(name) {
            Some(playlist) => playlist,
            None => return Err(Error::UnknownPlaylist(name.to_string())),
        }
        .as_ref()
        .clone();

        self.playlists.remove(&playlist);
        self.playlists_by_name.remove(&playlist.name);

        playlist.remove_tracks(tracks)?;
        let playlist = Arc::new(playlist);
        self.playlists.insert(playlist.clone());
        self.playlists_by_name
            .insert(playlist.name.clone(), playlist);

        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    // TEST: Check if importing M3U8 files works correctly
    pub fn import_m3u8_playlist(
        &mut self,
        path: &PathBuf,
        name: Option<String>,
    ) -> Result<(), Error> {
        trace!("Importing a playlist from an M3U8 file at {path:?}");
        let tracks = tracks_from_m3u8(path)?;
        let name = match name {
            Some(n) => n,
            None => {
                if let Some(path) = path.file_name() {
                    let path = path.to_string_lossy().to_string();
                    path.strip_suffix(".m3u8")
                        .unwrap_or(path.strip_suffix(".m3u").unwrap_or(&path))
                        .to_string()
                } else {
                    self.get_default_playlist_name()
                }
            }
        };
        self.create_playlist(name, &Some(tracks))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ConfMusicLibrary {
    playlists: Vec<ConfPlaylist>,
}

impl From<MusicLibrary> for ConfMusicLibrary {
    fn from(value: MusicLibrary) -> Self {
        Self {
            playlists: value
                .playlists
                .into_iter()
                .map(|t| t.as_ref().clone().into())
                .collect(),
        }
    }
}

const DEFAULT_PLAYLIST_NAME: &str = "New Playlist";

static LIBRARY_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data = DATA_DIR.read().unwrap().clone().unwrap();
    data.push("playlists");
    data
});

pub(crate) static MUSIC_LIBRARY: RwLock<Option<MusicLibrary>> = RwLock::new(None);

pub(crate) fn unwrap_lib_ref(maybe_lib: Option<&MusicLibrary>) -> Result<&MusicLibrary, Error> {
    match maybe_lib {
        Some(lib) => Ok(lib),
        None => Err(Error::LibraryNotInitialized),
    }
}

pub(crate) fn unwrap_lib_mut(
    maybe_lib: Option<&mut MusicLibrary>,
) -> Result<&mut MusicLibrary, Error> {
    match maybe_lib {
        Some(lib) => Ok(lib),
        None => Err(Error::LibraryNotInitialized),
    }
}

#[cfg(test)]
pub(crate) fn get_library() -> Result<MusicLibrary, Error> {
    let guard = MUSIC_LIBRARY.read().unwrap();
    unwrap_lib_ref(guard.as_ref()).cloned()
}

/// Save the library state to a file.
pub(crate) fn save_library() -> Result<(), Error> {
    trace!("Saving the library state");

    let lib = MUSIC_LIBRARY.read().unwrap().clone();

    if let Some(lib_data) = lib {
        let lib = ConfMusicLibrary::from(lib_data.clone());

        let library_state = LIBRARY_FILE.clone();

        let mut data = Vec::new();
        lib.serialize(&mut Serializer::new(&mut data)).unwrap();

        let mut file = match File::create(library_state) {
            Ok(file) => file,
            Err(e) => {
                error!("Could not open the library state in write-only mode: {e}");
                return Err(Error::StateWriteFailed);
            }
        };

        match file.write_all(&data) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Could not write to the library: {e}");
                Err(Error::StateWriteFailed)
            }
        }
    } else {
        error!("Cannot save the library since it is uninitialized!");
        Err(Error::LibraryNotInitialized)
    }
}

// FIX: This function hangs if if the `MusicLibrary` struct changes (and can't be deserialized)
/// Load the music library from the playlists file.
pub(crate) fn load(config: music_lib::Config) -> Result<(), Error> {
    trace!("Loading the music library");

    let time_start = SystemTime::now();

    let tracks = match indexer::index(config.clone()) {
        Ok(tracks) => tracks,
        Err(e) => {
            crate::send_event(Event::LibraryLoadFailed(e.to_string()));
            return Err(e);
        }
    };

    let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));
    trace!(
        "Finished indexing the music directory in {:.2}s, found {} audio files",
        time_elapsed.as_secs_f64(),
        tracks.len()
    );

    let library_state = LIBRARY_FILE.clone();

    trace!("Loading the library state from {library_state:?}");

    let exists = match library_state.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            error!("Could not check if the library exists: {e}");
            crate::send_event(Event::LibraryLoadFailed(e.to_string()));
            return Err(Error::StateNotReadable);
        }
    };

    if exists {
        trace!("Restoring the library state");

        if !library_state.is_file() {
            error!("The item at {library_state:?} must be a file!");
            crate::send_event(Event::LibraryLoadFailed(format!(
                "The item at {library_state:?} must be a file!"
            )));
            return Err(Error::StateNotAFile);
        }

        let data = match read(library_state) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not read the library: {e}");
                crate::send_event(Event::LibraryLoadFailed(e.to_string()));
                return Err(Error::StateNotReadable);
            }
        };

        let lib = match ConfMusicLibrary::deserialize(&mut Deserializer::from_read_ref(&data)) {
            Ok(lib) => {
                MusicLibrary::load(lib, tracks.clone().into_iter().collect(), config.indexer)
            }
            Err(e) => {
                error!("Could not decode the contents of the library state file: {e}");
                trace!("Creating a new library");
                MusicLibrary::new(tracks.into_iter().collect())
            }
        };

        *MUSIC_LIBRARY.write().unwrap() = Some(lib);
    } else {
        trace!("The library file does not exist, creating a new library");
        *MUSIC_LIBRARY.write().unwrap() = Some(MusicLibrary::new(tracks));
    }
    crate::send_event(Event::LibraryChanged(Box::new(
        MUSIC_LIBRARY.read().unwrap().as_ref().unwrap().clone(),
    )));
    save_library()?;

    let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));
    trace!(
        "Done loading the music library in {:.2}s",
        time_elapsed.as_secs_f64()
    );

    crate::send_event(Event::LoadProgressChanged(Progress::Done));

    Ok(())
}
