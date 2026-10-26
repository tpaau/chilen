use std::{
    collections::{HashMap, HashSet},
    fs::{File, read_to_string},
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use chilen_backend::music_lib::{Album, Artist, Genre, LibraryItem, MusicLibrary, Playlist};
use chrono::{DateTime, Utc};
use log::{debug, error, trace};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::gui::{Chilen, playback_view};

static STATE_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data_dir = crate::CACHE_DIR
        .clone()
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone();
    data_dir.push("state.json");
    data_dir
});

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecentItem {
    Playlist(Arc<Playlist>),
    Album(Arc<Album>),
    Artist(Arc<Artist>),
    Genre(Arc<Genre>),
}

impl From<Arc<Playlist>> for RecentItem {
    fn from(value: Arc<Playlist>) -> Self {
        Self::Playlist(value)
    }
}

impl From<Arc<Album>> for RecentItem {
    fn from(value: Arc<Album>) -> Self {
        Self::Album(value)
    }
}

impl From<Arc<Artist>> for RecentItem {
    fn from(value: Arc<Artist>) -> Self {
        Self::Artist(value)
    }
}

impl From<Arc<Genre>> for RecentItem {
    fn from(value: Arc<Genre>) -> Self {
        Self::Genre(value)
    }
}

impl From<RecentItem> for LibraryItem {
    fn from(value: RecentItem) -> Self {
        match value {
            RecentItem::Playlist(playlist) => playlist.into(),
            RecentItem::Album(album) => album.into(),
            RecentItem::Artist(artist) => artist.into(),
            RecentItem::Genre(genre) => genre.into(),
        }
    }
}

trait Storable<T>: Sized {
    fn from_stored(lib: &MusicLibrary, stored: T) -> Option<Self>;
    fn reload(self, lib: &MusicLibrary) -> Option<Self>;
}

impl Storable<RecentItemStored> for RecentItem {
    fn from_stored(lib: &MusicLibrary, stored: RecentItemStored) -> Option<Self> {
        match stored {
            RecentItemStored::Playlist(name) => lib
                .find_playlist_by_name(&name)
                .cloned()
                .map(Self::Playlist),
            RecentItemStored::Album(title) => lib.find_album(&title).cloned().map(Self::Album),
            RecentItemStored::Artist(name) => lib.find_artist(&name).cloned().map(Self::Artist),
            RecentItemStored::Genre(name) => lib.find_genre(&name).cloned().map(Self::Genre),
        }
    }

