use std::{
    collections::HashSet,
    fs::{File, read},
    hash::{DefaultHasher, Hash, Hasher},
    io::{BufReader, Write},
    path::{Path, PathBuf},
    sync::{LazyLock, RwLock},
    time::{Duration, SystemTime},
};

use lofty::{
    file::{AudioFile, TaggedFile, TaggedFileExt},
    tag::{Accessor, ItemValue, Tag},
};
use log::{error, trace};
use mpipc::library::LibraryError;
use rmp_serde::{Deserializer, Serializer};
use rodio::Decoder;
use serde::{Deserialize, Serialize};

use crate::{
    data::{
        DATA_DIR,
        cache::{
            covers::{CoverError, get_track_cover},
            indexer::{index, index_files},
        },
    },
    send_event,
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
    pub duration: Duration,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub lyrics: Option<String>,
    pub comment: Option<String>,
    pub track: Option<u32>,
    pub track_total: Option<u32>,
    pub disc: Option<u32>,
    pub disc_total: Option<u32>,
    pub year: Option<u32>,
}

impl From<mpipc::library::Track> for Track {
    fn from(value: mpipc::library::Track) -> Self {
        Track {
            path: value.path,
            cover_path: value.cover_path,
            duration: value.duration,
            artist: value.artist,
            title: value.title,
            album: value.album,
            genre: value.genre,
            comment: value.comment,
            track: value.track,
            track_total: value.track_total,
            disc: value.disc,
            disc_total: value.disc_total,
            year: value.year,
            lyrics: value.lyrics,
        }
    }
}

