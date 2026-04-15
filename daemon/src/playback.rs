use std::{
    fs::{File, read},
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
    thread,
    time::Duration,
};

use log::{debug, error, info, trace, warn};
#[cfg(feature = "shuffle")]
use mpipc::ShuffleState;
use mpipc::{
    DaemonEvent, LoopState, MusicLibraryError, PlaybackError, PlaybackEvent, PlaybackState,
    PlayerVolume,
};
#[cfg(feature = "shuffle")]
use rand::seq::SliceRandom;
use rmp_serde::{Deserializer, Serializer};
use rodio::Player;
use serde::{Deserialize, Serialize};

use crate::{
    data::{
        CACHE_DIR,
        cache::indexer::index_files,
        music_lib::{Track, get_library, tracks_from_hashes},
    },
    send_event,
};

#[derive(Debug, Clone, Default)]
struct PlayerState {
    position: usize,
    player_position: Duration,
    player_volume: PlayerVolume,
    playback_state: PlaybackState,
    tracks: Vec<Track>,
    #[cfg(feature = "shuffle")]
    shuffled_tracks: Vec<Track>,
    #[cfg(feature = "shuffle")]
    shuffle_state: ShuffleState,
    loop_state: LoopState,
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
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Command {
    Play(Option<usize>),
    Pause,
    SetQueue(Vec<Track>),
    AppendToQueue(Vec<Track>),
    Next,
    Previous,
    SetLoopState(LoopState),
    #[cfg(feature = "shuffle")]
    SetShuffleState(ShuffleState),
    #[cfg(not(feature = "shuffle"))]
    SetShuffleState,
    SetPlayerPosition(Duration),
    SetPlayerVolume(PlayerVolume),
}

impl TryFrom<mpipc::PlaybackCommand> for Command {
    type Error = MusicLibraryError;
    fn try_from(value: mpipc::PlaybackCommand) -> Result<Self, Self::Error> {
        match value {
            mpipc::PlaybackCommand::Play(maybe_pos) => Ok(Self::Play(maybe_pos)),
            mpipc::PlaybackCommand::Pause => Ok(Self::Pause),
            mpipc::PlaybackCommand::SetQueue(track_paths) => {
                let tracks = get_library()?.tracks;
                let indexed_tracks = index_files(track_paths, false)?;
                let track_hashes = Track::hash_tracks(&indexed_tracks);
                let filtered_tracks = tracks_from_hashes(track_hashes, &tracks);
                Ok(Self::SetQueue(filtered_tracks))
            }
            mpipc::PlaybackCommand::AppendToQueue(track_paths) => {
                let tracks = get_library()?.tracks;
                let indexed_tracks = index_files(track_paths, false)?;
                let track_hashes = Track::hash_tracks(&indexed_tracks);
                let filtered_tracks = tracks_from_hashes(track_hashes, &tracks);
                Ok(Self::AppendToQueue(filtered_tracks))
            }
            mpipc::PlaybackCommand::SetPlaylist(playlist_name) => {
                let lib = get_library()?;
                let playlist = match lib.playlists.iter().find(|p| p.name == playlist_name) {
                    Some(playlist) => playlist,
                    None => return Err(MusicLibraryError::NoSuchPlaylist),
                };
                Ok(Self::SetQueue(playlist.tracks.clone()))
            }
            mpipc::PlaybackCommand::AppendPlaylist(playlist_name) => {
                let lib = get_library()?;
                let playlist = match lib.playlists.iter().find(|p| p.name == playlist_name) {
                    Some(playlist) => playlist,
                    None => return Err(MusicLibraryError::NoSuchPlaylist),
                };
                Ok(Self::AppendToQueue(playlist.tracks.clone()))
            }
            mpipc::PlaybackCommand::Next => Ok(Self::Next),
            mpipc::PlaybackCommand::Previous => Ok(Self::Previous),
            mpipc::PlaybackCommand::SetLoopState(loop_state) => Ok(Self::SetLoopState(loop_state)),
            #[cfg(feature = "shuffle")]
            mpipc::PlaybackCommand::SetShuffleState(shuffle_state) => {
                Ok(Self::SetShuffleState(shuffle_state))
            }
            #[cfg(not(feature = "shuffle"))]
            mpipc::PlaybackCommand::SetShuffleState(_) => Ok(Self::SetShuffleState),
            mpipc::PlaybackCommand::SetPlayerPosition(position) => {
                Ok(Self::SetPlayerPosition(position))
            }
            mpipc::PlaybackCommand::SetPlayerVolume(volume) => Ok(Self::SetPlayerVolume(volume)),
        }
    }
}

static STATE_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data = CACHE_DIR.read().unwrap().clone().unwrap();
    data.push("player_state");
    data
});

static PLAYER_HANDLE: LazyLock<Arc<RwLock<Option<rodio::Player>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static PLAYER_STATE: LazyLock<Arc<RwLock<Option<PlayerState>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

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

fn background_save_state(state: PlayerState) {
    thread::spawn(|| {
        if let Err(e) = save_state(state) {
            error!("Could not save player state to cache: {e}");
        }
    });
}

