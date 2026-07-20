use std::{
    collections::{HashMap, HashSet},
    fs::{File, read},
    hash::{DefaultHasher, Hash, Hasher},
    io::{BufReader, Write},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, RwLock},
    time::{Duration, SystemTime},
};

use lofty::{
    file::{AudioFile, TaggedFile, TaggedFileExt},
    tag::{Accessor, ItemValue, Tag, items::Timestamp},
};
use log::{error, trace};
use lrc_rs::SyncedLyrics;
#[cfg(feature = "mpris")]
use mpris_server::TrackId;
use rmp_serde::{Deserializer, Serializer};
use rodio::Decoder;
use serde::{Deserialize, Serialize};

use crate::{
    Error, Event,
    music_lib::{
        DATA_DIR,
        covers::{CoverError, LoadMode, get_track_cover},
        indexer::{self},
        tracks_from_m3u8,
    },
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
    pub path: PathBuf,
    pub cover_path: Option<PathBuf>,
    pub duration: Duration,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
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
            self.artist.clone().unwrap_or(String::from("Unknown")),
            self.title.clone().unwrap_or(String::from("Unknown")),
        )
    }
}

impl TryFrom<&TaggedFile> for Track {
    type Error = String;
    fn try_from(value: &TaggedFile) -> Result<Self, Self::Error> {
        let tag = match value.primary_tag() {
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

        Ok(Track {
            path: PathBuf::new(),
            cover_path: None,
            duration: value.properties().duration(),
            artist: tag.artist().map(|artist| artist.into()),
            title: tag.title().map(|title| title.into()),
            album: tag.album().map(|album| album.into()),
            genre: tag.genre().map(|genre| genre.into()),
            lyrics,
            comment: tag.comment().map(|comment| comment.into()),
            track: tag.track(),
            track_total: tag.track_total(),
            disc: tag.disk(),
            disc_total: tag.disk_total(),
            date: tag.date(),
        })
    }
}

impl Track {
    /// Set the cover art from cache or extract it from the source file.
    pub fn get_cover(&mut self, tag: &Tag) -> Result<(), CoverError> {
        self.cover_path = Some(get_track_cover(self, tag, &LoadMode::Load)?);
        Ok(())
    }

