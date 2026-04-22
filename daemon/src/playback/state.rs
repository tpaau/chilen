use std::{
    fs::{File, read},
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
    thread,
    time::Duration,
};

use log::{error, trace};
use mpipc::{
    DaemonEvent, LoopState, MusicLibraryError, PlaybackError, PlaybackEvent, PlaybackRate,
    PlaybackState, PlayerVolume,
};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

#[cfg(feature = "shuffle")]
use mpipc::ShuffleState;
#[cfg(feature = "shuffle")]
use rand::seq::SliceRandom;

use crate::{
    data::{
        CACHE_DIR,
        music_lib::{Track, get_library, tracks_from_hashes},
    },
    send_event,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct PlayerState {
    pub position: usize,
    // TODO: Handle this properly
    pub player_position: Duration,
    pub player_volume: PlayerVolume,
    pub playback_state: PlaybackState,
    pub tracks: Vec<Track>,
    #[cfg(feature = "shuffle")]
    pub shuffled_tracks: Vec<Track>,
    #[cfg(feature = "shuffle")]
    pub shuffle_state: ShuffleState,
    pub loop_state: LoopState,
    pub playback_rate: PlaybackRate,
}

impl TryFrom<PlayerStateRaw> for PlayerState {
    type Error = MusicLibraryError;
    fn try_from(value: PlayerStateRaw) -> Result<Self, Self::Error> {
        let tracks = &get_library()?.tracks;
        Ok(Self {
            position: value.position,
            player_position: value.player_position,
            player_volume: value.player_volume,
            playback_state: PlaybackState::Stopped,
            tracks: tracks_from_hashes(value.track_hashes, tracks),
            #[cfg(feature = "shuffle")]
            shuffled_tracks: tracks_from_hashes(value.shuffled_track_hashes, tracks),
            #[cfg(feature = "shuffle")]
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
            playback_rate: value.playback_rate,
        })
    }
}

impl PlayerState {
    pub fn get_initial_events(&self) -> Vec<DaemonEvent> {
        vec![
            DaemonEvent::PlaybackEvent(PlaybackEvent::PlaybackStateChanged(self.playback_state)),
            DaemonEvent::PlaybackEvent(PlaybackEvent::LoopStateChanged(self.loop_state)),
            DaemonEvent::PlaybackEvent(PlaybackEvent::ShuffleStateChanged(self.shuffle_state)),
            DaemonEvent::PlaybackEvent(PlaybackEvent::QueueChanged(
                if self.shuffle_state == ShuffleState::On {
                    self.shuffled_tracks
                        .clone()
                        .into_iter()
                        .map(Into::into)
                        .collect()
                } else {
                    self.tracks.clone().into_iter().map(Into::into).collect()
                },
            )),
            DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(self.position)),
            DaemonEvent::PlaybackEvent(PlaybackEvent::PlayerPositionChanged(self.player_position)),
            DaemonEvent::PlaybackEvent(PlaybackEvent::PlayerVolumeChanged(self.player_volume)),
        ]
    }

    pub fn send_initial_events(&self) {
        trace!("Sending initial playback events to the daemon");
        for event in self.get_initial_events() {
            let _ = send_event(event);
        }
    }

    pub fn set_tracks(&mut self, tracks: Vec<Track>) {
        self.position = 0;
        self.tracks = tracks;
        #[cfg(feature = "shuffle")]
        self.shuffle();
    }

    pub fn append_tracks(&mut self, tracks: &mut Vec<Track>) {
        self.tracks.append(tracks);
        #[cfg(feature = "shuffle")]
        if self.shuffle_state == ShuffleState::On {
            self.shuffle();
        }
    }

    #[cfg(feature = "shuffle")]
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        let mut tracks = self.tracks.clone();
        tracks.shuffle(&mut rng);
        self.shuffled_tracks = tracks;
        if self.shuffle_state == ShuffleState::On {
            let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::QueueChanged(
                self.shuffled_tracks
                    .clone()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            )));
        }
    }

    #[cfg(feature = "shuffle")]
    pub fn set_shuffle_state(&mut self, shuffle_state: ShuffleState) {
        if self.shuffle_state != shuffle_state {
            self.shuffle_state = shuffle_state;
            let _ = send_event(DaemonEvent::PlaybackEvent(
                PlaybackEvent::ShuffleStateChanged(self.shuffle_state),
            ));
            let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::QueueChanged(
                if self.shuffle_state == ShuffleState::On {
                    self.shuffled_tracks
                        .clone()
                        .into_iter()
                        .map(Into::into)
                        .collect()
                } else {
                    self.tracks.clone().into_iter().map(Into::into).collect()
                },
            )));
        }
    }

    pub fn increment_player_position(&mut self, duration: Duration) {
        self.player_position += duration;
        let _ = send_event(DaemonEvent::PlaybackEvent(
            PlaybackEvent::PlayerPositionChanged(self.player_position),
        ));
    }

    pub fn set_player_position(&mut self, player_position: Duration) {
        if self.player_position != player_position {
            self.player_position = player_position;
            let _ = send_event(DaemonEvent::PlaybackEvent(
                PlaybackEvent::PlayerPositionChanged(self.player_position),
            ));
        }
    }

    pub fn set_player_volume(&mut self, player_volume: PlayerVolume) {
        if self.player_volume != player_volume {
            self.player_volume = player_volume;
            let _ = send_event(DaemonEvent::PlaybackEvent(
                PlaybackEvent::PlayerVolumeChanged(self.player_volume),
            ));
        }
    }

    pub fn set_loop_state(&mut self, loop_state: LoopState) {
        if self.loop_state != loop_state {
            self.loop_state = loop_state;
            let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::LoopStateChanged(
                self.loop_state,
            )));
        }
    }

    pub fn set_rate(&mut self, rate: f64) {
        if self.playback_rate.get_value() != rate {
            self.playback_rate.set_value(rate);
            let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::RateChanged(
                self.playback_rate,
            )));
        }
    }

    pub fn set_playback_state(&mut self, playback_state: PlaybackState) {
        if self.playback_state != playback_state {
            self.playback_state = playback_state;
            let _ = send_event(DaemonEvent::PlaybackEvent(
                PlaybackEvent::PlaybackStateChanged(self.playback_state),
            ));
        }
    }

    pub fn is_empty(&self) -> bool {
        #[cfg(feature = "shuffle")]
        match self.shuffle_state {
            ShuffleState::Off => self.tracks.is_empty(),
            ShuffleState::On => self.shuffled_tracks.is_empty(),
        }
        #[cfg(not(feature = "shuffle"))]
        self.tracks.is_empty()
    }

    pub fn current(&self) -> Option<&Track> {
        #[cfg(feature = "shuffle")]
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
        #[cfg(not(feature = "shuffle"))]
        if self.position < self.tracks.len() {
            return Some(&self.tracks[self.position]);
        }
        None
    }

    pub fn play_track(&mut self, index: usize) -> Option<&Track> {
        #[cfg(feature = "shuffle")]
        match self.shuffle_state {
            ShuffleState::Off => {
                if index < self.tracks.len() {
                    self.position = index;
                    let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(
                        self.position,
                    )));
                    return Some(&self.tracks[index]);
                }
            }
            ShuffleState::On => {
                if index < self.shuffled_tracks.len() {
                    self.position = index;
                    let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(
                        self.position,
                    )));
                    return Some(&self.shuffled_tracks[index]);
                }
            }
        }
        #[cfg(not(feature = "shuffle"))]
        if index < self.tracks.len() {
            self.position = index;
            let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(
                self.position,
            )));
            return Some(&self.tracks[index]);
        }
        None
    }

    #[cfg(feature = "mpris")]
    pub fn can_play(&self) -> bool {
        #[cfg(feature = "shuffle")]
        match self.shuffle_state {
            ShuffleState::Off => {
                self.position < self.tracks.len() && self.playback_state != PlaybackState::Playing
            }
            ShuffleState::On => {
                self.position < self.shuffled_tracks.len()
                    && self.playback_state != PlaybackState::Playing
            }
        }
        #[cfg(not(feature = "shuffle"))]
        return self.position < self.tracks.len() && self.playback_rate != PlaybackState::Playing;
    }

    pub fn can_go_next(&self) -> bool {
        match self.loop_state {
            LoopState::Off => {
                if cfg!(feature = "shuffle") {
                    match self.shuffle_state {
                        ShuffleState::Off => {
                            if self.tracks.is_empty() {
                                return false;
                            }
                            self.position < self.tracks.len() - 1
                        }
                        ShuffleState::On => {
                            if self.shuffled_tracks.is_empty() {
                                return false;
                            }
                            self.position < self.shuffled_tracks.len() - 1
                        }
                    }
                } else {
                    if self.tracks.is_empty() {
                        return false;
                    }
                    self.position < self.tracks.len() - 1
                }
            }
            _ => {
                #[cfg(feature = "shuffle")]
                match self.shuffle_state {
                    ShuffleState::Off => !self.tracks.is_empty(),
                    ShuffleState::On => !self.shuffled_tracks.is_empty(),
                }
                #[cfg(not(feature = "shuffle"))]
                !self.tracks.is_empty()
            }
        }
    }

    pub fn next(&mut self) -> Option<&Track> {
        match self.loop_state {
            LoopState::Off => {
                if cfg!(feature = "shuffle") {
                    match self.shuffle_state {
                        ShuffleState::Off => {
                            if self.position < self.tracks.len() - 1 {
                                self.position += 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                return self.current();
                            }
                            None
                        }
                        ShuffleState::On => {
                            if self.position < self.shuffled_tracks.len() - 1 {
                                self.position += 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                return self.current();
                            }
                            None
                        }
                    }
                } else {
                    if self.position < self.tracks.len() - 1 {
                        self.position += 1;
                        return self.current();
                    }
                    None
                }
            }
            LoopState::Track => {
                let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(
                    self.position,
                )));
                self.current()
            }
            LoopState::Playlist =>
            {
                #[cfg(feature = "shuffle")]
                if cfg!(feature = "shuffle") {
                    match self.shuffle_state {
                        ShuffleState::Off => {
                            if self.tracks.is_empty() {
                                None
                            } else if self.position < self.tracks.len() - 1 {
                                self.position += 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                self.current()
                            } else {
                                self.position = 0;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                self.current()
                            }
                        }
                        ShuffleState::On => {
                            if self.shuffled_tracks.is_empty() {
                                None
                            } else if self.position < self.shuffled_tracks.len() - 1 {
                                self.position += 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                self.current()
                            } else {
                                self.position = 0;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                self.current()
                            }
                        }
                    }
                } else if self.tracks.is_empty() {
                    None
                } else if self.position < self.tracks.len() - 1 {
                    self.position += 1;
                    let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(
                        self.position,
                    )));
                    self.current()
                } else {
                    self.position = 0;
                    let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(
                        self.position,
                    )));
                    self.current()
                }
            }
        }
    }

    pub fn can_go_previous(&self) -> bool {
        match self.loop_state {
            LoopState::Off => {
                if cfg!(feature = "shuffle") {
                    match self.shuffle_state {
                        ShuffleState::Off => {
                            if self.tracks.is_empty() {
                                return false;
                            }
                            self.position > 0
                        }
                        ShuffleState::On => {
                            if self.shuffled_tracks.is_empty() {
                                return false;
                            }
                            self.position > 0
                        }
                    }
                } else {
                    if self.tracks.is_empty() {
                        return false;
                    }
                    self.position > 0
                }
            }
            _ => {
                if cfg!(feature = "shuffle") {
                    match self.shuffle_state {
                        ShuffleState::Off => !self.tracks.is_empty(),
                        ShuffleState::On => !self.shuffled_tracks.is_empty(),
                    }
                } else {
                    !self.tracks.is_empty()
                }
            }
        }
    }

    pub fn previous(&mut self) -> Option<&Track> {
        match self.loop_state {
            LoopState::Off => {
                if cfg!(feature = "shuffle") {
                    match self.shuffle_state {
                        ShuffleState::Off => {
                            if self.position > 0 && !self.tracks.is_empty() {
                                self.position -= 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                return self.current();
                            }
                            None
                        }
                        ShuffleState::On => {
                            if self.position > 0 && !self.shuffled_tracks.is_empty() {
                                self.position -= 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                return self.current();
                            }
                            None
                        }
                    }
                } else {
                    if self.position > 0 && !self.tracks.is_empty() {
                        self.position -= 1;
                        let _ = send_event(DaemonEvent::PlaybackEvent(
                            PlaybackEvent::PositionChanged(self.position),
                        ));
                        return self.current();
                    }
                    None
                }
            }
            LoopState::Track => {
                let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(
                    self.position,
                )));
                self.current()
            }
            LoopState::Playlist => {
                if cfg!(feature = "shuffle") {
                    match self.shuffle_state {
                        ShuffleState::Off => {
                            if self.tracks.is_empty() {
                                None
                            } else if self.position > 0 {
                                self.position -= 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                self.current()
                            } else {
                                self.position = self.tracks.len() - 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                self.current()
                            }
                        }
                        ShuffleState::On => {
                            if self.shuffled_tracks.is_empty() {
                                None
                            } else if self.position > 0 {
                                self.position -= 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                self.current()
                            } else {
                                self.position = self.shuffled_tracks.len() - 1;
                                let _ = send_event(DaemonEvent::PlaybackEvent(
                                    PlaybackEvent::PositionChanged(self.position),
                                ));
                                self.current()
                            }
                        }
                    }
                } else if self.tracks.is_empty() {
                    None
                } else if self.position > 0 {
                    self.position -= 1;
                    let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(
                        self.position,
                    )));
                    self.current()
                } else {
                    self.position = self.tracks.len() - 1;
                    let _ = send_event(DaemonEvent::PlaybackEvent(PlaybackEvent::PositionChanged(
                        self.position,
                    )));
                    self.current()
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
    #[cfg(feature = "shuffle")]
    shuffled_track_hashes: Vec<u64>,
    #[cfg(feature = "shuffle")]
    shuffle_state: ShuffleState,
    loop_state: LoopState,
    playback_rate: PlaybackRate,
}