fn restore_state_from_cache() -> Result<PlayerState, MusicLibraryError> {
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

pub(crate) fn get_initial_events() -> Vec<DaemonEvent> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = state_guard.as_ref().unwrap();
    state.get_initial_events()
}

pub(crate) fn init() {
    trace!("Initializing the playback module");

    let state = match restore_state_from_cache() {
        Ok(state) => {
            debug!("Restored player state from cache");
            state
        }
        Err(e) => {
            error!("Could not restore player state from cache: {e}");
            let state = PlayerState::default();
            debug!("Creating a new state and attempting to save it in cache");
            background_save_state(state.clone());
            state
        }
    };
    trace!("Player state ready!");

    state.send_initial_events();

    let handle = match rodio::DeviceSinkBuilder::open_default_sink() {
        Ok(sink) => sink,
        Err(e) => {
            error!("Could not open the default sink, audio playback will not work! Error: {e}");
            return;
        }
    };
    let player = Player::connect_new(handle.mixer());
    player.set_volume(state.player_volume.get());

    *PLAYER_STATE.write().unwrap() = Some(state);
    *PLAYER_HANDLE.write().unwrap() = Some(player);

    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = state_guard.as_mut().unwrap();
    if let Some(track) = state.current()
        && let Ok(source) = track.open_source()
    {
        let player_guard = PLAYER_HANDLE.read().unwrap();
        let player = player_guard.as_ref().unwrap();
        player.append(source);
        player.pause();
        drop(player_guard);
    }
    drop(state_guard);

    let mut initial_iter = true;
    let sleep_duration = Duration::from_millis(100);
    loop {
        thread::sleep(sleep_duration);
        let mut state_guard = PLAYER_STATE.write().unwrap();
        let state = state_guard.as_mut().unwrap();
        let player_guard = PLAYER_HANDLE.read().unwrap();
        let player = player_guard.as_ref().unwrap();
        if !player.is_paused() && !player.empty() {
            state.increment_player_position(sleep_duration);
        } else if player.empty() {
            state.set_player_position(Duration::default());
            if !state.can_go_next() {
                state.set_playback_state(PlaybackState::Stopped);
                continue;
            }
            let source = match state.next().unwrap().open_source() {
                Ok(source) => source,
                Err(e) => {
                    error!("Could not open audio source: {e}");
                    continue;
                }
            };
            player.append(source);
            if initial_iter {
                player.pause();
                initial_iter = false;
                state.set_playback_state(PlaybackState::Paused);
            } else {
                state.set_playback_state(PlaybackState::Playing);
            }
        }
        drop(state_guard);
        drop(player_guard);
    }
}

