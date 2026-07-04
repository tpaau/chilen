use std::{
    fs::{File, read},
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
    thread,
    time::Duration,
};

use chilen_ipc::playback::{LoopState, PlaybackRate, PlaybackState, PlayerVolume};
use log::{error, trace};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use chilen_ipc::playback::ShuffleState;
use rand::seq::SliceRandom;

use crate::music_lib::state::Track;
use crate::music_lib::{CACHE_DIR, tracks_from_hashes};

#[cfg(feature = "mpris")]
use crate::playback::mpris;

/// Data structure used to store playback state on the disc and in the RAM at runtime.
#[derive(Debug, Clone, Default)]
pub(crate) struct PlayerState {
    /// The index of the current track.
    ///
    /// It can either point to the `tracks` variable or `shuffled_tracks` is shuffle is supported
    /// and set to [`ShuffleState::Playlist`].
    pub position: usize,
    pub player_position: Duration,
    pub player_volume: PlayerVolume,
    pub playback_state: PlaybackState,
    pub tracks: Vec<Track>,
    pub shuffled_tracks: Vec<Track>,
    pub shuffle_state: ShuffleState,
    pub loop_state: LoopState,
    pub playback_rate: PlaybackRate,
}

impl TryFrom<PlayerStateRaw> for PlayerState {
    type Error = chilen_ipc::Error;
    fn try_from(value: PlayerStateRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            position: value.position,
            player_position: value.player_position,
            player_volume: value.player_volume,
            playback_state: PlaybackState::Stopped,
            tracks: tracks_from_hashes(value.track_hashes)?
                .into_iter()
                .map(|t| t.as_ref().clone())
                .collect(),
            shuffled_tracks: tracks_from_hashes(value.shuffled_track_hashes)?
                .into_iter()
                .map(|t| t.as_ref().clone())
                .collect(),
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
            playback_rate: value.playback_rate,
        })
    }
}