impl From<PlayerState> for PlayerStateRaw {
    fn from(value: PlayerState) -> Self {
        let track_hashes = Track::hash_tracks(&value.tracks);
        #[cfg(feature = "shuffle")]
        let shuffled_track_hashes = Track::hash_tracks(&value.shuffled_tracks);
        Self {
            position: value.position,
            player_position: value.player_position,
            player_volume: value.player_volume,
            track_hashes,
            #[cfg(feature = "shuffle")]
            shuffled_track_hashes,
            #[cfg(feature = "shuffle")]
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
) -> Result<&PlayerState, PlaybackError> {
    match maybe_state {
        Some(state) => Ok(state),
        None => Err(PlaybackError::StateNotInitialized),
    }
}

pub(crate) fn unwrap_state_mut(
    maybe_state: Option<&mut PlayerState>,
) -> Result<&mut PlayerState, PlaybackError> {
    match maybe_state {
        Some(state) => Ok(state),
        None => Err(PlaybackError::StateNotInitialized),
    }
}

fn save_state(state: PlayerState) -> Result<(), MusicLibraryError> {
    let state_raw: PlayerStateRaw = state.into();
    let state_file = STATE_FILE.clone();

    let mut data = Vec::new();
    if let Err(e) = state_raw.serialize(&mut Serializer::new(&mut data)) {
        error!("Could not serialize the player state: {e}");
        return Err(MusicLibraryError::CacheError);
    }

    let mut file = match File::create(state_file) {
        Ok(file) => file,
        Err(e) => {
            error!("Could not open the player state cache in write-only mode: {e}");
            return Err(MusicLibraryError::CacheError);
        }
    };

    match file.write_all(&data) {
        Ok(_) => {
            trace!("Saved player state to cache");
            Ok(())
        }
        Err(e) => {
            error!("Could not write to the player state cache: {e}");
            Err(MusicLibraryError::CacheError)
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

pub(crate) fn restore_state_from_cache() -> Result<PlayerState, MusicLibraryError> {
    let state_file = STATE_FILE.clone();

    trace!("Restoring player state from {state_file:?}");

    let state_exists = match state_file.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            error!("Could not check if the player state file exists: {e}");
            return Err(MusicLibraryError::CacheError);
        }
    };

    if state_exists {
        let data = match read(state_file) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not read the player state cache: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        let state_raw = match PlayerStateRaw::deserialize(&mut Deserializer::from_read_ref(&data)) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not decode the contents of the player state file: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        match state_raw.try_into() {
            Ok(state) => Ok(state),
            Err(e) => {
                error!("Could not restore player state from cache: {e}");
                Err(e)
            }
        }
    } else {
        Ok(PlayerState::default())
    }
}