    /// Extract the cover art from the source file discarding the cache contents.
    pub fn extract_cover(&mut self, tag: &Tag) -> Result<(), CoverError> {
        self.cover_path = Some(get_track_cover(self, tag, &LoadMode::Rebuild)?);
        Ok(())
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

    pub fn hash_tracks(tracks: &Vec<Track>) -> Vec<u64> {
        let mut hashes = Vec::new();
        for track in tracks {
            hashes.push(Self::hash_track(track));
        }
        hashes
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

        MetadataBuilder::default()
            .length(Time::from_nanos(
                self.duration.as_nanos().try_into().unwrap_or(i64::MAX),
            ))
            .url(self.path.to_string_lossy())
            .art_url(
                self.cover_path
                    .clone()
                    .unwrap_or_default()
                    .to_string_lossy(),
            )
            .artist(self.artist.clone())
            .title(self.title.clone().unwrap_or_default())
            .album(self.album.clone().unwrap_or_default())
            .genre(self.genre.clone())
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
            .trackid(self.track_id(position))
            .build()
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
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Arc<Track>>,
}

impl Playlist {
    fn try_from_loaded_playlist(lib: &MusicLibrary, loaded: ConfPlaylist) -> Result<Self, Error> {
        Ok(Self {
            name: loaded.name,
            tracks: lib.tracks_from_hashes(loaded.track_hashes)?,
        })
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
        Self {
            name: value.name,
            track_hashes: value.tracks.iter().map(|t| t.hash_self()).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MusicLibrary {
    pub playlists: HashSet<Arc<Playlist>>,
    pub tracks: HashSet<Arc<Track>>,
    tracks_by_path: HashMap<String, Arc<Track>>,
    tracks_by_hash: HashMap<u64, Arc<Track>>,
    playlists_by_name: HashMap<String, Arc<Playlist>>,
}

impl MusicLibrary {
    fn try_from_loaded_lib(
        loaded: ConfMusicLibrary,
        tracks: HashSet<Track>,
    ) -> Result<Self, Error> {
        let tracks: HashSet<Arc<Track>> = tracks.into_iter().map(Arc::new).collect();
        let playlists: HashSet<Arc<Playlist>> = HashSet::new();

        let mut path_map: HashMap<_, _> = HashMap::with_capacity(tracks.len());
        for t in tracks.iter() {
            path_map.insert(t.path.to_string_lossy().to_string(), t.clone());
        }

        let mut hash_map: HashMap<_, _> = HashMap::with_capacity(tracks.len());
        for t in tracks.iter() {
            hash_map.insert(t.hash_self(), t.clone());
        }

        let mut lib = Self {
            playlists,
            tracks,
            tracks_by_path: path_map,
            tracks_by_hash: hash_map,
            playlists_by_name: HashMap::new(),
        };

        for p in loaded.playlists {
            let playlist = Arc::new(Playlist::try_from_loaded_playlist(&lib, p)?);
            lib.playlists.insert(playlist.clone());
            lib.playlists_by_name
                .insert(playlist.name.clone(), playlist);
        }

        Ok(lib)
    }

    pub fn new_from_tracks(tracks: Vec<Track>) -> Self {
        let tracks: HashSet<Arc<Track>> = tracks.into_iter().map(Arc::new).collect();
        let playlists: HashSet<Arc<Playlist>> = HashSet::new();

        let mut path_map: HashMap<_, _> = HashMap::with_capacity(tracks.len());
        for t in tracks.iter() {
            path_map.insert(t.path.to_string_lossy().to_string(), t.clone());
        }

        let mut hash_map: HashMap<_, _> = HashMap::with_capacity(tracks.len());
        for t in tracks.iter() {
            hash_map.insert(t.hash_self(), t.clone());
        }

        Self {
            playlists,
            tracks,
            tracks_by_path: path_map,
            tracks_by_hash: hash_map,
            playlists_by_name: HashMap::new(),
        }
    }

    pub fn find_playlist(&self, name: &str) -> Option<&Arc<Playlist>> {
        self.playlists_by_name.get(name)
    }

    pub fn tracks_from_hashes(&self, hashes: Vec<u64>) -> Result<Vec<Arc<Track>>, Error> {
        let mut tracks = Vec::with_capacity(hashes.len());
        for hash in hashes {
            match self.tracks_by_hash.get(&hash) {
                Some(track) => tracks.push(track.clone()),
                None => return Err(Error::UnknownTrack),
            }
        }

        Ok(tracks)
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
                return Err(Error::UnknownPlaylist);
            }
        }
        Ok(())
    }

    pub fn find_track_by_path(&self, path: &Path) -> Option<Arc<Track>> {
        self.tracks_by_path
            .get(&path.to_string_lossy().to_string())
            .cloned()
    }

    pub fn create_playlist(
        &mut self,
        name: String,
        track_paths: &Option<Vec<PathBuf>>,
    ) -> Result<(), Error> {
        trace!("Creating a new playlist \"{name}\" from a list of tracks");

        if self.find_playlist(&name).is_some() {
            error!("A playlist with name \"{name}\" already exists");
            return Err(Error::PlaylistExists);
        }

        let tracks = if let Some(tracks) = track_paths {
            let mut out = Vec::with_capacity(tracks.len());
            for path in tracks {
                if let Some(track) = self.find_track_by_path(path) {
                    out.push(track);
                } else {
                    error!("The track {path:?} was not found in the music library");
                    return Err(Error::UnknownTrack);
                }
            }
            out
        } else {
            Vec::new()
        };

        let playlist = Arc::new(Playlist {
            name: name.clone(),
            tracks,
        });
        self.playlists.insert(playlist.clone());
        self.playlists_by_name.insert(name, playlist);
        crate::send_event(Event::LibraryChanged(Box::new(self.clone())));
        Ok(())
    }

    pub fn add_tracks(&mut self, name: &str, tracks: Vec<PathBuf>) -> Result<(), Error> {
        trace!("Adding tracks to playlist \"{name}\"");

        let mut playlist = match self.find_playlist(name) {
            Some(playlist) => playlist.clone(),
            None => return Err(Error::UnknownPlaylist),
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
                return Err(Error::UnknownTrack);
            }
        }

        playlist.tracks.append(&mut out);

        let playlist = Arc::new(playlist);
        self.playlists.insert(playlist.clone());
        self.playlists_by_name
            .insert(playlist.name.clone(), playlist);

        Ok(())
    }

    pub fn remove_tracks(&mut self, name: &str, tracks: Vec<usize>) -> Result<(), Error> {
        trace!("Removing tracks from playlist \"{name}\"");

        let mut playlist = match self.find_playlist(name) {
            Some(playlist) => playlist,
            None => return Err(Error::UnknownPlaylist),
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

        Ok(())
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
                    path.strip_suffix(".m3u8").unwrap_or(&path).to_string()
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
pub(crate) fn load(load_mode: LoadMode) -> Result<(), Error> {
    trace!("Loading the music library");

    let time_start = SystemTime::now();

    let tracks = match indexer::index(load_mode) {
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
            Ok(data) => {
                match MusicLibrary::try_from_loaded_lib(data, tracks.clone().into_iter().collect())
                {
                    Ok(lib) => lib,
                    Err(e) => {
                        error!("Could not open the music library: {e}");
                        MusicLibrary::new_from_tracks(tracks.into_iter().collect())
                    }
                }
            }
            Err(e) => {
                error!("Could not decode the contents of the library state file: {e}");
                trace!("Creating a new library");
                MusicLibrary::new_from_tracks(tracks.into_iter().collect())
            }
        };

        *MUSIC_LIBRARY.write().unwrap() = Some(lib);
    } else {
        trace!("The library file does not exist, creating a new library");
        *MUSIC_LIBRARY.write().unwrap() = Some(MusicLibrary::new_from_tracks(tracks));
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

    Ok(())
}
