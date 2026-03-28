use std::{
    fs::{File, read},
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock, mpsc},
    thread,
    time::Duration,
};

use log::{debug, error, trace};
use mpipc::{LoopState, MusicLibraryError, ShuffleState};
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
    tracks: Vec<Track>,
    shuffle_state: ShuffleState,
    loop_state: LoopState,
}

impl TryFrom<QueueStateRaw> for QueueState {
    type Error = MusicLibraryError;
    fn try_from(value: QueueStateRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            position: value.position,
            tracks: tracks_from_hashes(value.track_hashes, &get_library()?.tracks),
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
        })
    }
}

impl QueueState {
    pub fn set_tracks(&mut self, tracks: Vec<Track>) {
        self.tracks = tracks;
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn pos(&self) -> usize {
        self.position
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueueStateRaw {
    position: usize,
    track_hashes: Vec<u64>,
    shuffle_state: ShuffleState,
    loop_state: LoopState,
}

impl From<QueueState> for QueueStateRaw {
    fn from(value: QueueState) -> Self {
        let track_hashes = Track::hash_tracks(&value.tracks);
        Self {
            position: value.position,
            track_hashes,
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
        }
    }
}

impl QueueState {
    pub fn current(&self) -> Option<&Track> {
        if self.position <= self.tracks.len() {
            return Some(&self.tracks[self.position - 1]);
        }
        None
    }

    pub fn can_go_next(&self) -> bool {
        match self.loop_state {
            LoopState::Off => {
                if self.tracks.is_empty() {
                    return false;
                }
                self.position < self.tracks.len()
            }
            _ => true,
        }
    }

    pub fn next(&mut self) -> Option<&Track> {
        match self.loop_state {
            LoopState::Off => match self.shuffle_state {
                ShuffleState::Off => {
                    if self.position < self.tracks.len() {
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
                    if self.position < self.tracks.len() {
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
}

#[derive(Debug, Clone)]
pub(crate) enum Command {
    Play,
    Pause,
    SetQueue(Vec<Track>),
    Next,
    Previous,
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
                Ok(Command::SetQueue(filtered_tracks))
            }
            mpipc::PlaybackCommand::SetPlaylist(playlist_name) => {
                let lib = get_library()?;
                let playlist = match lib.playlists.iter().find(|p| p.name == playlist_name) {
                    Some(playlist) => playlist,
                    None => return Err(MusicLibraryError::NoSuchPlaylist),
                };
                Ok(Command::SetQueue(playlist.tracks.clone()))
            }
            mpipc::PlaybackCommand::Next => Ok(Self::Next),
            mpipc::PlaybackCommand::Previous => Ok(Self::Previous),
        }
    }
}

static STATE_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data = CACHE_DIR.read().unwrap().clone().unwrap();
    data.push("queue_state");
    data
});

static COMMAND_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Command>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

fn save_state_to_cache(state: QueueState) -> Result<(), MusicLibraryError> {
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
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Could not write to the queue state cache: {e}");
            Err(MusicLibraryError::CacheError)
        }
    }
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

pub(crate) fn send_command(cmd: Command) -> Result<(), String> {
    trace!("Sending a command to the playback thread: {cmd:?}");
    match COMMAND_SENDER.read().as_mut() {
        Ok(guard) => match guard.clone().unwrap().send(cmd) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
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
            if let Err(e) = save_state_to_cache(state.clone()) {
                error!("Could not save queue state to cache: {e}");
            }
            state
        }
    };
    trace!("Queue state ready!");
    let queue_state = Arc::new(RwLock::new(state));

    let (command_sender, command_receiver) = mpsc::channel();
    *COMMAND_SENDER.write().unwrap() = Some(command_sender);

    let state_cloned = queue_state.clone();
    thread::spawn(move || {
        let handle = match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(sink) => sink,
            Err(e) => {
                error!("Could not open the default sink, audio playback will not work! Error: {e}");
                return;
            }
        };
        let player = Player::connect_new(handle.mixer());

        loop {
            thread::sleep(Duration::from_millis(100));
            if player.len() == 0 {
                let arc = state_cloned.clone();
                let mut state = arc.write().unwrap();
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
                println!("Source set!");
                player.append(source);
            }
        }
    });

    loop {
        let command = match command_receiver.recv() {
            Ok(command) => command,
            Err(e) => {
                error!("Failed receiving command: {e}");
                continue;
            }
        };

        trace!("Got command: {command:?}");
        match command {
            Command::Play => {
                trace!("Playing the current media");
            }
            Command::Pause => {
                trace!("Pausing the current media");
            }
            Command::SetQueue(queue) => {
                trace!("Setting a new queue");
                let arc = queue_state.clone();
                let mut state = arc.write().unwrap();
                state.set_tracks(queue);
                trace!("Queue length: {}", state.len());
                match save_state_to_cache(state.clone()) {
                    Ok(_) => trace!("Saved queue state to cache"),
                    Err(e) => error!("Could not save queue state to cache: {e}"),
                }
            }
            Command::Next => {
                trace!("Skipping to the next track");
                todo!("Not implemented!")
            }
            Command::Previous => {
                trace!("Skipping to the previous track");
                todo!("Not implemented!")
            }
        }
    }
}