impl From<Track> for mpipc::library::Track {
    fn from(value: Track) -> Self {
        mpipc::library::Track {
            path: value.path,
            cover_path: value.cover_path,
            duration: value.duration,
            artist: value.artist,
            title: value.title,
            album: value.album,
            genre: value.genre,
            comment: value.comment,
            track: value.track,
            track_total: value.track_total,
            disc: value.disc,
            disc_total: value.disc_total,
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

impl TryFrom<&TaggedFile> for Track {
    type Error = String;
    fn try_from(value: &TaggedFile) -> Result<Self, Self::Error> {
        let tag = match value.primary_tag() {
            Some(tag) => tag,
            None => {
                return Err(String::from("The provided tagged file had no tags"));
            }
        };

        let lyrics = match tag.get(&lofty::tag::ItemKey::Lyrics) {
            Some(tag_item) => match tag_item.value() {
                ItemValue::Text(lyrics) => Some(lyrics.clone()),
                _ => None,
            },
            None => None,
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
            year: tag.year(),
        })
    }
}

impl Track {
    pub fn get_cover(&mut self, tag: &Tag) -> Result<(), CoverError> {
        get_track_cover(self, tag, false)
    }

    pub fn extract_cover(&mut self, tag: &Tag) -> Result<(), CoverError> {
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

    #[cfg(feature = "mpris")]
    pub fn get_meta(self) -> mpris_server::Metadata {
        use mpris_server::{Time, builder::MetadataBuilder};

        MetadataBuilder::default()
            .length(Time::from_nanos(
                self.duration.as_nanos().try_into().unwrap_or(i64::MAX),
            ))
            .url(self.path.to_string_lossy())
            .art_url(self.cover_path.unwrap_or_default().to_string_lossy())
            .artist(self.artist)
            .title(self.title.unwrap_or_default())
            .album(self.album.unwrap_or_default())
            .genre(self.genre)
            .comment(self.comment)
            .track_number(self.track.unwrap_or(0).try_into().unwrap_or(0))
            .disc_number(self.disc.unwrap_or(0).try_into().unwrap_or(0))
            .lyrics(self.lyrics.unwrap_or_default())
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

    #[cfg(test)]
    pub fn new() -> Track {
        Track {
            path: PathBuf::new(),
            cover_path: None,
            duration: Duration::default(),
            artist: None,
            title: None,
            album: None,
            genre: None,
            comment: None,
            track: None,
            track_total: None,
            disc: None,
            disc_total: None,
            year: None,
            lyrics: None,
        }
    }

    /// Returns a [Vec] of unique [Track] structs for testing.
    #[cfg(test)]
    pub fn unique_tracks(size: usize) -> Vec<Track> {
        let mut tracks = Vec::new();
        for i in 0..size {
            let mut track = Track::new();
            track.duration = Duration::from_secs(i.try_into().unwrap());
            tracks.push(track);
        }
        tracks
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
}

impl From<mpipc::library::Playlist> for Playlist {
    fn from(value: mpipc::library::Playlist) -> Self {
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

impl From<Playlist> for mpipc::library::Playlist {
    fn from(value: Playlist) -> Self {
        let mut tracks = Vec::new();
        for track in value.tracks {
            tracks.push(track.into());
        }
        mpipc::library::Playlist {
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

    pub(crate) fn remove_tracks(&mut self, mut ids: Vec<usize>) -> Result<(), LibraryError> {
        if let Some(_) = ids.iter().find(|i| i >= &&self.tracks.len()) {
            return Err(LibraryError::IndexOutOfBounds);
        }
        ids.dedup();
        let remove_set: HashSet<usize> =
            ids.into_iter().filter(|&i| i < self.tracks.len()).collect();
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

#[derive(Default, Clone, Debug)]
pub(crate) struct MusicLibrary {
    pub playlists: Vec<Playlist>,
    pub tracks: Vec<Track>,
}

impl From<mpipc::library::MusicLibrary> for MusicLibrary {
    fn from(value: mpipc::library::MusicLibrary) -> Self {
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

impl From<MusicLibrary> for mpipc::library::MusicLibrary {
    fn from(value: MusicLibrary) -> Self {
        let mut playlists = Vec::new();
        let mut tracks = Vec::new();
        for playlist in value.playlists {
            playlists.push(playlist.into());
        }
        for track in value.tracks {
            tracks.push(track.into());
        }
        mpipc::library::MusicLibrary { playlists, tracks }
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

static LIBRARY_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data = DATA_DIR.read().unwrap().clone().unwrap();
    data.push("playlists");
    data
});

static MUSIC_LIBRARY: RwLock<Option<MusicLibrary>> = RwLock::new(None);

pub(crate) fn get_library() -> Result<MusicLibrary, LibraryError> {
    if let Some(lib) = MUSIC_LIBRARY.read().unwrap().clone() {
        Ok(lib)
    } else {
        error!("Tried to get the music library, but it was uninitialized!");
        Err(LibraryError::LibraryNotInitialized)
    }
}

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
pub(crate) fn save_library() -> Result<(), LibraryError> {
    trace!("Saving the library state");

    let lib = MUSIC_LIBRARY.read().unwrap().clone();

    if let Some(lib_data) = lib {
        let lib = ConfMusicLibrary::from(lib_data.clone());

        let library_state = LIBRARY_FILE.clone();

        let mut data = Vec::new();
        if let Err(e) = lib.serialize(&mut Serializer::new(&mut data)) {
            error!("Could not serialize the music library: {e}");
            return Err(LibraryError::CacheError);
        }

        let mut file = match File::create(library_state) {
            Ok(file) => file,
            Err(e) => {
                error!("Could not open the library in write-only mode: {e}");
                return Err(LibraryError::CacheError);
            }
        };

        match file.write_all(&data) {
            Ok(_) => {
                let _ = send_event(mpipc::Event::LibraryChanged(lib_data.into()));
                Ok(())
            }
            Err(e) => {
                error!("Could not write to the library: {e}");
                Err(LibraryError::CacheError)
            }
        }
    } else {
        error!("Cannot save the library since it is uninitialized!");
        Err(LibraryError::CacheError)
    }
}

/// Load the music library from the playlists file.
pub(crate) fn load(rebuild_covers: bool) -> Result<(), LibraryError> {
    trace!("Loading the music library");

    let time_start = SystemTime::now();

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

    let library_state = LIBRARY_FILE.clone();

    trace!("Loading the library state from {library_state:?}");

    let exists = match library_state.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            error!("Could not check if the library exists: {e}");
            return Err(LibraryError::CacheError);
        }
    };

    if exists {
        trace!("Restoring the library state");

        if library_state.is_dir() {
            error!("The item at {library_state:?} must not be a directory!");
            return Err(LibraryError::CacheError);
        }

        let data = match read(library_state) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not read the library: {e}");
                return Err(LibraryError::CacheError);
            }
        };

        let lib_conf = match ConfMusicLibrary::deserialize(&mut Deserializer::from_read_ref(&data))
        {
            Ok(data) => data,
            Err(e) => {
                error!("Could not decode the contents of the library state file: {e}");
                return Err(LibraryError::CacheError);
            }
        };

        let lib = MusicLibrary::from_loaded_lib(lib_conf, tracks);
        let mut guard = MUSIC_LIBRARY.write().unwrap();
        *guard = Some(lib);
        drop(guard);
    } else {
        trace!("The library file does not exist, creating a new library");
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

    Ok(())
}

// FIX: This function should fail if any of the tracks are not registered in the music library
// TODO: This function should also accept directory paths
pub(crate) fn create_playlist(
    name: String,
    tracks: &Option<Vec<PathBuf>>,
) -> Result<(), LibraryError> {
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
            trace!("Created a new playlist with name \"{name}\"");
            Ok(())
        } else {
            error!("A playlist with name \"{name}\" already exists");
            Err(LibraryError::PlaylistExists)
        }
    } else {
        error!("Cannot modify an uninitialized library");
        Err(LibraryError::LibraryNotInitialized)
    }
}

pub(crate) fn import_playlist_from_m3u8(
    name: Option<String>,
    m3u8_file: &Path,
) -> Result<(), LibraryError> {
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
                    return Err(LibraryError::CacheError);
                }
            }
        }
    };
    let lib = &*MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = lib {
        if lib.get_playlist_with_name(&name).is_none() {
            todo!("Importing playlists from M3U8 is not yet supported")
        } else {
            Err(LibraryError::PlaylistExists)
        }
    } else {
        Err(LibraryError::LibraryNotInitialized)
    }
}

// TODO: Make this function accept multiple playlists at once and fail if any of them are not
// present in the music library without making any changes.
pub(crate) fn delete_playlist(name: &str, save_state: bool) -> Result<(), LibraryError> {
    trace!("Deleting playlist with name \"{name}\"");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    // FIX: This likely won't work as expected
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
            None => Err(LibraryError::NoSuchPlaylist),
        }
    } else {
        Err(LibraryError::LibraryNotInitialized)
    }
}

// FIX: This function should fail if any of the tracks are not registered in the music library
// TODO: This function should also accept directory paths
pub(crate) fn add_tracks(name: &str, tracks: Vec<PathBuf>) -> Result<(), LibraryError> {
    trace!("Appending tracks to playlist \"{name}\"");

    let mut guard = MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = guard.as_mut() {
        let playlist = match lib
            .playlists
            .iter()
            .position(|playlist| playlist.name == name)
        {
            Some(pos) => &mut lib.playlists[pos],
            None => return Err(LibraryError::NoSuchPlaylist),
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
        Err(LibraryError::LibraryNotInitialized)
    }
}

// TODO: Add a test to make sure this removes tracks properly
/// Remove tracks by indices from a specific playlist
pub(crate) fn remove_tracks(name: &str, ids: Vec<usize>) -> Result<(), LibraryError> {
    trace!("Removing tracks from playlist \"{name}\"");
    let mut guard = MUSIC_LIBRARY.write().unwrap();
    if let Some(lib) = guard.as_mut() {
        let playlist = match lib
            .playlists
            .iter()
            .position(|playlist| playlist.name == name)
        {
            Some(pos) => &mut lib.playlists[pos],
            None => return Err(LibraryError::NoSuchPlaylist),
        };
        playlist.remove_tracks(ids)?;
        drop(guard);
        save_library()?;
        Ok(())
    } else {
        Err(LibraryError::LibraryNotInitialized)
    }
}
