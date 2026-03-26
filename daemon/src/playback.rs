use std::{
    fs::{File, read},
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock, mpsc},
};

use log::{debug, error, trace};
use mpipc::{LoopState, MusicLibraryError, PlaybackCommand, ShuffleState};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use crate::data::{
    CACHE_DIR,
    music_lib::{Track, get_library, tracks_from_hashes},
};

#[derive(Debug, Clone, Default)]
pub struct Queue {
    pub position: u64,
    pub tracks: Vec<Track>,
}

impl From<Track> for Queue {
    fn from(value: Track) -> Self {
        Self {
            position: 0,
            tracks: vec![value],
        }
    }
}

impl From<Vec<Track>> for Queue {
    fn from(value: Vec<Track>) -> Self {
        Self {
            position: 0,
            tracks: value,
        }
    }
}

impl TryFrom<QueueRaw> for Queue {
    type Error = MusicLibraryError;
    fn try_from(value: QueueRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            position: value.position,
            tracks: tracks_from_hashes(value.track_hashes, &get_library()?.tracks),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueueRaw {
    pub position: u64,
    pub track_hashes: Vec<u64>,
}

impl From<Queue> for QueueRaw {
    fn from(value: Queue) -> Self {
        Self {
            position: value.position,
            track_hashes: Track::hash_tracks(&value.tracks),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct QueueState {
    queue: Queue,
    shuffle_state: ShuffleState,
    loop_state: LoopState,
}

impl TryFrom<QueueStateRaw> for QueueState {
    type Error = MusicLibraryError;
    fn try_from(value: QueueStateRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            queue: value.queue.try_into()?,
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueueStateRaw {
    queue: QueueRaw,
    shuffle_state: ShuffleState,
    loop_state: LoopState,
}

impl From<QueueState> for QueueStateRaw {
    fn from(value: QueueState) -> Self {
        Self {
            queue: value.queue.into(),
            shuffle_state: value.shuffle_state,
            loop_state: value.loop_state,
        }
    }
}

static STATE_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data = CACHE_DIR.read().unwrap().clone().unwrap();
    data.push("queue_state");
    data
});

static QUEUE_STATE: RwLock<Option<QueueState>> = RwLock::new(None);

static COMMAND_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<PlaybackCommand>>>>> =
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

pub(crate) fn init() {
    trace!("Initializing the playback module");

    let queue_state = match restore_state_from_cache() {
        Ok(state) => {
            debug!("Restored queue state from cache");
            state
        }
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    let mut guard = QUEUE_STATE.write().unwrap();
    *guard = Some(queue_state);
    drop(guard);

    let (command_sender, command_receiver) = mpsc::channel();
    let mut guard = COMMAND_SENDER.write().unwrap();
    *guard = Some(command_sender);
    drop(guard);

    let handle = match rodio::DeviceSinkBuilder::open_default_sink() {
        Ok(sink) => sink,
        Err(e) => {
            error!("Could not open the default sink, audio playback will not work! Error: {e}");
            return;
        }
    };

    loop {
        let command = match command_receiver.recv() {
            Ok(command) => command,
            Err(e) => {
                error!("Failed receiving command: {e}");
                continue;
            }
        };

        trace!("Got command: {command}");
        match command {
            PlaybackCommand::Play => {}
            PlaybackCommand::Pause => {}
            PlaybackCommand::Next => {}
            PlaybackCommand::Previous => {}
        }
    }
}
