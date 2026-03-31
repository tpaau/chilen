use std::{
    fs::{File, read},
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
    thread,
    time::Duration,
};

use log::{debug, error, info, trace, warn};
use mpipc::{LoopState, MusicLibraryError, PlaybackError, ShuffleState};
use rmp_serde::{Deserializer, Serializer};
use rodio::Player;
use serde::{Deserialize, Serialize};

use crate::data::{
    CACHE_DIR,
    cache::indexer::index_files,
    music_lib::{Track, get_library, tracks_from_hashes},
};

#[derive(Debug, Clone, Default)]
struct QueueState {
    position: usize,
    player_position: Duration,
    tracks: Vec<Track>,
    shuffle_state: ShuffleState,
    loop_state: LoopState,
}

impl TryFrom<QueueStateRaw> for QueueState {
    type Error = MusicLibraryError;
    fn try_from(value: QueueStateRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            position: value.position,
            player_position: value.player_position,
            tracks: tracks_from_hashes(value.track_hashes, &get_library()?.tracks),
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
        })
    }
}

impl QueueState {
    pub fn set_tracks(&mut self, tracks: Vec<Track>) {
        self.position = 0;
        self.tracks = tracks;
    }

    pub fn append_tracks(&mut self, tracks: &mut Vec<Track>) {
        self.tracks.append(tracks);
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn pos(&self) -> usize {
        self.position
    }

    pub fn current(&self) -> Option<&Track> {
        if self.position < self.len() {
            return Some(&self.tracks[self.position]);
        }
        None
    }

    pub fn can_go_next(&self) -> bool {
        match self.loop_state {
            LoopState::Off => {
                if self.tracks.is_empty() {
                    return false;
                }
                self.pos() < self.len() - 1
            }
            _ => true,
        }
    }

    pub fn next(&mut self) -> Option<&Track> {
        match self.loop_state {
            LoopState::Off => match self.shuffle_state {
                ShuffleState::Off => {
                    if self.position < self.len() - 1 {
                        self.position += 1;
                        return self.current();
                    }
                    None
                }
                ShuffleState::On => {
                    todo!("Implement shuffle")
                }
            },
            LoopState::Track => self.current(),
            LoopState::Playlist => match self.shuffle_state {
                ShuffleState::Off => {
                    if self.position < self.len() - 1 {
                        self.position += 1;
                        self.current()
                    } else {
                        self.position = 0;
                        self.current()
                    }
                }
                ShuffleState::On => {
                    todo!("Implement shuffle")
                }
            },
        }
    }

    pub fn can_go_previous(&self) -> bool {
        trace!("Going previous");
        trace!("Position: {}", self.pos());
        trace!("Length: {}", self.len());
        if self.tracks.is_empty() {
            return false;
        }
        self.pos() > 0
    }

    pub fn previous(&mut self) -> Option<&Track> {
        match self.loop_state {
            LoopState::Off => {
                if self.pos() > 0 && !self.tracks.is_empty() {
                    self.position -= 1;
                    return self.current();
                }
                None
            }
            LoopState::Track => self.current(),
            LoopState::Playlist => {
                if self.tracks.is_empty() {
                    return None;
                }
                if self.pos() > 0 {
                    self.position -= 1;
                    self.current()
                } else {
                    self.position = self.len() - 1;
                    self.current()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueueStateRaw {
    position: usize,
    player_position: Duration,
    track_hashes: Vec<u64>,
    shuffle_state: ShuffleState,
    loop_state: LoopState,
}

impl From<QueueState> for QueueStateRaw {
    fn from(value: QueueState) -> Self {
        let track_hashes = Track::hash_tracks(&value.tracks);
        Self {
            position: value.position,
            player_position: value.player_position,
            track_hashes,
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Command {
    Play,
    Pause,
    SetQueue(Vec<Track>),
    AppendToQueue(Vec<Track>),
    Next,
    Previous,
    SetLoopState(LoopState),
    SetShuffleState(ShuffleState),
    SetPosition(Duration),
}

impl TryFrom<mpipc::PlaybackCommand> for Command {
    type Error = MusicLibraryError;
    fn try_from(value: mpipc::PlaybackCommand) -> Result<Self, Self::Error> {
        match value {
            mpipc::PlaybackCommand::Play => Ok(Self::Play),
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
            mpipc::PlaybackCommand::Next => Ok(Self::Next),
            mpipc::PlaybackCommand::Previous => Ok(Self::Previous),
            mpipc::PlaybackCommand::SetLoopState(loop_state) => Ok(Self::SetLoopState(loop_state)),
            mpipc::PlaybackCommand::SetShuffleState(shuffle_state) => {
                Ok(Self::SetShuffleState(shuffle_state))
            }
            mpipc::PlaybackCommand::SetPosition(position) => Ok(Self::SetPosition(position)),
        }
    }
}

static STATE_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data = CACHE_DIR.read().unwrap().clone().unwrap();
    data.push("queue_state");
    data
});

static PLAYER_HANDLE: LazyLock<Arc<RwLock<Option<rodio::Player>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static QUEUE_STATE: LazyLock<Arc<RwLock<Option<QueueState>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

fn save_state(state: QueueState) -> Result<(), MusicLibraryError> {
    let state_raw: QueueStateRaw = state.into();
    let state_file = STATE_FILE.clone();

    let mut data = Vec::new();
    if let Err(e) = state_raw.serialize(&mut Serializer::new(&mut data)) {
        error!("Could not serialize the queue state: {e}");
        return Err(MusicLibraryError::CacheError);
    }

    let mut file = match File::create(state_file) {
        Ok(file) => file,
        Err(e) => {
            error!("Could not open the queue state cache in write-only mode: {e}");
            return Err(MusicLibraryError::CacheError);
        }
    };

    match file.write_all(&data) {
        Ok(_) => {
            trace!("Saved queue state to cache");
            Ok(())
        }
        Err(e) => {
            error!("Could not write to the queue state cache: {e}");
            Err(MusicLibraryError::CacheError)
        }
    }
}

fn background_save_state(state: QueueState) {
    thread::spawn(|| {
        if let Err(e) = save_state(state) {
            error!("Could not save queue state to cache: {e}")
        }
    });
}

fn restore_state_from_cache() -> Result<QueueState, MusicLibraryError> {
    let state_file = STATE_FILE.clone();

    trace!("Restoring queue state from {state_file:?}");

    let state_exists = match state_file.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            error!("Could not check if the queue state file exists: {e}");
            return Err(MusicLibraryError::CacheError);
        }
    };

    if state_exists {
        let data = match read(state_file) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not read the queue state cache: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        let state_raw = match QueueStateRaw::deserialize(&mut Deserializer::from_read_ref(&data)) {
            Ok(data) => data,
            Err(e) => {
                error!("Could not decode the contents of the queue state cache: {e}");
                return Err(MusicLibraryError::CacheError);
            }
        };

        match state_raw.try_into() {
            Ok(state) => Ok(state),
            Err(e) => {
                error!("Could not restore queue state from cache: {e}");
                Err(e)
            }
        }
    } else {
        Ok(QueueState::default())
    }
}

pub(crate) fn init() {
    trace!("Initializing the playback module");

    let state = match restore_state_from_cache() {
        Ok(state) => {
            debug!("Restored queue state from cache");
            state
        }
        Err(e) => {
            error!("Could not restore queue state from cache: {e}");
            let state = QueueState::default();
            debug!("Creating a new state and attempting to save it in cache");
            background_save_state(state.clone());
            state
        }
    };
    trace!("Queue state ready!");

    let handle = match rodio::DeviceSinkBuilder::open_default_sink() {
        Ok(sink) => sink,
        Err(e) => {
            error!("Could not open the default sink, audio playback will not work! Error: {e}");
            return;
        }
    };
    let player = Player::connect_new(handle.mixer());

    *QUEUE_STATE.write().unwrap() = Some(state);
    *PLAYER_HANDLE.write().unwrap() = Some(player);

    let mut queue_guard = QUEUE_STATE.write().unwrap();
    let state = queue_guard.as_mut().unwrap();
    if let Some(track) = state.current()
        && let Ok(source) = track.open_source()
    {
        let player_guard = PLAYER_HANDLE.read().unwrap();
        let player = player_guard.as_ref().unwrap();
        player.append(source);
        player.pause();
        drop(player_guard);
    }
    drop(queue_guard);

    let mut initial_iter = true;
    let sleep_duration = Duration::from_millis(100);
    loop {
        thread::sleep(sleep_duration);
        let mut queue_guard = QUEUE_STATE.write().unwrap();
        let state = queue_guard.as_mut().unwrap();
        let player_guard = PLAYER_HANDLE.read().unwrap();
        let player = player_guard.as_ref().unwrap();
        if !player.is_paused() && !player.empty() {
            state.player_position += sleep_duration;
        } else if player.empty() {
            state.player_position = Duration::default();
            if !state.can_go_next() {
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
            }
        }
        drop(queue_guard);
        drop(player_guard);
    }
}

pub(crate) fn run_command(cmd: Command) -> Result<(), PlaybackError> {
    match cmd {
        Command::Play => {
            trace!("Playing the current media");
            let player_guard = PLAYER_HANDLE.read().unwrap();
            if let Some(player) = player_guard.as_ref() {
                let mut queue_guard = QUEUE_STATE.write().unwrap();
                let state = queue_guard.as_mut().unwrap();
                if !player.is_paused() && !player.empty() {
                    return Err(PlaybackError::PlayerPlaying);
                } else if player.empty() {
                    if let Some(track) = state.current() {
                        let source = match track.open_source() {
                            Ok(source) => source,
                            Err(e) => {
                                error!("Could not open audio source: {e}");
                                return Err(PlaybackError::AudioSourceError);
                            }
                        };
                        player.append(source);
                    } else {
                        return Err(PlaybackError::NoTracksInQueue);
                    }
                } else {
                    player.play();
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
            }
        }
        Command::SetQueue(queue) => {
            trace!("Setting a new queue");
            let mut queue_guard = QUEUE_STATE.write().unwrap();
            let state = queue_guard.as_mut().unwrap();
            state.set_tracks(queue);
            background_save_state(state.clone());
        }
        Command::AppendToQueue(mut queue) => {
            trace!("Appending tracks to queue");
            let mut queue_guard = QUEUE_STATE.write().unwrap();
            let state = queue_guard.as_mut().unwrap();
            state.append_tracks(&mut queue);
            background_save_state(state.clone());
        }
        Command::Next => {
            trace!("Skipping to the next track");
            let mut queue_guard = QUEUE_STATE.write().unwrap();
            let state = queue_guard.as_mut().unwrap();
            if state.can_go_next() {
                let track = state.next().unwrap().clone();
                background_save_state(state.clone());
                let source = match track.open_source() {
                    Ok(source) => source,
                    Err(e) => {
                        error!("Could not open audio source: {e}");
                        todo!()
                    }
                };
                let player_guard = PLAYER_HANDLE.read().unwrap();
                if let Some(player) = player_guard.as_ref() {
                    state.player_position = Duration::default();
                    player.clear();
                    player.append(source);
                    player.play();
                } else {
                    warn!("Cannot skip to the next track, player is not connected");
                    return Err(PlaybackError::PlayerNotConnected);
                }
            } else {
                info!("Cannot skip to the next track, the current track is last in the queue");
            }
        }
        Command::Previous => {
            trace!("Skipping to the previous track");
            let mut queue_guard = QUEUE_STATE.write().unwrap();
            let state = queue_guard.as_mut().unwrap();
            if state.can_go_previous() {
                let track = state.previous().unwrap().clone();
                background_save_state(state.clone());
                let source = match track.open_source() {
                    Ok(source) => source,
                    Err(e) => {
                        error!("Could not open audio source: {e}");
                        todo!();
                    }
                };
                let player_guard = PLAYER_HANDLE.read().unwrap();
                if let Some(player) = player_guard.as_ref() {
                    state.player_position = Duration::default();
                    player.clear();
                    player.append(source);
                    player.play();
                } else {
                    warn!("Cannot skip to the previous track, player is not connected");
                    return Err(PlaybackError::PlayerNotConnected);
                }
            } else {
                info!("Cannot go the the previous track");
            }
        }
        Command::SetLoopState(loop_state) => {
            trace!("Setting loop state to {loop_state:?}");
            let mut queue_guard = QUEUE_STATE.write().unwrap();
            let state = queue_guard.as_mut().unwrap();
            state.loop_state = loop_state;
            background_save_state(state.clone());
        }
        Command::SetShuffleState(shuffle_state) => {
            trace!("Setting shuffle state to {shuffle_state:?}");
            let mut queue_guard = QUEUE_STATE.write().unwrap();
            let state = queue_guard.as_mut().unwrap();
            state.shuffle_state = shuffle_state;
            background_save_state(state.clone());
        }
        Command::SetPosition(position) => {
            trace!("Setting position to {:?}", position.as_secs());
            let player_guard = PLAYER_HANDLE.read().unwrap();
            if let Some(player) = player_guard.as_ref() {
                if player.empty() {
                    return Err(PlaybackError::NoTracksInQueue);
                }
                if let Err(e) = player.try_seek(position) {
                    error!("Could not seek: {e}");
                    return Err(PlaybackError::SeekNotSupported);
                } else {
                    let mut queue_guard = QUEUE_STATE.write().unwrap();
                    let state = queue_guard.as_mut().unwrap();
                    state.player_position = position;
                }
            } else {
                warn!("Cannot seek, player is not connected");
                return Err(PlaybackError::PlayerNotConnected);
            }
        }
    }

    Ok(())
}
