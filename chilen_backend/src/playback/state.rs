use std::{
    fs::{File, read},
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
    thread,
    time::Duration,
};

use log::{error, trace, warn};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use rand::seq::SliceRandom;

use crate::{
    Error,
    music_lib::{Album, Artist, CACHE_DIR, Genre, Playlist, Track, tracks_from_hashes},
    playback::{LoopState, PlaybackState, PlayerVolume, ShuffleState},
};

#[cfg(feature = "mpris")]
use crate::playback::mpris;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    StateInitialized(PlayerState),
    PositionChanged(usize),
    PlayerPositionChanged(Duration),
    PlayerVolumeChanged(PlayerVolume),
    PlaybackStateChanged(PlaybackState),
    TracksChanged(Vec<Arc<Track>>),
    ShuffledTrackIndicesChanged(Vec<usize>),
    QueueSourceChanged(QueueSource),
    ShuffleStateChanged(ShuffleState),
    LoopStateChanged(LoopState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Queue {
    Playlist(Arc<Playlist>),
    Album(Arc<Album>),
    Artist(Arc<Artist>),
    Genre(Arc<Genre>),
    AllTracks(Vec<Arc<Track>>),
    Custom {
        label: String,
        tracks: Vec<Arc<Track>>,
    },
}

impl Queue {
    pub fn source(&self) -> QueueSource {
        match self {
            Self::Playlist(playlist) => QueueSource::Playlist {
                name: playlist.name.clone(),
            },
            Self::Album(album) => QueueSource::Album {
                title: album.title.clone(),
            },
            Self::Artist(artist) => QueueSource::Artist {
                name: artist.name.clone(),
            },
            Self::Genre(genre) => QueueSource::Genre {
                name: genre.name.clone(),
            },
            Self::AllTracks(_) => QueueSource::AllTracks,
            Self::Custom { label, tracks: _ } => QueueSource::Custom {
                label: label.clone(),
            },
        }
    }

    pub fn tracks(self) -> Vec<Arc<Track>> {
        match self {
            Self::Playlist(playlist) => playlist.tracks.clone(),
            Self::Album(album) => album.tracks.clone(),
            Self::Artist(artist) => artist.tracks.clone(),
            Self::Genre(genre) => genre.tracks.clone(),
            Self::AllTracks(tracks) => tracks,
            Self::Custom { label: _, tracks } => tracks,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum QueueSource {
    #[default]
    None,
    AllTracks,
    Custom {
        label: String,
    },
    Playlist {
        name: String,
    },
    Album {
        title: String,
    },
    Artist {
        name: String,
    },
    Genre {
        name: String,
    },
}

impl QueueSource {
    pub fn identity(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::AllTracks => Some("All tracks"),
            Self::Custom { label } => Some(label),
            Self::Playlist { name } => Some(name),
            Self::Album { title } => Some(title),
            Self::Artist { name } => Some(name),
            Self::Genre { name } => Some(name),
        }
    }
}

/// Data structure used to store playback state on the disc and in the RAM at runtime.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerState {
    /// The index of the current track.
    ///
    /// It can either point to the `tracks` variable or `shuffled_tracks` is shuffle is supported
    /// and set to [`ShuffleState::On`].
    pub position: usize,
    pub player_position: Duration,
    pub player_volume: PlayerVolume,
    pub playback_state: PlaybackState,
    pub tracks: Vec<Arc<Track>>,
    pub shuffled_track_indices: Vec<usize>,
    pub queue_source: QueueSource,
    pub shuffle_state: ShuffleState,
    pub loop_state: LoopState,
}

impl TryFrom<PlayerStateRaw> for PlayerState {
    type Error = Error;
    fn try_from(value: PlayerStateRaw) -> Result<Self, Self::Error> {
        let (tracks, shuffled_track_indices) = {
            let result = tracks_from_hashes(value.track_hashes)?;
            if !result.unmatched.is_empty() {
                warn!("{} missing tracks in the queue", result.unmatched.len());
                let indices = result.matched.iter().enumerate().map(|(i, _)| i).collect();
                (result.matched, indices)
            } else {
                let mut indices_deduped = value.shuffled_track_indices;
                indices_deduped.dedup();
                if result.matched.len() == indices_deduped.len()
                    && indices_deduped.iter().max().unwrap_or(&0) < &result.matched.len()
                {
                    (result.matched, indices_deduped)
                } else {
                    let indices = result.matched.iter().enumerate().map(|(i, _)| i).collect();
                    (result.matched, indices)
                }
            }
        };
        let playback_state = if !tracks.is_empty() {
            PlaybackState::Stopped
        } else {
            PlaybackState::Paused
        };
        let position = value.position.min(tracks.len() - 1);
        Ok(Self {
            position,
            player_position: value.player_position,
            player_volume: value.player_volume,
            playback_state,
            tracks,
            shuffled_track_indices,
            queue_source: value.queue_source,
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
        })
    }
}

impl PlayerState {
    pub fn queue_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn current(&self) -> Option<Arc<Track>> {
        match self.shuffle_state {
            ShuffleState::Off => {
                return self.tracks.get(self.position).cloned();
            }
            ShuffleState::On => {
                if let Some(index) = self.shuffled_track_indices.get(self.position) {
                    return self.tracks.get(*index).cloned();
                }
            }
        }
        None
    }

    /// Returns the index of the current track in the *unshuffled* queue.
    ///
    /// If the queue is not shuffled, this will simply return the
    /// [position](PlayerState::position) of the player. Otherwise, it will return the index
    /// attached to the current track.
    ///
    /// Returns [`None`] if the index is out of bounds.
    pub fn real_track_index(&self, index: usize) -> Option<usize> {
        match self.shuffle_state {
            ShuffleState::Off => {
                if index >= self.tracks.len() {
                    None
                } else {
                    Some(index)
                }
            }
            ShuffleState::On => self.shuffled_track_indices.get(index).copied(),
        }
    }

    pub fn can_seek(&self) -> bool {
        self.playback_state != PlaybackState::Stopped
    }

    pub fn can_play(&self) -> bool {
        self.position < self.tracks.len() && !self.is_playing()
    }

    pub fn can_pause(&self) -> bool {
        self.playback_state == PlaybackState::Playing
    }

    pub fn can_toggle_playing(&self) -> bool {
        match self.playback_state {
            PlaybackState::Playing => self.can_pause(),
            PlaybackState::Paused | PlaybackState::Stopped => self.can_play(),
        }
    }

    pub fn can_go_next(&self) -> bool {
        match self.loop_state {
            LoopState::Off => !self.tracks.is_empty() && self.position < self.tracks.len() - 1,
            _ => !self.tracks.is_empty(),
        }
    }

    pub fn can_go_previous(&self) -> bool {
        match self.loop_state {
            LoopState::Off => !self.tracks.is_empty() && self.position > 0,
            _ => !self.tracks.is_empty(),
        }
    }

    pub fn is_playing(&self) -> bool {
        self.playback_state == PlaybackState::Playing
    }

    pub fn is_paused(&self) -> bool {
        self.playback_state == PlaybackState::Paused
    }

    pub fn stopped(&self) -> bool {
        self.playback_state == PlaybackState::Stopped
    }

    pub fn shuffle_enabled(&self) -> bool {
        self.shuffle_state.enabled()
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::StateInitialized(player_state) => *self = player_state,
            Event::PositionChanged(pos) => self.position = pos,
            Event::PlayerPositionChanged(player_position) => self.player_position = player_position,
            Event::PlayerVolumeChanged(player_volume) => self.player_volume = player_volume,
            Event::PlaybackStateChanged(playback_state) => self.playback_state = playback_state,
            Event::ShuffleStateChanged(shuffle_state) => self.shuffle_state = shuffle_state,
            Event::LoopStateChanged(loop_state) => self.loop_state = loop_state,
            Event::TracksChanged(tracks) => self.tracks = tracks,
            Event::ShuffledTrackIndicesChanged(indices) => self.shuffled_track_indices = indices,
            Event::QueueSourceChanged(queue_source) => self.queue_source = queue_source,
        }
    }

    pub(crate) fn on_playback_state_changed(&self) {
        trace!("Playback state changed: {}", self.playback_state);

        crate::send_event(crate::Event::Playback(Event::PlaybackStateChanged(
            self.playback_state,
        )));

        #[cfg(feature = "mpris")]
        {
            use mpris_server::Property;
            mpris::update_properties(vec![
                Property::PlaybackStatus(mpris::playback_state_2_mpris(&self.playback_state)),
                Property::CanPlay(self.can_play()),
                Property::CanPause(self.can_pause()),
                Property::CanSeek(self.can_seek()),
            ]);
        }
    }

    pub(crate) fn set_queue(&mut self, queue: Queue) {
        self.position = 0;
        self.queue_source = queue.source();
        self.tracks = queue.tracks();
        self.set_playback_state(PlaybackState::Stopped);
        if self.shuffle_state.enabled() {
            self.shuffle();
        }
        crate::send_event(crate::Event::Playback(Event::QueueSourceChanged(
            self.queue_source.clone(),
        )));
        self.on_track_changed();
    }

    pub(crate) fn play_new_queue(&mut self, queue: Queue, index: usize) {
        trace!("Setting a new queue and playing a track at index {index}");
        let shuffle_enabled = self.shuffle_state.enabled();
        if shuffle_enabled {
            // Setting this manually to prevent needless shuffling
            self.shuffle_state = ShuffleState::Off;
        }
        self.position = index;
        self.queue_source = queue.source();
        crate::send_event(crate::Event::Playback(Event::QueueSourceChanged(
            self.queue_source.clone(),
        )));
        self.tracks = queue.tracks();
        crate::send_event(crate::Event::Playback(Event::TracksChanged(
            self.tracks.clone(),
        )));
        if shuffle_enabled {
            self.shuffle_state = ShuffleState::On;
            self.shuffle();
        }
        self.on_track_changed();
        self.set_player_position(Duration::default());
    }

    pub(crate) fn append_tracks(&mut self, tracks: &mut Vec<Arc<Track>>) {
        self.tracks.append(tracks);
        crate::send_event(crate::Event::Playback(Event::TracksChanged(
            self.tracks.clone(),
        )));
        if self.shuffle_state.enabled() {
            self.shuffle();
        }
        #[cfg(feature = "mpris")]
        {
            use mpris_server::{Metadata, Property};

            mpris::update_properties(vec![
                match self.current() {
                    Some(track) => Property::Metadata(track.get_meta(self.position)),
                    None => Property::Metadata(Metadata::new()),
                },
                Property::CanGoPrevious(self.can_go_previous()),
                Property::CanGoNext(self.can_go_next()),
                Property::CanPlay(self.can_play()),
                Property::CanPause(self.can_pause()),
            ]);
        }
    }

    // FIX: There should be a separate shuffle function that shuffles the entire queue, for setting
    // the current queue as shuffled.
    /// Shuffle the queue.
    ///
    /// This operation will preserve the current track, but it will be put first in the queue
    /// (position will reset to 0).
    pub(crate) fn shuffle(&mut self) {
        if self.tracks.is_empty() {
            use log::warn;

            warn!("Refusing to shuffle an empty queue");
            return;
        }

        self.shuffled_track_indices = self.tracks.iter().enumerate().map(|(i, _)| i).collect();
        let len = self.shuffled_track_indices.len();
        let pos = self.position;

        self.shuffled_track_indices.swap(pos, 0);
        let mut rng = rand::rng();
        self.shuffled_track_indices[1..len].shuffle(&mut rng);
        self.position = 0;

        crate::send_event(crate::Event::Playback(Event::PositionChanged(
            self.position,
        )));
        crate::send_event(crate::Event::Playback(Event::ShuffledTrackIndicesChanged(
            self.shuffled_track_indices.clone(),
        )));
    }

    pub(crate) fn set_shuffle_state(&mut self, shuffle_state: ShuffleState) {
        if self.shuffle_state != shuffle_state {
            if !shuffle_state.enabled()
                && let Some(track) = self.current()
            {
                match self.tracks.iter().position(|t| *t == track) {
                    Some(pos) => {
                        self.position = pos;
                        crate::send_event(crate::Event::Playback(Event::PositionChanged(
                            self.position,
                        )));
                    }
                    None => {
                        log::warn!(
                            "Could not find the previous track in the queue, this should never happen"
                        );
                        thread::spawn(crate::playback::stop);
                    }
                }
            } else {
                self.shuffle();
            }
            self.shuffle_state = shuffle_state;
            crate::send_event(crate::Event::Playback(Event::ShuffleStateChanged(
                self.shuffle_state,
            )));
            #[cfg(feature = "mpris")]
            {
                use mpris_server::Property;

                let properties = vec![
                    Property::Shuffle(self.shuffle_state.enabled()),
                    Property::CanGoPrevious(self.can_go_previous()),
                    Property::CanGoNext(self.can_go_next()),
                ];
                mpris::update_properties(properties);
            }
        }
    }

    pub(crate) fn increment_player_position(&mut self, duration: Duration) {
        self.player_position += duration;
        crate::send_event(crate::Event::Playback(Event::PlayerPositionChanged(
            self.player_position,
        )));
        #[cfg(feature = "mpris")]
        {
            use mpris_server::{Metadata, Property};

            let meta = match self.current() {
                Some(track) => track.get_meta(self.position),
                None => Metadata::new(),
            };
            mpris::update_properties(vec![Property::Metadata(meta)]);
            mpris::set_position(self.player_position);
        }
    }

    pub(crate) fn set_player_position(&mut self, player_position: Duration) {
        if self.player_position != player_position {
            self.player_position = player_position;
            crate::send_event(crate::Event::Playback(Event::PlayerPositionChanged(
                self.player_position,
            )));
            #[cfg(feature = "mpris")]
            {
                use mpris_server::{Metadata, Property};

                let meta = match self.current() {
                    Some(track) => track.get_meta(self.position),
                    None => Metadata::new(),
                };
                mpris::update_properties(vec![Property::Metadata(meta)]);
                mpris::set_position(player_position);
            }
        }
    }

    pub(crate) fn set_player_volume(&mut self, player_volume: PlayerVolume) {
        if self.player_volume != player_volume {
            self.player_volume = player_volume;
            crate::send_event(crate::Event::Playback(Event::PlayerVolumeChanged(
                self.player_volume,
            )));
            #[cfg(feature = "mpris")]
            {
                use mpris_server::Property;

                let properties = vec![Property::Volume(self.player_volume.get())];
                mpris::update_properties(properties);
            }
        }
    }

    pub(crate) fn set_loop_state(&mut self, loop_state: LoopState) {
        if self.loop_state != loop_state {
            self.loop_state = loop_state;
            crate::send_event(crate::Event::Playback(Event::LoopStateChanged(
                self.loop_state,
            )));
            #[cfg(feature = "mpris")]
            {
                use mpris_server::Property;

                let properties = vec![
                    Property::LoopStatus(mpris::loop_state_2_mpris(&self.loop_state)),
                    Property::CanGoPrevious(self.can_go_previous()),
                    Property::CanGoNext(self.can_go_next()),
                ];
                mpris::update_properties(properties);
            }
        }
    }

    pub(crate) fn set_playback_state(&mut self, playback_state: PlaybackState) {
        if self.playback_state != playback_state {
            self.playback_state = playback_state;
            if self.playback_state == PlaybackState::Stopped {
                self.set_player_position(Duration::default());
            }
            self.on_playback_state_changed();
        }
    }

    pub(crate) fn play_track(&mut self, index: usize) -> Option<Arc<Track>> {
        if index < self.tracks.len() {
            self.position = index;
            self.on_track_changed();
            self.current()
        } else {
            None
        }
    }

    pub(crate) fn next_track(&mut self) -> Option<Arc<Track>> {
        match self.loop_state {
            LoopState::Off => {
                if !self.tracks.is_empty() && self.position < self.tracks.len() - 1 {
                    self.position += 1;
                    self.on_track_changed();
                    return self.current();
                }
                None
            }
            LoopState::Track => {
                self.on_track_changed();
                self.current()
            }
            LoopState::Playlist => {
                if self.tracks.is_empty() {
                    None
                } else if !self.tracks.is_empty() && self.position < self.tracks.len() - 1 {
                    self.position += 1;
                    self.on_track_changed();
                    self.current()
                } else {
                    self.position = 0;
                    self.on_track_changed();
                    self.current()
                }
            }
        }
    }

    pub(crate) fn previous_track(&mut self) -> Option<Arc<Track>> {
        match self.loop_state {
            LoopState::Off => {
                if self.position > 0 && !self.tracks.is_empty() {
                    self.position -= 1;
                    self.on_track_changed();
                    self.current()
                } else {
                    None
                }
            }
            LoopState::Track => {
                self.on_track_changed();
                self.current()
            }
            LoopState::Playlist => {
                if self.tracks.is_empty() {
                    None
                } else if self.position > 0 {
                    self.position -= 1;
                    self.on_track_changed();
                    self.current()
                } else if !self.tracks.is_empty() {
                    self.position = self.tracks.len() - 1;
                    self.on_track_changed();
                    self.current()
                } else {
                    None
                }
            }
        }
    }

    fn on_track_changed(&self) {
        crate::send_event(crate::Event::Playback(Event::PositionChanged(
            self.position,
        )));

        #[cfg(feature = "mpris")]
        {
            use mpris_server::{Metadata, Property};

            let properties = vec![
                match self.current() {
                    Some(track) => Property::Metadata(track.get_meta(self.position)),
                    None => Property::Metadata(Metadata::new()),
                },
                Property::CanGoPrevious(self.can_go_previous()),
                Property::CanGoNext(self.can_go_next()),
            ];
            mpris::update_properties(properties);
        }
    }

    #[cfg(feature = "mpris")]
    pub(crate) fn get_mpris_properties(&self) -> Vec<mpris_server::Property> {
        use mpris_server::{Metadata, Property};

        vec![
            Property::PlaybackStatus(mpris::playback_state_2_mpris(&self.playback_state)),
            Property::LoopStatus(mpris::loop_state_2_mpris(&self.loop_state)),
            Property::Shuffle(self.shuffle_state.enabled()),
            Property::Volume(self.player_volume.get()),
            Property::CanGoNext(self.can_go_next()),
            Property::CanGoPrevious(self.can_go_previous()),
            Property::CanPlay(self.can_play()),
            Property::CanPause(self.can_pause()),
            Property::CanSeek(self.can_seek()),
            match self.current() {
                Some(track) => Property::Metadata(track.get_meta(self.position)),
                None => Property::Metadata(Metadata::new()),
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerStateRaw {
    position: usize,
    player_position: Duration,
    player_volume: PlayerVolume,
    track_hashes: Vec<u64>,
    shuffled_track_indices: Vec<usize>,
    queue_source: QueueSource,
    shuffle_state: ShuffleState,
    loop_state: LoopState,
}

impl From<PlayerState> for PlayerStateRaw {
    fn from(value: PlayerState) -> Self {
        let track_hashes = Track::hash_tracks(value.tracks);
        Self {
            position: value.position,
            player_position: value.player_position,
            player_volume: value.player_volume,
            track_hashes,
            shuffled_track_indices: value.shuffled_track_indices,
            queue_source: value.queue_source,
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
        }
    }
}

static STATE_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data = CACHE_DIR.read().unwrap().clone().unwrap();
    data.push("player_state");
    data
});

pub(crate) static PLAYER_STATE: LazyLock<Arc<RwLock<Option<PlayerState>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub(crate) fn unwrap_state_ref(maybe_state: Option<&PlayerState>) -> Result<&PlayerState, Error> {
    match maybe_state {
        Some(state) => Ok(state),
        None => Err(Error::StateNotInitialized),
    }
}

pub(crate) fn unwrap_state_mut(
    maybe_state: Option<&mut PlayerState>,
) -> Result<&mut PlayerState, Error> {
    match maybe_state {
        Some(state) => Ok(state),
        None => Err(Error::StateNotInitialized),
    }
}

pub(crate) fn save_state(state: PlayerState) -> Result<(), String> {
    let state_raw: PlayerStateRaw = state.into();
    let state_file = STATE_FILE.clone();

    let mut data = Vec::new();
    if let Err(e) = state_raw.serialize(&mut Serializer::new(&mut data)) {
        error!("Could not serialize the player state: {e}");
        return Err(e.to_string());
    }

    let mut file = match File::create(state_file) {
        Ok(file) => file,
        Err(e) => {
            error!("Could not open the player state cache in write-only mode: {e}");
            return Err(e.to_string());
        }
    };

    match file.write_all(&data) {
        Ok(_) => {
            trace!("Saved player state to cache");
            Ok(())
        }
        Err(e) => {
            error!("Could not write to the player state cache: {e}");
            Err(e.to_string())
        }
    }
}

pub(crate) fn background_save_state(state: PlayerState) {
    thread::spawn(|| {
        if let Err(e) = save_state(state) {
            error!("Could not save player state to cache: {e}");
        }
    });
}

pub(crate) fn restore_state_from_cache() -> Result<PlayerState, String> {
    let state_file = STATE_FILE.clone();

    trace!("Restoring player state from {state_file:?}");

    let state_exists = match state_file.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            error!("Could not check if the player state file exists: {e}");
            return Err(e.to_string());
        }
    };

    if state_exists {
        let data = match read(state_file) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not read the player state cache: {e}");
                return Err(e.to_string());
            }
        };

        let state_raw = match PlayerStateRaw::deserialize(&mut Deserializer::from_read_ref(&data)) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not decode the contents of the player state file: {e}");
                return Err(e.to_string());
            }
        };

        match PlayerState::try_from(state_raw) {
            Ok(state) => Ok(state),
            Err(e) => {
                error!("Could not restore player state from cache: {e}");
                Err(e.to_string())
            }
        }
    } else {
        Ok(PlayerState::default())
    }
}
