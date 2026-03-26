use std::{
    collections::HashSet,
    fs::{File, read},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
    sync::{LazyLock, RwLock},
    thread,
    time::{Duration, SystemTime},
};

use lofty::tag::{Accessor, ItemValue, Tag};
use log::{error, trace};
use mpipc::{DataError, MusicLibraryError};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use crate::{
    data::{
        DATA_DIR,
        cache::{
            covers::get_track_cover,
            indexer::{index, index_files},
        },
    },
    playback, send_event,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ConfPlaylist {
    pub name: String,
    pub track_hashes: Vec<u64>,
}

impl From<Playlist> for ConfPlaylist {
    fn from(value: Playlist) -> Self {
        Self {
            name: value.name,
            track_hashes: Track::hash_tracks(&value.tracks),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ConfMusicLibrary {
    pub playlists: Vec<ConfPlaylist>,
}

impl From<MusicLibrary> for ConfMusicLibrary {
    fn from(value: MusicLibrary) -> Self {
        let mut playlists = Vec::new();
        for playlist in value.playlists {
            playlists.push(playlist.into());
        }
        Self { playlists }
    }
}

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

impl From<mpipc::Track> for Track {
    fn from(value: mpipc::Track) -> Self {
        Track {
            path: value.path,
            cover_path: value.cover_path,
            artist: value.artist,
            title: value.title,
            album: value.album,
            genre: value.genre,
            comment: value.comment,
            track: value.track,
            track_total: value.track_total,
            disk: value.disk,
            disk_total: value.disk_total,
            year: value.year,
            lyrics: value.lyrics,
        }
    }
}

impl From<Track> for mpipc::Track {
    fn from(value: Track) -> Self {
        mpipc::Track {
            path: value.path,
            cover_path: value.cover_path,
            artist: value.artist,
            title: value.title,
            album: value.album,
            genre: value.genre,
            comment: value.comment,
            track: value.track,
            track_total: value.track_total,
            disk: value.disk,
            disk_total: value.disk_total,
            year: value.year,
            lyrics: value.lyrics,
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
    pub fn get_cover(&mut self, tag: &Tag) -> Result<(), DataError> {
        get_track_cover(self, tag, false)
    }

    pub fn extract_cover(&mut self, tag: &Tag) -> Result<(), DataError> {
        get_track_cover(self, tag, true)
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
}

#[derive(Clone, Debug)]
pub(crate) struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
}

impl From<mpipc::Playlist> for Playlist {
    fn from(value: mpipc::Playlist) -> Self {
        let mut tracks = Vec::new();
        for track in value.tracks {
            tracks.push(track.into());
        }

        Playlist {
            name: value.name,
            tracks,
        }
    }
}

impl From<Playlist> for mpipc::Playlist {
    fn from(value: Playlist) -> Self {
        let mut tracks = Vec::new();
        for track in value.tracks {
            tracks.push(track.into());
        }
        mpipc::Playlist {
            name: value.name,
            tracks,
        }
    }
}

impl Playlist {
    fn from_loaded_playlist(loaded: ConfPlaylist, tracks: &[Track]) -> Self {
        let tracks = tracks_from_hashes(loaded.track_hashes, tracks);

        Self {
            name: loaded.name,
            tracks,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct MusicLibrary {
    pub playlists: Vec<Playlist>,
    pub tracks: Vec<Track>,
}

impl From<mpipc::MusicLibrary> for MusicLibrary {
    fn from(value: mpipc::MusicLibrary) -> Self {
        let mut playlists = Vec::new();
        let mut tracks = Vec::new();
        for playlist in value.playlists {
            playlists.push(playlist.into());
        }
        for track in value.tracks {
            tracks.push(track.into());
        }

        MusicLibrary { playlists, tracks }
    }
}

impl From<MusicLibrary> for mpipc::MusicLibrary {
    fn from(value: MusicLibrary) -> Self {
        let mut playlists = Vec::new();
        let mut tracks = Vec::new();
        for playlist in value.playlists {
            playlists.push(playlist.into());
        }
        for track in value.tracks {
            tracks.push(track.into());
        }
        mpipc::MusicLibrary { playlists, tracks }
    }
}

impl MusicLibrary {
    fn from_loaded_lib(loaded: ConfMusicLibrary, tracks: Vec<Track>) -> Self {
        let mut playlists = Vec::new();
        for playlist in loaded.playlists {
            playlists.push(Playlist::from_loaded_playlist(playlist, &tracks));
        }
        Self { playlists, tracks }
    }

    pub fn get_playlist_with_name(&'_ self, name: &str) -> Option<&'_ Playlist> {
        self.playlists
            .iter()
            .find(|&playlist| playlist.name == name)
    }
}

#[derive(Debug, PartialEq, Eq)]
/// The mode in which to load the music library.
pub enum LoadMode {
    /// Initialize the library.
    Initialize,
    /// Reinitialize the library.
    Reinitialize,
    /// Reinitialize the library and rebuild the cover cache.
    Rebuild,
}

impl LoadMode {
    /// Creates library load mode from the command line cache command.
    pub fn from_cache_command(cmd: mpipc::CacheCommand) -> Self {
        match cmd {
            mpipc::CacheCommand::Reload => Self::Reinitialize,
            mpipc::CacheCommand::Rebuild => Self::Rebuild,
        }
    }
}

static LIBRARY_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data = DATA_DIR.read().unwrap().clone().unwrap();
    data.push("playlists");
    data
});

static MUSIC_LIBRARY: RwLock<Option<MusicLibrary>> = RwLock::new(None);

pub(crate) fn tracks_from_hashes(track_hashes: Vec<u64>, tracks: &[Track]) -> Vec<Track> {
    let wanted: HashSet<u64> = track_hashes.into_iter().collect();
    tracks
        .iter()
        .filter_map(|track| {
            let h = Track::hash_track(track);
            if wanted.contains(&h) {
                Some(track.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Save the library state to a file.
pub(crate) fn save_library() -> Result<(), MusicLibraryError> {
    trace!("Saving the library state to library cache");

    let lib = MUSIC_LIBRARY.read().unwrap().clone();

    if let Some(lib_data) = lib {
        let lib = ConfMusicLibrary::from(lib_data.clone());

        let library_cache = LIBRARY_FILE.clone();

        let mut data = Vec::new();
        if let Err(e) = lib.serialize(&mut Serializer::new(&mut data)) {
            error!("Could not serialize the music library: {e}");
            return Err(MusicLibraryError::CacheError);
        }

        let mut file = match File::create(library_cache) {
            Ok(file) => file,
            Err(e) => {
                error!("Could not open the library cache in write-only mode: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        match file.write_all(&data) {
            Ok(_) => {
                if let Err(e) = send_event(mpipc::DaemonEvent::MusicLibraryChanged(lib_data.into()))
                {
                    error!("Could not emit the library changed event: {e}");
                }
                Ok(())
            }
            Err(e) => {
                error!("Could not write to the library cache: {e}");
                Err(MusicLibraryError::CacheError)
            }
        }
    } else {
        error!("Cannot save the library since it is uninitialized!");
        Err(MusicLibraryError::CacheError)
    }
}

/// Load the music library from the playlists file.
pub(crate) fn load(mode: LoadMode) -> Result<(), MusicLibraryError> {
    trace!("Loading the music library");

    // This should never occur, `LoadMode::Initialize` shall only be passed to this function on
    // startup.
    if mode == LoadMode::Initialize && MUSIC_LIBRARY.read().unwrap().is_some() {
        error!("Cannot load the music library, it is already initialized!");
        return Err(MusicLibraryError::CacheError);
    }

    let time_start = SystemTime::now();

    let rebuild_covers = mode == LoadMode::Rebuild;
    // No need to specify the music directory, it is detected by the indexer
    let tracks = match index(None, rebuild_covers) {
        Ok(tracks) => tracks,
        Err(e) => {
            return Err(e);
        }
    };

    let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));
    trace!(
        "Finished indexing the music directory in {:.2}s, found {} audio files",
        time_elapsed.as_secs_f64(),
        tracks.len()
    );

    let library_cache = LIBRARY_FILE.clone();

    trace!("Loading the library cache from {library_cache:?}");

    let exists = match library_cache.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            error!("Could not check if the library cache exists: {e}");
            return Err(MusicLibraryError::CacheError);
        }
    };

    if exists {
        trace!("Library cache exists, restoring library state");

        if library_cache.is_dir() {
            error!("The library cache at {library_cache:?} must not be a directory!");
            return Err(MusicLibraryError::CacheError);
        }

        let data = match read(library_cache) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not read the library cache: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        let lib_conf = match ConfMusicLibrary::deserialize(&mut Deserializer::from_read_ref(&data))
        {
            Ok(data) => data,
            Err(e) => {
                error!("Could not decode the contents of the library cache: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        let lib = MusicLibrary::from_loaded_lib(lib_conf, tracks);
        let mut guard = MUSIC_LIBRARY.write().unwrap();
        *guard = Some(lib);
        drop(guard);
    } else {
        trace!("The library cache does not exist, creating a new library");
        let mut guard = MUSIC_LIBRARY.write().unwrap();
        *guard = Some(MusicLibrary::default());
        drop(guard);
        save_library()?;
    }

    let time_elapsed = time_start.elapsed().unwrap_or(Duration::from_secs(0));
    trace!(
        "Done loading the music library in {:.2}s",
        time_elapsed.as_secs_f64()
    );

    thread::spawn(playback::init);
    Ok(())
}

pub(crate) fn get_library() -> Result<MusicLibrary, MusicLibraryError> {
    if let Some(lib) = MUSIC_LIBRARY.read().unwrap().clone() {
        Ok(lib)
    } else {
        error!("Tried to get the music library, but it was uninitialized!");
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}

pub fn create_playlist(
    name: String,
    tracks: &Option<Vec<PathBuf>>,
) -> Result<(), MusicLibraryError> {
    trace!("Creating a new playlist with name \"{name}\" from a list of tracks");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = guard.as_mut() {
        if lib.get_playlist_with_name(&name).is_none() {
            let tracks = if let Some(tracks) = tracks {
                match index_files(tracks.to_vec(), false) {
                    Ok(tracks) => tracks,
                    Err(e) => {
                        error!("Got an error while indexing the provided files: {e}");
                        return Err(e);
                    }
                }
            } else {
                Vec::new()
            };

            let lib_set: HashSet<_> = lib.tracks.iter().collect();
            let intersecting_tracks: Vec<Track> = tracks
                .iter()
                .filter(|t| lib_set.contains(t))
                .cloned()
                .collect();

            lib.playlists.push(Playlist {
                name: name.clone(),
                tracks: intersecting_tracks,
            });

            drop(guard);
            save_library()?;
            trace!("Created a new playlist with the name \"{name}\"");
            Ok(())
        } else {
            error!("A playlist with the name \"{name}\" already exists");
            Err(MusicLibraryError::PlaylistExists)
        }
    } else {
        error!("Cannot modify an uninitialized library");
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}

pub fn import_playlist_from_m3u8(
    name: Option<String>,
    m3u8_file: &Path,
) -> Result<(), MusicLibraryError> {
    let name = {
        if let Some(name) = name {
            name
        } else {
            match m3u8_file.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => {
                    error!(
                        "Could not determine the name for the imported playlist, the M3U8 file path had no final component!"
                    );
                    return Err(MusicLibraryError::CacheError);
                }
            }
        }
    };
    let lib = &*MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = lib {
        if lib.get_playlist_with_name(&name).is_none() {
            todo!()
        } else {
            Err(MusicLibraryError::PlaylistExists)
        }
    } else {
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}

pub fn delete_playlist(name: &str, save_state: bool) -> Result<(), MusicLibraryError> {
    trace!("Deleting playlist with name \"{name}\"");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = guard.as_mut() {
        match lib
            .playlists
            .iter()
            .position(|playlist| playlist.name == name)
        {
            Some(pos) => {
                lib.playlists.remove(pos);
                if save_state {
                    drop(guard);
                    save_library()?;
                }
                trace!("Deleted playlist with name \"{name}\"");
                Ok(())
            }
            None => Err(MusicLibraryError::NoSuchPlaylist),
        }
    } else {
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}

pub fn add_tracks(name: &str, tracks: Vec<PathBuf>) -> Result<(), MusicLibraryError> {
    trace!("Appending tracks to playlist \"{name}\"");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = guard.as_mut() {
        let playlist = match lib
            .playlists
            .iter()
            .position(|playlist| playlist.name == name)
        {
            Some(pos) => &mut lib.playlists[pos],
            None => return Err(MusicLibraryError::NoSuchPlaylist),
        };
        let tracks = match index_files(tracks, false) {
            Ok(tracks) => tracks,
            Err(e) => {
                return Err(e);
            }
        };
        let lib_set: HashSet<_> = lib.tracks.iter().collect();
        let mut intersecting_tracks: Vec<Track> = tracks
            .iter()
            .filter(|t| lib_set.contains(t))
            .cloned()
            .collect();
        playlist.tracks.append(&mut intersecting_tracks);
        drop(guard);
        save_library()?;
        Ok(())
    } else {
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}

pub fn remove_tracks(name: &str, ids: Vec<usize>) -> Result<(), MusicLibraryError> {
    trace!("Removing tracks from playlist \"{name}\"");
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = guard.as_mut() {
        let playlist = match lib
            .playlists
            .iter()
            .position(|playlist| playlist.name == name)
        {
            Some(pos) => &mut lib.playlists[pos],
            None => return Err(MusicLibraryError::NoSuchPlaylist),
        };
        for id in &ids {
            if *id >= playlist.tracks.len() {
                return Err(MusicLibraryError::IndexOutOfBounds);
            }
        }
        for id in &ids {
            playlist.tracks.remove(*id);
        }
        drop(guard);
        save_library()?;
        Ok(())
    } else {
        Err(MusicLibraryError::LibraryNotInitialized)
    }
}