    fn reload(self, lib: &MusicLibrary) -> Option<Self> {
        match self {
            RecentItem::Playlist(playlist) => lib
                .find_playlist_by_id(playlist.id)
                .cloned()
                .map(Self::Playlist),
            RecentItem::Album(album) => lib.find_album(&album.title).cloned().map(Self::Album),
            RecentItem::Artist(artist) => lib.find_artist(&artist.name).cloned().map(Self::Artist),
            RecentItem::Genre(genre) => lib.find_genre(&genre.name).cloned().map(Self::Genre),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum RecentItemStored {
    Playlist(String),
    Album(String),
    Artist(String),
    Genre(String),
}

impl From<RecentItem> for RecentItemStored {
    fn from(value: RecentItem) -> Self {
        match value {
            RecentItem::Playlist(playlist) => Self::Playlist(playlist.name.clone().into_string()),
            RecentItem::Album(album) => Self::Album(album.title.clone().into_string()),
            RecentItem::Artist(artist) => Self::Artist(artist.name.clone().into_string()),
            RecentItem::Genre(genre) => Self::Genre(genre.name.clone().into_string()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum LibraryItemStored {
    Track(u64),
    Playlist(String),
    Album(String),
    Artist(String),
    Genre(String),
}

impl Storable<LibraryItemStored> for LibraryItem {
    fn from_stored(lib: &MusicLibrary, stored: LibraryItemStored) -> Option<Self> {
        match stored {
            LibraryItemStored::Track(hash) => lib.track_from_hash(hash).cloned().map(Self::Track),
            LibraryItemStored::Playlist(name) => lib
                .find_playlist_by_name(&name)
                .cloned()
                .map(Self::Playlist),
            LibraryItemStored::Album(title) => lib.find_album(&title).cloned().map(Self::Album),
            LibraryItemStored::Artist(name) => lib.find_artist(&name).cloned().map(Self::Artist),
            LibraryItemStored::Genre(name) => lib.find_genre(&name).cloned().map(Self::Genre),
        }
    }

    fn reload(self, lib: &MusicLibrary) -> Option<Self> {
        match self {
            Self::Track(track) => lib
                .track_from_hash(track.hash_self())
                .cloned()
                .map(Self::Track),
            Self::Playlist(playlist) => lib
                .find_playlist_by_id(playlist.id)
                .cloned()
                .map(Self::Playlist),
            Self::Album(album) => lib.find_album(&album.title).cloned().map(Self::Album),
            Self::Artist(artist) => lib.find_artist(&artist.name).cloned().map(Self::Artist),
            Self::Genre(genre) => lib.find_genre(&genre.name).cloned().map(Self::Genre),
        }
    }
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

#[derive(Clone)]
struct RecentFavorite {
    item: LibraryItem,
    shown_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct RecentFavoriteStored {
    item: LibraryItemStored,
    shown_at: Option<DateTime<Utc>>,
}

impl From<RecentFavorite> for RecentFavoriteStored {
    fn from(value: RecentFavorite) -> Self {
        Self {
            item: value.item.into(),
            shown_at: value.shown_at,
        }
    }
}

#[derive(Clone)]
pub struct State {
    pub sidebar_opened: bool,
    pub playback_tab: playback_view::Tab,
    recent_items: Vec<RecentItem>,
    tracked_favorites: Vec<RecentFavorite>,
    shown_favorites: Vec<RecentFavorite>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            sidebar_opened: true,
            playback_tab: playback_view::Tab::default(),
            recent_items: Vec::new(),
            tracked_favorites: Vec::new(),
            shown_favorites: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StoredState {
    sidebar_opened: bool,
    playback_tab: playback_view::Tab,
    recent_items: Vec<RecentItemStored>,
    tracked_favorites: Vec<RecentFavoriteStored>,
}

impl From<State> for StoredState {
    fn from(value: State) -> Self {
        Self {
            sidebar_opened: value.sidebar_opened,
            playback_tab: value.playback_tab,
            recent_items: value.recent_items.into_iter().map(|i| i.into()).collect(),
            tracked_favorites: value
                .tracked_favorites
                .into_iter()
                .map(|i| i.into())
                .collect(),
        }
    }
}

impl State {
    const MAX_RECENT_ITEMS: usize = 8;

    fn from_stored(stored: StoredState) -> Self {
        Self {
            sidebar_opened: stored.sidebar_opened,
            playback_tab: stored.playback_tab,
            recent_items: Vec::new(),
            tracked_favorites: Vec::new(),
            shown_favorites: Vec::new(),
        }
    }

    fn restore_recent_items(&mut self, lib: &MusicLibrary, items: Vec<RecentItemStored>) {
        trace!("Restoring recently played items");

        self.recent_items = items
            .into_iter()
            .flat_map(|i| RecentItem::from_stored(lib, i))
            .take(Self::MAX_RECENT_ITEMS)
            .collect();
    }

    fn restore_tracked_favorites(&mut self, lib: &MusicLibrary, items: Vec<RecentFavoriteStored>) {
        trace!("Restoring tracked favorite items");

        let stored_map: HashMap<LibraryItemStored, Option<DateTime<Utc>>> =
            items.into_iter().map(|f| (f.item, f.shown_at)).collect();

        self.tracked_favorites = lib
            .favorites
            .clone()
            .into_iter()
            .map(|item| {
                let stored_at = stored_map
                    .get(&LibraryItemStored::from(item.clone()))
                    .cloned()
                    .unwrap_or(None);

                RecentFavorite {
                    item,
                    shown_at: stored_at,
                }
            })
            .collect();
    }

    fn sync_tracked_favorites(&mut self, lib: &MusicLibrary) {
        let existing_map: HashMap<LibraryItem, Option<DateTime<Utc>>> = self
            .tracked_favorites
            .iter()
            .map(|f| (f.item.clone(), f.shown_at))
            .collect();

        self.tracked_favorites = lib
            .favorites
            .clone()
            .into_iter()
            .map(|item| {
                let shown_at = existing_map.get(&item).cloned().unwrap_or(None);
                RecentFavorite { item, shown_at }
            })
            .collect();
    }

    fn refresh_shown_favorites(&mut self) {
        self.tracked_favorites.sort_by_key(|f| f.shown_at);

        let mut selected_indices: Vec<usize> = (0..self.tracked_favorites.len())
            .take(Self::MAX_RECENT_ITEMS)
            .collect();

        selected_indices.shuffle(&mut rand::rng());

        let now = Utc::now();
        self.shown_favorites = selected_indices
            .into_iter()
            .map(|idx| {
                let fav = &mut self.tracked_favorites[idx];
                fav.shown_at = Some(now);
                fav.clone()
            })
            .collect();
    }

    pub fn repair_shown_favorites(&mut self, lib: &MusicLibrary) {
        let current_favs: HashSet<&LibraryItem> = lib.favorites.iter().collect();

        self.shown_favorites
            .retain(|f| current_favs.contains(&f.item));

        if self.shown_favorites.len() < Self::MAX_RECENT_ITEMS {
            let gap = Self::MAX_RECENT_ITEMS - self.shown_favorites.len();

            let mut candidates: Vec<_> = self
                .tracked_favorites
                .clone()
                .into_iter()
                .filter(|tf| !self.shown_favorites.iter().any(|sf| sf.item == tf.item))
                .collect();

            candidates.sort_by_key(|f| f.shown_at);

            let now = Utc::now();
            for candidate in candidates.into_iter().take(gap) {
                if let Some(master_fav) = self
                    .tracked_favorites
                    .iter_mut()
                    .find(|f| f.item == candidate.item)
                {
                    master_fav.shown_at = Some(now);
                }

                self.shown_favorites.push(RecentFavorite {
                    item: candidate.item.clone(),
                    shown_at: Some(now),
                });
            }
        }
    }

    fn refresh_recent_items(&mut self, lib: &MusicLibrary) {
        trace!("Reloading the persistent state");
        let items = std::mem::take(&mut self.recent_items);
        self.recent_items = items.into_iter().flat_map(|i| i.reload(lib)).collect();
    }

    pub(super) fn save(self) -> Result<(), String> {
        let state: StoredState = self.into();
        let data = match serde_json::to_string_pretty(&state) {
            Ok(data) => data,
            Err(e) => return Err(e.to_string()),
        };

        let mut handle = match File::create(STATE_FILE.clone()) {
            Ok(handle) => handle,
            Err(e) => return Err(e.to_string()),
        };

        if let Err(e) = handle.write_all(data.as_bytes()) {
            return Err(e.to_string());
        }

        Ok(())
    }

    pub fn load() -> Result<Self, String> {
        let data = match read_to_string(STATE_FILE.clone()) {
            Ok(data) => data,
            Err(e) => {
                let msg = format!("Failed to load app state: {e}");
                error!("{}", msg.clone());
                return Err(msg);
            }
        };

        let stored_state: StoredState = match serde_json::from_str(&data) {
            Ok(state) => state,
            Err(e) => {
                let msg = format!("Failed to load app state: {e}");
                error!("{}", msg.clone());
                return Err(msg);
            }
        };

        debug!("App state loaded");

        Ok(State::from_stored(stored_state))
    }

    pub fn recent_items(&self) -> &Vec<RecentItem> {
        &self.recent_items
    }

    pub fn favorites(&self) -> Vec<LibraryItem> {
        self.shown_favorites
            .iter()
            .map(|f| f.item.clone())
            .collect()
    }
}

pub fn library_loaded(state: &mut Chilen) {
    if let Some(lib) = &state.library {
        if state.persistent.recent_items.is_empty() {
            let data = match read_to_string(STATE_FILE.clone()) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to load app state: {e}");
                    return;
                }
            };

            let stored_state: StoredState = match serde_json::from_str(&data) {
                Ok(state) => state,
                Err(e) => {
                    error!("Failed to load app state: {e}");
                    return;
                }
            };

            state
                .persistent
                .restore_recent_items(lib, stored_state.recent_items);
            state
                .persistent
                .restore_tracked_favorites(lib, stored_state.tracked_favorites);
            state.persistent.refresh_shown_favorites();
        } else {
            state.persistent.refresh_recent_items(lib);
            state.persistent.sync_tracked_favorites(lib);
            state.persistent.repair_shown_favorites(lib);
        }
        state.save_persistent();
    }
}

pub fn push_recent(state: &mut Chilen, item: impl Into<RecentItem>) {
    let item = item.into();

    if let Some(index) = state
        .persistent
        .recent_items
        .iter()
        .position(|i| *i == item)
    {
        state.persistent.recent_items.remove(index);
    } else if state.persistent.recent_items.len() == State::MAX_RECENT_ITEMS {
        state.persistent.recent_items.pop();
    }

    state.persistent.recent_items.insert(0, item);
    state.save_persistent();
}
