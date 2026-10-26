#[cfg(test)]
mod tests;

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use icu::collator::CollatorBorrowed;
pub use lofty::tag::items::Timestamp;
use log::{error, trace};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::music_lib::indexer::CacheMode;
use crate::{
    COLLATOR, Error, Event, get_config,
    music_lib::{
        indexer::{
            self,
            covers::{Cover, get_playlist_cover},
        },
        state::{Playlist, PlaylistStored, Progress, Track},
        tracks_from_m3u8,
    },
};

const DEFAULT_PLAYLIST_NAME: &str = "New Playlist";

#[derive(Clone, Debug, PartialEq)]
pub struct HashMatchingResult {
    pub matched: Vec<Arc<Track>>,
    pub unmatched: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Album {
    pub title: Box<str>,
    pub cover: Cover,
    pub artists: Box<[Box<str>]>,
    pub tracks: Box<[Arc<Track>]>,
    pub date: Option<Timestamp>,
    pub duration: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Artist {
    pub name: Box<str>,
    pub cover: Cover,
    pub tracks: Box<[Arc<Track>]>,
    pub albums: Box<[Arc<Album>]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Genre {
    pub name: Box<str>,
    pub cover: Cover,
    pub artists: Box<[Arc<Artist>]>,
    pub albums: Box<[Arc<Album>]>,
    pub tracks: Box<[Arc<Track>]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LibraryItem {
    Track(Arc<Track>),
    Playlist(Arc<Playlist>),
    Album(Arc<Album>),
    Artist(Arc<Artist>),
    Genre(Arc<Genre>),
}

impl From<Arc<Track>> for LibraryItem {
    fn from(value: Arc<Track>) -> Self {
        Self::Track(value)
    }
}

impl From<Arc<Playlist>> for LibraryItem {
    fn from(value: Arc<Playlist>) -> Self {
        Self::Playlist(value)
    }
}

impl From<Arc<Album>> for LibraryItem {
    fn from(value: Arc<Album>) -> Self {
        Self::Album(value)
    }
}

impl From<Arc<Artist>> for LibraryItem {
    fn from(value: Arc<Artist>) -> Self {
        Self::Artist(value)
    }
}

impl From<Arc<Genre>> for LibraryItem {
    fn from(value: Arc<Genre>) -> Self {
        Self::Genre(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MusicLibrary {
    pub playlists: Vec<Arc<Playlist>>,
    pub favorites: HashSet<LibraryItem>,
    pub tracks: Vec<Arc<Track>>,
    pub albums: Vec<Arc<Album>>,
    pub artists: Vec<Arc<Artist>>,
    pub genres: Vec<Arc<Genre>>,
    pub total_duration: Duration,
    playlist_id_counter: u64,
    tracks_by_path: HashMap<Box<str>, Arc<Track>>,
    tracks_by_hash: HashMap<u64, Arc<Track>>,
    playlists_by_name: HashMap<Box<str>, Arc<Playlist>>,
    playlists_by_id: HashMap<u64, Arc<Playlist>>,
    artists_by_name: HashMap<Box<str>, Arc<Artist>>,
    albums_by_title: HashMap<Box<str>, Arc<Album>>,
    genres_by_name: HashMap<Box<str>, Arc<Genre>>,
}

// TODO: Fallback alphabetic sorting w/out the collator
impl MusicLibrary {
    fn sort_tracks_alphabetically(
        tracks: &mut [Arc<Track>],
        collator: Option<&Arc<CollatorBorrowed<'_>>>,
    ) {
        if let Some(collator) = collator {
            tracks.sort_by(|t1, t2| {
                collator.compare(
                    t1.title.as_deref().unwrap_or(""),
                    t2.title.as_deref().unwrap_or(""),
                )
            });
        }
    }

    fn sort_tracks_chronologically(
        tracks: &mut [Arc<Track>],
        collator: Option<&Arc<CollatorBorrowed<'_>>>,
    ) {
        tracks.sort_by(|a, b| match (a.date, b.date) {
            (Some(ad), Some(bd)) => bd.cmp(&ad),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                if let Some(collator) = collator {
                    collator.compare(
                        a.title.as_deref().unwrap_or(""),
                        b.title.as_deref().unwrap_or(""),
                    )
                } else {
                    std::cmp::Ordering::Equal
                }
            }
        });
    }

    fn sort_tracks_in_album(
        tracks: &mut [Arc<Track>],
        collator: Option<&Arc<CollatorBorrowed<'_>>>,
    ) {
        tracks.sort_by(|a, b| match (a.track, b.track) {
            (Some(at), Some(bt)) => at.cmp(&bt),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                if let Some(c) = collator {
                    c.compare(
                        a.title.as_deref().unwrap_or(""),
                        b.title.as_deref().unwrap_or(""),
                    )
                } else {
                    std::cmp::Ordering::Equal
                }
            }
        });
    }

    fn sort_albums_alphabetically(
        albums: &mut [Arc<Album>],
        collator: Option<&Arc<CollatorBorrowed<'_>>>,
    ) {
        if let Some(collator) = collator {
            albums.sort_by(|a1, a2| collator.compare(&a1.title, &a2.title));
        }
    }

    fn sort_albums_chronologically(
        albums: &mut [Arc<Album>],
        collator: Option<&Arc<CollatorBorrowed<'_>>>,
    ) {
        albums.sort_by(|a, b| match (a.date, b.date) {
            (Some(ad), Some(bd)) => bd.cmp(&ad),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                if let Some(c) = collator {
                    c.compare(&a.title, &b.title)
                } else {
                    std::cmp::Ordering::Equal
                }
            }
        });
    }

    fn sort_artists(artists: &mut [Arc<Artist>], collator: Option<&Arc<CollatorBorrowed<'_>>>) {
        if let Some(collator) = collator {
            artists.sort_by(|a1, a2| collator.compare(&a1.name, &a2.name));
        }
    }

    fn sort_playlists(&mut self) {
        let guard = COLLATOR.read().unwrap();
        if let Some(collator) = guard.as_ref() {
            self.playlists
                .sort_by(|p1, p2| collator.compare(&p1.name, &p2.name));
        }
    }

    pub(super) fn new(tracks: Vec<Track>) -> Self {
        crate::send_event(Event::LoadProgressChanged(Progress::RebuildingLibrary));

        let mut tracks: Vec<_> = tracks.into_iter().map(Arc::new).collect();

        let guard = COLLATOR.read().unwrap();
        let collator = guard.as_ref();
        Self::sort_tracks_alphabetically(&mut tracks, collator);

        let album_titles: HashSet<_> = tracks.iter().flat_map(|t| &t.album).collect();
        let artist_names: HashSet<_> = tracks
            .iter()
            .flat_map(|t| t.artists.as_ref().into_iter().flatten())
            .collect();
        let genre_names: HashSet<_> = tracks
            .iter()
            .flat_map(|t| t.genres.as_ref().into_iter().flatten())
            .collect();

        let mut tracks_by_artist: HashMap<&Box<str>, HashSet<Arc<Track>>> =
            HashMap::with_capacity(artist_names.len());
        let mut tracks_by_album: HashMap<&Box<str>, HashSet<Arc<Track>>> =
            HashMap::with_capacity(album_titles.len());
        let mut tracks_by_genre: HashMap<&Box<str>, HashSet<Arc<Track>>> =
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
                Self::sort_tracks_in_album(&mut tracks, collator);

                let mut artists: Vec<_> = tracks
                    .iter()
                    .flat_map(|t| t.artists.clone().into_iter().flatten())
                    .collect::<HashSet<Box<str>>>()
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
                    title: title.to_string().into(),
                    cover,
                    tracks: tracks.into(),
                    artists: artists.into(),
                    date,
                    duration,
                })
            })
            .collect();
        Self::sort_albums_alphabetically(&mut albums, collator);

        let albums_by_title: HashMap<Box<str>, Arc<Album>> = albums
            .iter()
            .map(|a| (a.title.clone(), a.clone()))
            .collect();

        let mut albums_by_artist: HashMap<&Box<str>, HashSet<Arc<Album>>> =
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
                // TODO: Sort tracks chronologically if possible, then fall back to track index in
                // albums, then to alphabetic sorting
                Self::sort_tracks_chronologically(&mut tracks, collator);

                let mut albums = if let Some(albums) = albums_by_artist.get(name) {
                    albums.iter().cloned().collect()
                } else {
                    Vec::new()
                };
                Self::sort_albums_chronologically(&mut albums, collator);

                let cover = tracks
                    .first()
                    .map(|t| t.cover.clone())
                    .unwrap_or(Cover::none());

                Arc::new(Artist {
                    name: name.to_string().into(),
                    cover,
                    tracks: tracks.into(),
                    albums: albums.into(),
                })
            })
            .collect();
        Self::sort_artists(&mut artists, collator);

        let mut artists_by_album: HashMap<&Box<str>, HashSet<Arc<Artist>>> =
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

        let mut albums_by_genre: HashMap<&Box<str>, HashSet<Arc<Album>>> =
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

        let mut artists_by_genre: HashMap<&Box<str>, HashSet<Arc<Artist>>> =
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
                Self::sort_tracks_alphabetically(&mut tracks, collator);

                let mut counts: HashMap<Cover, usize> = HashMap::with_capacity(tracks.len());
                for track in &tracks {
                    *counts.entry(track.cover.clone()).or_insert(0) += 1;
                }
                let commonest = counts.into_iter().max_by_key(|(_, c)| *c).map(|(k, _)| k);
                let cover = commonest.unwrap_or(tracks[0].cover.clone());

                let mut artists: Vec<_> = artists_by_genre[name].clone().into_iter().collect();
                Self::sort_artists(&mut artists, collator);

                let mut albums: Vec<_> = albums_by_genre[name].clone().into_iter().collect();
                Self::sort_albums_alphabetically(&mut albums, collator);

                Arc::new(Genre {
                    name: name.to_string().into(),
                    cover,
                    tracks: tracks.into(),
                    albums: albums.into(),
                    artists: artists.into(),
                })
            })
            .collect();

        let genres_by_name: HashMap<Box<str>, Arc<Genre>> =
            genres.iter().map(|g| (g.name.clone(), g.clone())).collect();

        if let Some(collator) = collator {
            genres.sort_by(|g1, g2| collator.compare(&g1.name, &g2.name));
        }

        let mut track_path_map: HashMap<Box<str>, _> = HashMap::with_capacity(tracks.len());
        for t in tracks.iter() {
            track_path_map.insert(t.path.to_string_lossy().to_string().into(), t.clone());
        }

        let mut track_hash_map: HashMap<_, _> = HashMap::with_capacity(tracks.len());
        for t in tracks.iter() {
            track_hash_map.insert(t.hash_self(), t.clone());
        }

        let artists_by_name: HashMap<Box<str>, Arc<Artist>> = artists
            .iter()
            .map(|a| (a.name.clone(), a.clone()))
            .collect();

        let total_duration = tracks.iter().map(|t| t.duration).sum();

        Self {
            playlists: Vec::new(),
            favorites: HashSet::new(),
            tracks,
            albums,
            artists,
            genres,
            playlist_id_counter: 0,
            tracks_by_path: track_path_map,
            tracks_by_hash: track_hash_map,
            playlists_by_name: HashMap::new(),
            playlists_by_id: HashMap::new(),
            artists_by_name,
            albums_by_title,
            genres_by_name,
            total_duration,
        }
    }

    pub(super) fn load(
        loaded: MusicLibraryStored,
        tracks: Vec<Track>,
        config: indexer::Config,
    ) -> Self {
        crate::send_event(Event::LoadProgressChanged(Progress::RestoringState));

        let mut lib = Self::new(tracks);

        let mut playlists = Vec::with_capacity(loaded.playlists.len());
        lib.playlists_by_name = HashMap::with_capacity(loaded.playlists.len());
        lib.playlists_by_id = HashMap::with_capacity(loaded.playlists.len());
        for p in loaded.playlists {
            let playlist = Arc::new(Playlist::load(&mut lib, p, config));
            playlists.push(playlist.clone());
            lib.playlists_by_name
                .insert(playlist.name.clone(), playlist.clone());
            lib.playlists_by_id.insert(playlist.id, playlist);
        }

        let favorites = loaded
            .favorites
            .into_iter()
            .flat_map(|favorite| match favorite {
                LibraryItemStored::Track(hash) => {
                    lib.find_track_by_hash(hash).cloned().map(|t| t.into())
                }
                LibraryItemStored::Playlist(name) => {
                    lib.find_playlist_by_name(&name).cloned().map(|p| p.into())
                }
                LibraryItemStored::Album(title) => {
                    lib.find_album(&title).cloned().map(|a| a.into())
                }
                LibraryItemStored::Artist(name) => {
                    lib.find_artist(&name).cloned().map(|a| a.into())
                }
                LibraryItemStored::Genre(name) => lib.find_genre(&name).cloned().map(|g| g.into()),
            })
            .collect();

        lib.favorites = favorites;
        lib.playlists = playlists;
        lib.sort_playlists();

        lib
    }

    #[cfg(test)]
    pub(crate) fn new_testing(tracks: Vec<Track>) -> Self {
        Self::new(tracks)
    }

    pub(super) fn get_playlist_id(&mut self) -> u64 {
        self.playlist_id_counter += 1;
        self.playlist_id_counter
    }

    fn check_name(&self, name: &str) -> Result<(), Error> {
        let name = name.trim();
        if name.is_empty() {
            return Err(Error::EmptyName);
        }
        if self.find_playlist_by_name(name).is_some() {
            error!("A playlist with name \"{name}\" already exists");
            return Err(Error::PlaylistExists);
        }
        Ok(())
    }

    pub fn find_track_by_path(&self, path: &str) -> Option<Arc<Track>> {
        self.tracks_by_path.get(path).cloned()
    }

    pub fn find_artist(&self, name: &str) -> Option<&Arc<Artist>> {
        self.artists_by_name.get(name)
    }

    pub fn find_playlist_by_name(&self, name: &str) -> Option<&Arc<Playlist>> {
        self.playlists_by_name.get(name)
    }

    pub fn find_playlist_by_id(&self, id: u64) -> Option<&Arc<Playlist>> {
        self.playlists_by_id.get(&id)
    }

    pub fn find_album(&self, title: &str) -> Option<&Arc<Album>> {
        self.albums_by_title.get(title)
    }

    pub fn find_genre(&self, name: &str) -> Option<&Arc<Genre>> {
        self.genres_by_name.get(name)
    }

    /// Returns the default playlist name ("New Playlist").
    ///
    /// If a playlist with the default name exists, then a number will be added to the end of the
    /// playlist name so it's unique, eg. "New Playlist 1", "New Playlist 2", etc.
    pub fn get_default_playlist_name(&self) -> String {
        let mut i = 0;
        let mut playlist_name = DEFAULT_PLAYLIST_NAME.to_string();
        while self.find_playlist_by_name(&playlist_name).is_some() {
            i += 1;
            playlist_name = format!("{DEFAULT_PLAYLIST_NAME} {i}");
        }
        playlist_name
    }

    /// Retrieves a track corresponding to the provided hash.
    pub fn find_track_by_hash(&self, hash: u64) -> Option<&Arc<Track>> {
        self.tracks_by_hash.get(&hash)
    }

    /// Retrieves tracks corresponding to a list of provided hashes.
    pub fn find_tracks_by_hashes(&self, hashes: Vec<u64>) -> HashMatchingResult {
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

    pub fn remove_playlists(&mut self, mut playlists: Vec<&str>) -> Result<(), Error> {
        playlists.sort();
        let mut unique = playlists.clone();
        unique.dedup();
        if unique != playlists {
            return Err(Error::DuplicateItems);
        }

        for name in &playlists {
            if let Some(index) = self.playlists.iter().position(|p| p.name.as_ref() == *name) {
                let id = self.playlists[index].id;
                self.playlists.remove(index);
                self.playlists_by_name.remove(*name);
                self.playlists_by_id.remove(&id);
            } else {
                return Err(Error::UnknownPlaylist(name.to_string()));
            }
        }

        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    pub fn create_playlist(
        &mut self,
        name: &str,
        track_paths: &Option<Vec<PathBuf>>,
    ) -> Result<(), Error> {
        trace!("Creating a new playlist \"{name}\" from a list of tracks");

        let name = name.trim();
        self.check_name(name)?;

        let tracks = if let Some(tracks) = track_paths {
            let mut out = Vec::with_capacity(tracks.len());
            for path in tracks {
                if let Some(track) = self.find_track_by_path(&path.to_string_lossy()) {
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
            name: name.to_string().into_boxed_str(),
            tracks: tracks.into(),
            duration,
            unmatched: Box::new([]),
            id: self.get_playlist_id(),
            cover,
        });
        self.playlists.push(playlist.clone());
        self.playlists_by_name.insert(name.into(), playlist);
        self.sort_playlists();
        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    pub fn rename_playlist(&mut self, source: &str, target: &str) -> Result<(), Error> {
        let source = source.trim();
        let target = target.trim();
        trace!("Renaming playlist \"{source}\" to \"{target}\"");
        self.check_name(target)?;

        let src_playlist_index = match self
            .playlists
            .iter()
            .position(|p| p.name.as_ref() == source)
        {
            Some(pos) => pos,
            None => return Err(Error::UnknownPlaylist(source.to_string())),
        };
        let src_playlist = self.playlists[src_playlist_index].clone();
        let playlist_id = src_playlist.id;
        let mut playlist = src_playlist.as_ref().clone();
        playlist.name = target.to_string().into();

        self.playlists.remove(src_playlist_index);
        self.playlists_by_name.remove(source);
        self.playlists_by_id.remove(&playlist_id);
        let playlist = Arc::new(playlist);
        self.playlists.push(playlist.clone());
        self.playlists_by_name
            .insert(target.into(), playlist.clone());
        self.playlists_by_id.insert(playlist_id, playlist);
        self.sort_playlists();

        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    pub fn add_tracks(&mut self, name: &str, tracks: Vec<u64>) -> Result<(), Error> {
        let name = name.trim();
        trace!("Adding tracks to playlist \"{name}\"");

        let playlist_index = match self.playlists.iter().position(|p| p.name.as_ref() == name) {
            Some(i) => i,
            None => return Err(Error::UnknownPlaylist(name.to_string())),
        };

        let mut result = self.find_tracks_by_hashes(tracks);
        if let Some(unmatched) = result.unmatched.first() {
            return Err(Error::UnknownTrack { hash: *unmatched });
        }

        let mut playlist = self.playlists[playlist_index].as_ref().clone();
        self.playlists.remove(playlist_index);
        self.playlists_by_name.remove(&playlist.name);
        self.playlists_by_id.remove(&playlist.id);

        let mut tracks = std::mem::take(&mut playlist.tracks).into_vec();
        tracks.append(&mut result.matched);
        playlist.tracks = tracks.into_boxed_slice();

        let playlist = Arc::new(playlist);
        self.playlists.push(playlist.clone());
        self.playlists_by_name
            .insert(playlist.name.clone(), playlist.clone());
        self.playlists_by_id.insert(playlist.id, playlist);
        self.sort_playlists();

        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    pub fn remove_tracks(&mut self, name: &str, tracks: Vec<usize>) -> Result<(), Error> {
        let name = name.trim();
        trace!("Removing tracks from playlist \"{name}\"");

        let playlist_index = match self.playlists.iter().position(|p| p.name.as_ref() == name) {
            Some(i) => i,
            None => return Err(Error::UnknownPlaylist(name.to_string())),
        };
        let mut playlist = self.playlists[playlist_index].as_ref().clone();

        self.playlists.remove(playlist_index);
        self.playlists_by_name.remove(&playlist.name);
        self.playlists_by_id.remove(&playlist.id);

        playlist.remove_tracks(tracks)?;
        let playlist = Arc::new(playlist);
        self.playlists.push(playlist.clone());
        self.playlists_by_name
            .insert(playlist.name.clone(), playlist.clone());
        self.playlists_by_id.insert(playlist.id, playlist);
        self.sort_playlists();

        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    // TODO: Partial importing when there are missing tracks (also should be reflected in the error
    // type) and complete failure when the playlist is unreadable
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
        self.create_playlist(&name, &Some(tracks))
    }

    /// Adds a new favorite item to the library.
    ///
    /// Returns whether the item was already marked as a favorite.
    pub fn add_favorite(&mut self, favorite: LibraryItem) -> Result<(), Error> {
        self.favorites.insert(favorite);
        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    /// Removes an item from the favorites list.
    ///
    /// Returns whether the item was marked as a favorite.
    pub fn remove_favorite(&mut self, favorite: &LibraryItem) -> Result<(), Error> {
        self.favorites.remove(favorite);
        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub(super) enum LibraryItemStored {
    Track(u64),
    Playlist(String),
    Album(String),
    Artist(String),
    Genre(String),
}

impl From<LibraryItem> for LibraryItemStored {
    fn from(value: LibraryItem) -> Self {
        match value {
            LibraryItem::Track(track) => Self::Track(track.hash_self()),
            LibraryItem::Playlist(playlist) => Self::Playlist(playlist.name.clone().into_string()),
            LibraryItem::Album(album) => Self::Album(album.title.clone().into_string()),
            LibraryItem::Artist(artist) => Self::Artist(artist.name.clone().into_string()),
            LibraryItem::Genre(genre) => Self::Genre(genre.name.clone().into_string()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct MusicLibraryStored {
    playlists: Vec<PlaylistStored>,
    favorites: Vec<LibraryItemStored>,
}

impl From<MusicLibrary> for MusicLibraryStored {
    fn from(value: MusicLibrary) -> Self {
        Self {
            playlists: value
                .playlists
                .into_iter()
                .map(|t| t.as_ref().clone().into())
                .collect(),
            favorites: value.favorites.into_iter().map(|i| i.into()).collect(),
        }
    }
}