impl PlayerState {
    #[cfg(feature = "mpris")]
    fn on_track_changed(&self) {
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

    #[cfg(feature = "mpris")]
    pub(crate) fn on_playback_state_changed(&self) {
        use mpris_server::Property;

        trace!("Playback state changed: {}", self.playback_state);
        mpris::update_properties(vec![
            Property::PlaybackStatus(mpris::playback_state_2_mpris(&self.playback_state)),
            Property::CanPlay(self.can_play()),
            Property::CanPause(self.can_pause()),
            Property::CanSeek(self.can_seek()),
        ]);
    }

    #[cfg(feature = "mpris")]
    pub(crate) fn get_mpris_properties(&self) -> Vec<mpris_server::Property> {
        use mpris_server::{Metadata, Property};

        vec![
            Property::PlaybackStatus(mpris::playback_state_2_mpris(&self.playback_state)),
            Property::LoopStatus(mpris::loop_state_2_mpris(&self.loop_state)),
            Property::Rate(self.playback_rate.get_value()),
            Property::Shuffle(self.shuffle_state.into()),
            Property::Volume(self.player_volume.get()),
            Property::MinimumRate(self.get_actual_min_rate()),
            Property::MaximumRate(self.get_actual_max_rate()),
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

    #[cfg(feature = "mpris")]
    fn rate_modification_allowed() -> bool {
        false
    }

    /// Returns the minimum playback rate taking into account whether the playback rate
    /// modification is allowed.
    #[cfg(feature = "mpris")]
    pub fn get_actual_min_rate(&self) -> f64 {
        if Self::rate_modification_allowed() {
            self.playback_rate.get_min()
        } else {
            self.playback_rate.get_value()
        }
    }

    /// Returns the maximum playback rate taking into account whether the playback rate
    /// modification is allowed.
    #[cfg(feature = "mpris")]
    pub fn get_actual_max_rate(&self) -> f64 {
        if Self::rate_modification_allowed() {
            self.playback_rate.get_max()
        } else {
            self.playback_rate.get_value()
        }
    }

    pub fn set_tracks(&mut self, tracks: Vec<Track>) {
        self.position = 0;
        self.tracks = tracks;
        self.set_playback_state(PlaybackState::Stopped);
        if self.shuffle_state == ShuffleState::On {
            self.shuffle();
        }
        let tracks = if self.shuffle_state == ShuffleState::On {
            &self.shuffled_tracks
        } else {
            &self.tracks
        };
        #[cfg(feature = "mpris")]
        self.on_track_changed();
    }

    pub fn append_tracks(&mut self, tracks: &mut Vec<Track>) {
        self.tracks.append(tracks);
        if self.shuffle_state == ShuffleState::On {
            self.shuffle();
        }
        let tracks = if self.shuffle_state == ShuffleState::On {
            &self.shuffled_tracks
        } else {
            &self.tracks
        };
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

    /// Shuffle the queue without changing the current track.
    pub fn shuffle(&mut self) {
        if self.tracks.is_empty() {
            use log::warn;

            warn!("Refusing to shuffle an empty queue");
            return;
        }
        // Maybe a position check here is necessary to prevent panics?
        let mut tracks = self.tracks.clone();
        let prev_pos = self.position;
        let track = tracks.swap_remove(prev_pos);
        let mut rng = rand::rng();
        tracks.shuffle(&mut rng);
        tracks.insert(prev_pos, track);
        self.shuffled_tracks = tracks;
    }

    pub fn set_shuffle_state(&mut self, shuffle_state: ShuffleState) {
        if self.shuffle_state != shuffle_state {
            if shuffle_state == ShuffleState::Off
                && let Some(track) = self.current().cloned()
            {
                match self.tracks.iter().position(|t| *t == track) {
                    Some(pos) => self.position = pos,
                    None => {
                        log::warn!(
                            "Could not find the previous track in the queue, this should never happen"
                        );
                        thread::spawn(crate::playback::stop);
                    }
                }
            }
            self.shuffle_state = shuffle_state;
            #[cfg(feature = "mpris")]
            {
                use mpris_server::Property;

                let properties = vec![
                    Property::Shuffle(self.shuffle_state.into()),
                    Property::CanGoPrevious(self.can_go_previous()),
                    Property::CanGoNext(self.can_go_next()),
                ];
                mpris::update_properties(properties);
            }
        }
    }

    pub fn increment_player_position(&mut self, duration: Duration) {
        self.player_position += duration;
    }

    pub fn set_player_position(&mut self, player_position: Duration) {
        if self.player_position != player_position {
            self.player_position = player_position;
            #[cfg(feature = "mpris")]
            {
                use mpris_server::{Metadata, Property};

                let meta = match self.current() {
                    Some(track) => track.get_meta(self.position),
                    None => Metadata::new(),
                };
                let properties = vec![Property::Metadata(meta)];
                mpris::update_properties(properties);
                mpris::set_position(player_position);
            }
        }
    }

    pub fn set_player_volume(&mut self, player_volume: PlayerVolume) {
        if self.player_volume != player_volume {
            self.player_volume = player_volume;
            #[cfg(feature = "mpris")]
            {
                use mpris_server::Property;

                let properties = vec![Property::Volume(self.player_volume.get())];
                mpris::update_properties(properties);
            }
        }
    }

    pub fn set_loop_state(&mut self, loop_state: LoopState) {
        if self.loop_state != loop_state {
            self.loop_state = loop_state;
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

    pub fn set_rate(&mut self, rate: f64) {
        if self.playback_rate.get_value() != rate {
            self.playback_rate.set_value(rate);
            #[cfg(feature = "mpris")]
            {
                use mpris_server::Property;

                let properties = vec![
                    Property::Rate(self.playback_rate.get_value()),
                    Property::MinimumRate(self.get_actual_min_rate()),
                    Property::MaximumRate(self.get_actual_max_rate()),
                ];
                mpris::update_properties(properties);
            }
        }
    }

    pub fn set_playback_state(&mut self, playback_state: PlaybackState) {
        if self.playback_state != playback_state {
            self.playback_state = playback_state;
            if self.playback_state == PlaybackState::Stopped {
                self.set_player_position(Duration::default());
            }
            #[cfg(feature = "mpris")]
            self.on_playback_state_changed();
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.shuffle_state {
            ShuffleState::Off => self.tracks.is_empty(),
            ShuffleState::On => self.shuffled_tracks.is_empty(),
        }
    }

    pub fn current(&self) -> Option<&Track> {
        match self.shuffle_state {
            ShuffleState::Off => {
                if self.position < self.tracks.len() {
                    return Some(&self.tracks[self.position]);
                }
            }
            ShuffleState::On => {
                if self.position < self.shuffled_tracks.len() {
                    return Some(&self.shuffled_tracks[self.position]);
                }
            }
        }
        None
    }

    pub fn play_track(&mut self, index: usize) -> Option<&Track> {
        if index < self.tracks.len() {
            self.position = index;
            #[cfg(feature = "mpris")]
            self.on_track_changed();
            self.current()
        } else {
            None
        }
    }

    #[cfg(feature = "mpris")]
    pub fn can_seek(&self) -> bool {
        self.playback_state != PlaybackState::Stopped
    }

    #[cfg(feature = "mpris")]
    pub fn can_play(&self) -> bool {
        match self.shuffle_state {
            ShuffleState::Off => {
                self.position < self.tracks.len() && self.playback_state != PlaybackState::Playing
            }
            ShuffleState::On => {
                self.position < self.shuffled_tracks.len()
                    && self.playback_state != PlaybackState::Playing
            }
        }
    }

    #[cfg(feature = "mpris")]
    pub fn can_pause(&self) -> bool {
        self.playback_state == PlaybackState::Playing
    }

    pub fn can_go_next(&self) -> bool {
        match self.loop_state {
            LoopState::Off => match self.shuffle_state {
                ShuffleState::Off => {
                    !self.tracks.is_empty() && self.position < self.tracks.len() - 1
                }
                ShuffleState::On => {
                    !self.shuffled_tracks.is_empty()
                        && self.position < self.shuffled_tracks.len() - 1
                }
            },
            _ => match self.shuffle_state {
                ShuffleState::Off => !self.tracks.is_empty(),
                ShuffleState::On => !self.shuffled_tracks.is_empty(),
            },
        }
    }

    pub fn next(&mut self) -> Option<&Track> {
        match self.loop_state {
            LoopState::Off => {
                let tracks = match self.shuffle_state {
                    ShuffleState::Off => &self.tracks,
                    ShuffleState::On => &self.shuffled_tracks,
                };
                if !tracks.is_empty() && self.position < tracks.len() - 1 {
                    self.position += 1;
                    #[cfg(feature = "mpris")]
                    self.on_track_changed();
                    return self.current();
                }
                None
            }
            LoopState::Track => {
                #[cfg(feature = "mpris")]
                self.on_track_changed();
                self.current()
            }
            LoopState::Playlist => {
                let tracks = match self.shuffle_state {
                    ShuffleState::Off => &self.tracks,
                    ShuffleState::On => &self.shuffled_tracks,
                };
                if tracks.is_empty() {
                    None
                } else if !tracks.is_empty() && self.position < tracks.len() - 1 {
                    self.position += 1;
                    #[cfg(feature = "mpris")]
                    self.on_track_changed();
                    self.current()
                } else {
                    self.position = 0;
                    #[cfg(feature = "mpris")]
                    self.on_track_changed();
                    self.current()
                }
            }
        }
    }

    pub fn can_go_previous(&self) -> bool {
        match self.loop_state {
            LoopState::Off => match self.shuffle_state {
                ShuffleState::Off => !self.tracks.is_empty() && self.position > 0,
                ShuffleState::On => !self.shuffled_tracks.is_empty() && self.position > 0,
            },
            _ => match self.shuffle_state {
                ShuffleState::Off => !self.tracks.is_empty(),
                ShuffleState::On => !self.shuffled_tracks.is_empty(),
            },
        }
    }

    pub fn previous(&mut self) -> Option<&Track> {
        match self.loop_state {
            LoopState::Off => {
                let tracks = match self.shuffle_state {
                    ShuffleState::Off => &self.tracks,
                    ShuffleState::On => &self.shuffled_tracks,
                };
                if self.position > 0 && !tracks.is_empty() {
                    self.position -= 1;
                    #[cfg(feature = "mpris")]
                    self.on_track_changed();
                    self.current()
                } else {
                    None
                }
            }
            LoopState::Track => {
                #[cfg(feature = "mpris")]
                self.on_track_changed();
                self.current()
            }
            LoopState::Playlist => {
                let tracks = match self.shuffle_state {
                    ShuffleState::Off => &self.tracks,
                    ShuffleState::On => &self.shuffled_tracks,
                };
                if tracks.is_empty() {
                    None
                } else if self.position > 0 {
                    self.position -= 1;
                    #[cfg(feature = "mpris")]
                    self.on_track_changed();
                    self.current()
                } else if !tracks.is_empty() {
                    self.position = tracks.len() - 1;
                    #[cfg(feature = "mpris")]
                    self.on_track_changed();
                    self.current()
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerStateRaw {
    position: usize,
    player_position: Duration,
    player_volume: PlayerVolume,
    track_hashes: Vec<u64>,
    shuffled_track_hashes: Vec<u64>,
    shuffle_state: ShuffleState,
    loop_state: LoopState,
    playback_rate: PlaybackRate,
}

impl From<PlayerState> for PlayerStateRaw {
    fn from(value: PlayerState) -> Self {
        let track_hashes = Track::hash_tracks(&value.tracks);
        let shuffled_track_hashes = Track::hash_tracks(&value.shuffled_tracks);
        Self {
            position: value.position,
            player_position: value.player_position,
            player_volume: value.player_volume,
            track_hashes,
            shuffled_track_hashes,
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
            playback_rate: value.playback_rate,
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

pub(crate) fn unwrap_state_ref(
    maybe_state: Option<&PlayerState>,
) -> Result<&PlayerState, chilen_ipc::Error> {
    match maybe_state {
        Some(state) => Ok(state),
        None => Err(chilen_ipc::Error::StateNotInitialized),
    }
}

pub(crate) fn unwrap_state_mut(
    maybe_state: Option<&mut PlayerState>,
) -> Result<&mut PlayerState, chilen_ipc::Error> {
    match maybe_state {
        Some(state) => Ok(state),
        None => Err(chilen_ipc::Error::StateNotInitialized),
    }
}

fn save_state(state: PlayerState) -> Result<(), String> {
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

        match <PlayerStateRaw as TryInto<PlayerState>>::try_into(state_raw) {
            Ok(mut state) => {
                state.playback_state = PlaybackState::Stopped;
                Ok(state)
            }
            Err(e) => {
                error!("Could not restore player state from cache: {e}");
                Err(e.to_string())
            }
        }
    } else {
        Ok(PlayerState::default())
    }
}