pub(crate) fn run_command(cmd: Command) -> Result<(), PlaybackError> {
    match cmd {
        Command::Play(maybe_pos) => {
            // trace!("Playing the current media");
            let player_guard = PLAYER_HANDLE.read().unwrap();
            if let Some(player) = player_guard.as_ref() {
                let mut state_guard = PLAYER_STATE.write().unwrap();
                let state = state_guard.as_mut().unwrap();
                match maybe_pos {
                    Some(pos) => {
                        let track = match state.play_track(pos) {
                            Some(track) => track,
                            None => {
                                error!("No track at index {pos}");
                                return Err(PlaybackError::NoTrackAtIndex(pos));
                            }
                        };
                        let source = match track.open_source() {
                            Ok(source) => source,
                            Err(e) => {
                                error!("Could not open audio source: {e}");
                                return Err(PlaybackError::SourceError);
                            }
                        };
                        player.empty();
                        player.append(source);
                        player.play();
                        state.set_playback_state(PlaybackState::Playing);
                    }
                    None => {
                        if !player.is_paused() && !player.empty() {
                            return Err(PlaybackError::PlayerPlaying);
                        } else if player.empty() {
                            if let Some(track) = state.current() {
                                let source = match track.open_source() {
                                    Ok(source) => source,
                                    Err(e) => {
                                        error!("Could not open audio source: {e}");
                                        return Err(PlaybackError::SourceError);
                                    }
                                };
                                player.append(source);
                                state.set_playback_state(PlaybackState::Playing);
                            } else {
                                return Err(PlaybackError::QueueEmpty);
                            }
                        } else {
                            player.play();
                            state.set_playback_state(PlaybackState::Playing);
                        }
                    }
                }
            } else {
                warn!("Cannot play, player is not connected");
                return Err(PlaybackError::PlayerNotConnected);
            }
        }
        Command::Pause => {
            trace!("Pausing the current media");
            let player_guard = PLAYER_HANDLE.read().unwrap();
            let player = player_guard.as_ref().unwrap();
            if player.is_paused() {
                return Err(PlaybackError::PlayerPaused);
            } else {
                player.pause();
                let mut state_guard = PLAYER_STATE.write().unwrap();
                let state = state_guard.as_mut().unwrap();
                state.set_playback_state(PlaybackState::Paused);
            }
        }
        Command::SetQueue(queue) => {
            trace!("Setting a new queue");
            let mut state_guard = PLAYER_STATE.write().unwrap();
            let state = state_guard.as_mut().unwrap();
            state.set_tracks(queue);
            background_save_state(state.clone());
        }
        Command::AppendToQueue(mut queue) => {
            trace!("Appending tracks to queue");
            let mut state_guard = PLAYER_STATE.write().unwrap();
            let state = state_guard.as_mut().unwrap();
            state.append_tracks(&mut queue);
            background_save_state(state.clone());
        }
        Command::Next => {
            trace!("Skipping to the next track");
            let mut state_guard = PLAYER_STATE.write().unwrap();
            let state = state_guard.as_mut().unwrap();
            if state.can_go_next() {
                let track = state.next().unwrap().clone();
                background_save_state(state.clone());
                let source = match track.open_source() {
                    Ok(source) => source,
                    Err(e) => {
                        error!("Could not open audio source: {e}");
                        return Err(PlaybackError::SourceError);
                    }
                };
                let player_guard = PLAYER_HANDLE.read().unwrap();
                if let Some(player) = player_guard.as_ref() {
                    state.set_player_position(Duration::default());
                    player.clear();
                    player.append(source);
                    player.play();
                } else {
                    warn!("Cannot skip to the next track, player is not connected");
                    return Err(PlaybackError::PlayerNotConnected);
                }
            } else if state.is_empty() {
                info!("Cannot skip to the next track, queue is empty");
                return Err(PlaybackError::QueueEmpty);
            } else {
                info!("Cannot skip to the next track");
                return Err(PlaybackError::CannotGoNext);
            }
        }
        Command::Previous => {
            trace!("Skipping to the previous track");
            let mut state_guard = PLAYER_STATE.write().unwrap();
            let state = state_guard.as_mut().unwrap();
            if state.can_go_previous() {
                let track = state.previous().unwrap().clone();
                background_save_state(state.clone());
                let source = match track.open_source() {
                    Ok(source) => source,
                    Err(e) => {
                        error!("Could not open audio source: {e}");
                        return Err(PlaybackError::SourceError);
                    }
                };
                let player_guard = PLAYER_HANDLE.read().unwrap();
                if let Some(player) = player_guard.as_ref() {
                    state.set_player_position(Duration::default());
                    player.clear();
                    player.append(source);
                    player.play();
                } else {
                    warn!("Cannot skip to the previous track, player is not connected");
                    return Err(PlaybackError::PlayerNotConnected);
                }
            } else if state.is_empty() {
                info!("Cannot go to the previous track, queue is empty");
                return Err(PlaybackError::QueueEmpty);
            } else {
                info!("Cannot go to the previous track");
                return Err(PlaybackError::CannotGoPrevious);
            }
        }
        Command::SetLoopState(loop_state) => {
            trace!("Setting loop state to {loop_state:?}");
            let mut state_guard = PLAYER_STATE.write().unwrap();
            let state = state_guard.as_mut().unwrap();
            state.set_loop_state(loop_state);
            background_save_state(state.clone());
        }
        #[cfg(feature = "shuffle")]
        Command::SetShuffleState(shuffle_state) => {
            trace!("Setting shuffle state to {shuffle_state:?}");
            let mut state_guard = PLAYER_STATE.write().unwrap();
            let state = state_guard.as_mut().unwrap();
            state.set_shuffle_state(shuffle_state);
            state.shuffle();
            background_save_state(state.clone());
            #[cfg(not(feature = "shuffle"))]
            return Err(PlaybackError::ShuffleNotSupported);
        }
        #[cfg(not(feature = "shuffle"))]
        Command::SetShuffleState => {
            return Err(PlaybackError::ShuffleNotSupported);
        }
        Command::SetPlayerPosition(position) => {
            trace!("Setting player position to {:?}", position.as_secs());
            let player_guard = PLAYER_HANDLE.read().unwrap();
            if let Some(player) = player_guard.as_ref() {
                if player.empty() {
                    return Err(PlaybackError::QueueEmpty);
                }
                if let Err(e) = player.try_seek(position) {
                    error!("Could not seek: {e}");
                    return Err(PlaybackError::SeekNotSupported);
                } else {
                    let mut state_guard = PLAYER_STATE.write().unwrap();
                    let state = state_guard.as_mut().unwrap();
                    state.set_player_position(position);
                }
            } else {
                warn!("Cannot seek, player is not connected");
                return Err(PlaybackError::PlayerNotConnected);
            }
        }
        Command::SetPlayerVolume(volume) => {
            // trace!("Setting position to {:?}", position.as_secs());
            let player_guard = PLAYER_HANDLE.read().unwrap();
            if let Some(player) = player_guard.as_ref() {
                player.set_volume(volume.get());
                let mut state_guard = PLAYER_STATE.write().unwrap();
                let state = state_guard.as_mut().unwrap();
                state.set_player_volume(volume);
            } else {
                warn!("Cannot set player volume, the player is not connected");
                return Err(PlaybackError::PlayerNotConnected);
            }
        }
    }

    Ok(())
}
