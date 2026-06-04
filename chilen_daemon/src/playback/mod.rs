#[cfg(feature = "mpris")]
mod mpris;
pub(crate) mod state;
#[cfg(test)]
mod tests;

use std::{
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
    thread,
    time::Duration,
};

use chilen_ipc::playback::{LoopState, PlaybackResponse};
#[cfg(feature = "shuffle")]
use chilen_ipc::{
    Event, Response,
    playback::{
        PlaybackCommand, PlaybackRate, PlaybackState, PlayerVolume, ShuffleState, SignedDuration,
    },
};
use log::{debug, error, info, trace, warn};
use rodio::Player;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    music_lib::{
        state::{Track, get_library},
        tracks_from_paths,
    },
    playback::state::{
        PLAYER_STATE, PlayerState, background_save_state, restore_state_from_cache,
        unwrap_state_mut, unwrap_state_ref,
    },
};

/// Configuration options specific to the [`playback`](crate::playback) module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    /// A friendly name to identify the media player to users (eg: “VLC media player”).
    ///
    /// This should usually match the name found in .desktop files.
    #[cfg(any(feature = "mpris", doc))]
    pub identity: String,
    /// The bus name suffix to be used with MPRIS.
    ///
    /// The resulting bus name will be `org.mpris.MediaPlayer2.<bus_name_suffix>`, where
    /// `<bus_name_suffix>` must be a unique identifier, such as one based on a UNIX process id.
    /// For example, this could be:
    ///
    /// - `org.mpris.MediaPlayer2.vlc.instance7389`
    ///
    /// **Note:** According to the D-Bus specification, the unique identifier “must only contain
    /// the ASCII characters \[A-Z\]\[a-z\]\[0-9\]_-” and “must not begin with a digit”.
    #[cfg(any(feature = "mpris", doc))]
    pub bus_name_suffix: String,
    /// Whether to allow clients to modify the playback rate of the player.
    ///
    /// If set to true, the playback rate will be locked the value saved in cache the player state,
    /// or set to the default value of `1.0` (regular speed) if it's not cached.
    pub allow_rate_modification: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum Command {
    Play(Option<usize>),
    Pause,
    Stop,
    TogglePlaying,
    GetPlaybackState,
    SetQueue(Vec<Track>),
    AppendToQueue(Vec<Track>),
    GetCurrentTrack,
    Next,
    Previous,
    SetLoopState(LoopState),
    GetLoopState,
    SetRate(f64),
    GetRate,
    #[cfg(feature = "shuffle")]
    SetShuffleState(ShuffleState),
    GetShuffleState,
    #[cfg(not(feature = "shuffle"))]
    SetShuffleState,
    SetPlayerPosition(Duration),
    Seek(SignedDuration),
    GetPlayerPosition,
    SetPlayerVolume(PlayerVolume),
    GetPlayerVolume,
    OpenURI(String),
}

impl TryFrom<PlaybackCommand> for Command {
    type Error = chilen_ipc::Error;
    fn try_from(value: PlaybackCommand) -> Result<Self, Self::Error> {
        match value {
            PlaybackCommand::Play(maybe_index) => Ok(Self::Play(maybe_index)),
            PlaybackCommand::Pause => Ok(Self::Pause),
            PlaybackCommand::Stop => Ok(Self::Stop),
            PlaybackCommand::TogglePlaying => Ok(Self::TogglePlaying),
            PlaybackCommand::GetPlaybackState => Ok(Self::GetPlaybackState),
            PlaybackCommand::SetQueue(track_paths) => {
                let tracks = tracks_from_paths(&track_paths, false)?;
                Ok(Self::SetQueue(tracks))
            }
            PlaybackCommand::AppendToQueue(track_paths) => {
                let tracks = tracks_from_paths(&track_paths, false)?;
                Ok(Self::AppendToQueue(tracks))
            }
            PlaybackCommand::SetPlaylist(playlist) => {
                let lib = get_library()?;
                let playlist = match lib.find_playlist(&playlist) {
                    Some(playlist) => playlist,
                    None => return Err(chilen_ipc::Error::UnknownPlaylist),
                };
                Ok(Self::SetQueue(
                    playlist.tracks.iter().map(|t| t.as_ref().clone()).collect(),
                ))
            }
            PlaybackCommand::AppendPlaylist(playlist) => {
                let lib = get_library()?;
                let playlist = match lib.find_playlist(&playlist) {
                    Some(playlist) => playlist,
                    None => return Err(chilen_ipc::Error::UnknownPlaylist),
                };
                Ok(Self::AppendToQueue(
                    playlist.tracks.iter().map(|t| t.as_ref().clone()).collect(),
                ))
            }
            PlaybackCommand::GetCurrentTrack => Ok(Self::GetCurrentTrack),
            PlaybackCommand::Next => Ok(Self::Next),
            PlaybackCommand::Previous => Ok(Self::Previous),
            PlaybackCommand::SetLoopState(loop_state) => Ok(Self::SetLoopState(loop_state)),
            PlaybackCommand::GetLoopState => Ok(Self::GetLoopState),
            PlaybackCommand::SetRate(rate) => Ok(Self::SetRate(rate)),
            PlaybackCommand::GetRate => Ok(Self::GetRate),
            #[cfg(feature = "shuffle")]
            PlaybackCommand::SetShuffleState(shuffle_state) => {
                Ok(Self::SetShuffleState(shuffle_state))
            }
            #[cfg(not(feature = "shuffle"))]
            PlaybackCommand::SetShuffleState(_) => Ok(Self::SetShuffleState),
            PlaybackCommand::GetShuffleState => Ok(Self::GetShuffleState),
            PlaybackCommand::SetPlayerPosition(position) => Ok(Self::SetPlayerPosition(position)),
            PlaybackCommand::Seek(delta) => Ok(Self::Seek(delta)),
            PlaybackCommand::GetPlayerPosition => Ok(Self::GetPlayerPosition),
            PlaybackCommand::SetPlayerVolume(volume) => Ok(Self::SetPlayerVolume(volume)),
            PlaybackCommand::GetPlayerVolume => Ok(Self::GetPlayerVolume),
            PlaybackCommand::OpenURI(uri) => Ok(Self::OpenURI(uri)),
        }
    }
}

static PLAYER_HANDLE: LazyLock<Arc<RwLock<Option<rodio::Player>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub(crate) static CONFIG: LazyLock<Arc<RwLock<Option<Config>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub(crate) fn cleanup() {
    *PLAYER_HANDLE.write().unwrap() = None;
    *CONFIG.write().unwrap() = None;
}

pub(crate) fn unwrap_player(
    maybe_player: Option<&rodio::Player>,
) -> Result<&rodio::Player, chilen_ipc::Error> {
    match maybe_player {
        Some(player) => Ok(player),
        None => Err(chilen_ipc::Error::StateNotInitialized),
    }
}

/// Set whether the daemon should allow rate modification by clients.
///
/// Will fail with [`Error::DaemonNotRunning`](super::Error::DaemonNotRunning) if the daemon isn't
/// running.
pub fn set_allow_rate_modification(allow_rate_modification: bool) -> Result<(), super::Error> {
    let mut conf_guard = CONFIG.write().unwrap();
    let conf = match conf_guard.as_mut() {
        Some(conf) => conf,
        None => return Err(super::Error::DaemonNotRunning),
    };
    conf.allow_rate_modification = allow_rate_modification;
    Ok(())
}

/// Play the current track or a track at a specified index in the queue.
pub(crate) fn play(index: Option<usize>) -> Result<(), chilen_ipc::Error> {
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = unwrap_player(player_guard.as_ref())?;
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    if let Some(id) = index {
        trace!("Playing a track at index {id}");
        let track = match state.play_track(id) {
            Some(track) => track,
            None => {
                error!("No track at index {id}");
                return Err(chilen_ipc::Error::NoTrackAtIndex(id));
            }
        };
        let source = match track.open_source() {
            Ok(source) => source,
            Err(e) => {
                error!("Could not open audio source: {e}");
                return Err(chilen_ipc::Error::SourceError);
            }
        };
        player.stop();
        player.append(source);
        player.play();
        state.set_playback_state(PlaybackState::Playing);
        background_save_state(state.clone());
        Ok(())
    } else {
        trace!("Playing the current media");
        if !player.is_paused() && !player.empty() {
            Err(chilen_ipc::Error::PlayerPlaying)
        } else if player.empty() {
            if let Some(track) = state.current() {
                let source = match track.open_source() {
                    Ok(source) => source,
                    Err(e) => {
                        error!("Could not open audio source: {e}");
                        return Err(chilen_ipc::Error::SourceError);
                    }
                };
                player.append(source);
                state.set_playback_state(PlaybackState::Playing);
                Ok(())
            } else {
                Err(chilen_ipc::Error::QueueEmpty)
            }
        } else {
            player.play();
            state.set_playback_state(PlaybackState::Playing);
            Ok(())
        }
    }
}

pub(crate) fn pause() -> Result<(), chilen_ipc::Error> {
    trace!("Pausing the current media");
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = player_guard.as_ref().unwrap();
    if player.is_paused() {
        Err(chilen_ipc::Error::PlayerPaused)
    } else {
        // FIX: Audio popping by adding a 10ms fade out effect here
        // NOTE: This is a missing feature in Rodio, see https://github.com/RustAudio/rodio/issues/889
        player.pause();
        let mut state_guard = PLAYER_STATE.write().unwrap();
        let state = unwrap_state_mut(state_guard.as_mut())?;
        state.set_playback_state(PlaybackState::Paused);
        Ok(())
    }
}

pub(crate) fn stop() -> Result<(), chilen_ipc::Error> {
    trace!("Stopping the playback");
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = player_guard.as_ref().unwrap();
    if player.empty() {
        Err(chilen_ipc::Error::PlayerStopped)
    } else {
        player.stop();
        let mut state_guard = PLAYER_STATE.write().unwrap();
        let state = unwrap_state_mut(state_guard.as_mut())?;
        state.set_playback_state(PlaybackState::Stopped);
        Ok(())
    }
}

pub(crate) fn toggle_playing() -> Result<(), chilen_ipc::Error> {
    trace!("Toggling playback state");
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    let playback_state = state.playback_state;
    match playback_state {
        PlaybackState::Paused => {
            drop(state_guard);
            play(None)
        }
        PlaybackState::Playing => {
            drop(state_guard);
            pause()
        }
        PlaybackState::Stopped => {
            if state.current().is_some() {
                let pos = state.position;
                drop(state_guard);
                play(Some(pos))
            } else {
                Err(chilen_ipc::Error::QueueEmpty)
            }
        }
    }
}

pub(crate) fn get_playback_state() -> Result<PlaybackState, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.playback_state)
}

pub(crate) fn open_uri(uri: PathBuf) -> Result<(), chilen_ipc::Error> {
    trace!("Opening URI {uri:?}");
    match uri.try_exists() {
        Ok(exists) => {
            if !exists {
                return Err(chilen_ipc::Error::PathDoesNotExist);
            }
        }
        Err(e) => {
            error!("Could not check if the URI {uri:?} exists: {e}");
            return Err(chilen_ipc::Error::PathExistenceUnknown);
        }
    }
    if uri.is_dir() {
        trace!("The provided URI {uri:?} is a directory, indexing it");

        let mut files = Vec::new();
        for result in WalkDir::new(uri).into_iter() {
            let entry = match result {
                Ok(entry) => entry,
                Err(e) => {
                    warn!("Error while trying to access the file: {e}");
                    continue;
                }
            };

            match entry.metadata() {
                Ok(meta) => {
                    if meta.is_file() {
                        files.push(PathBuf::from(entry.path()));
                    }
                }
                Err(e) => {
                    warn!("Could not get path metadata: {e}");
                    continue;
                }
            };
        }
        let tracks = tracks_from_paths(&files, true)?;
        if tracks.is_empty() {
            Err(chilen_ipc::Error::DirectoryWithNoTracks)
        } else {
            set_queue(tracks)
        }
    } else {
        trace!(
            "The provided URI {uri:?} is a file, so we're either opening an audio file or a playlist"
        );
        // TODO: Detect if the file is a playlist and open it
        let track = tracks_from_paths(&[uri], false)?;
        set_queue(track)
    }
}

pub(crate) fn set_queue(queue: Vec<Track>) -> Result<(), chilen_ipc::Error> {
    trace!("Setting a new queue");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.set_tracks(queue);
    #[cfg(feature = "shuffle")]
    if state.shuffle_state == ShuffleState::On {
        state.shuffle();
    }
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = unwrap_player(player_guard.as_ref())?;
    player.stop();
    if let Some(track) = state.current() {
        let source = match track.open_source() {
            Ok(source) => source,
            Err(e) => {
                error!("Could not open audio source: {e}");
                return Err(chilen_ipc::Error::SourceError);
            }
        };
        player.append(source);
        player.pause();
    }
    background_save_state(state.clone());
    Ok(())
}

pub(crate) fn append_to_queue(queue: &mut Vec<Track>) -> Result<(), chilen_ipc::Error> {
    trace!("Appending tracks to queue");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.append_tracks(queue);
    background_save_state(state.clone());
    Ok(())
}

pub(crate) fn get_current_track() -> Result<Option<Track>, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    match state.current() {
        Some(track) => Ok(Some(track.clone())),
        None => Ok(None),
    }
}

#[cfg(feature = "mpris")]
pub(crate) fn get_current_meta() -> Result<Option<mpris_server::Metadata>, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    match state.current() {
        Some(track) => Ok(Some(track.get_meta(state.position))),
        None => Ok(None),
    }
}

pub(crate) fn skip_next() -> Result<(), chilen_ipc::Error> {
    trace!("Skipping to the next track");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    if state.can_go_next() {
        let track = state.next().unwrap();
        let source = match track.open_source() {
            Ok(source) => source,
            Err(e) => {
                error!("Could not open audio source: {e}");
                return Err(chilen_ipc::Error::SourceError);
            }
        };
        background_save_state(state.clone());
        let player_guard = PLAYER_HANDLE.read().unwrap();
        let player = unwrap_player(player_guard.as_ref())?;
        state.set_player_position(Duration::default());
        state.set_playback_state(PlaybackState::Playing);
        player.clear();
        player.append(source);
        player.play();
        Ok(())
    } else if state.is_empty() {
        info!("Cannot skip to the next track, queue is empty");
        Err(chilen_ipc::Error::QueueEmpty)
    } else {
        info!("Cannot skip to the next track");
        Err(chilen_ipc::Error::CannotGoNext)
    }
}

pub(crate) fn skip_previous() -> Result<(), chilen_ipc::Error> {
    trace!("Skipping to the previous track");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    if state.can_go_previous() {
        let track = state.previous().unwrap();
        let source = match track.open_source() {
            Ok(source) => source,
            Err(e) => {
                error!("Could not open audio source: {e}");
                return Err(chilen_ipc::Error::SourceError);
            }
        };
        background_save_state(state.clone());
        let player_guard = PLAYER_HANDLE.read().unwrap();
        let player = unwrap_player(player_guard.as_ref())?;
        state.set_player_position(Duration::default());
        state.set_playback_state(PlaybackState::Playing);
        player.clear();
        player.append(source);
        player.play();
        Ok(())
    } else if state.is_empty() {
        info!("Cannot go to the previous track, queue is empty");
        Err(chilen_ipc::Error::QueueEmpty)
    } else {
        info!("Cannot go to the previous track");
        Err(chilen_ipc::Error::CannotGoPrevious)
    }
}

pub(crate) fn set_loop_state(loop_state: LoopState) -> Result<(), chilen_ipc::Error> {
    trace!("Setting loop state to {loop_state:?}");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.set_loop_state(loop_state);
    background_save_state(state.clone());
    Ok(())
}

pub(crate) fn get_loop_state() -> Result<LoopState, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.loop_state)
}

pub(crate) fn set_rate(rate: f64) -> Result<(), chilen_ipc::Error> {
    let conf = CONFIG.read().unwrap();
    if !conf.as_ref().unwrap().allow_rate_modification {
        return Err(chilen_ipc::Error::FixedRate);
    }
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    if !state.playback_rate.is_in_range(rate) {
        Err(chilen_ipc::Error::RateOutOfRange)
    } else {
        let player_guard = PLAYER_HANDLE.read().unwrap();
        let player = unwrap_player(player_guard.as_ref())?;
        player.set_speed(PlaybackRate::from(rate).get_value_f32());
        state.set_rate(rate);
        background_save_state(state.clone());
        Ok(())
    }
}

pub(crate) fn get_rate() -> Result<PlaybackRate, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.playback_rate)
}

pub(crate) fn set_shuffle_state(shuffle_state: ShuffleState) -> Result<(), chilen_ipc::Error> {
    #[cfg(feature = "shuffle")]
    {
        trace!("Setting shuffle state to {shuffle_state:?}");
        let mut state_guard = PLAYER_STATE.write().unwrap();
        let state = unwrap_state_mut(state_guard.as_mut())?;
        state.set_shuffle_state(shuffle_state);
        state.shuffle();
        background_save_state(state.clone());
        Ok(())
    }
    #[cfg(not(feature = "shuffle"))]
    {
        info!("The daemon was built without shuffle support");
        return Err(chilen_ipc::Error::ShuffleNotSupported);
    }
}

pub(crate) fn get_shuffle_state() -> Result<ShuffleState, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.shuffle_state)
}

pub(crate) fn set_player_position(position: Duration) -> Result<(), chilen_ipc::Error> {
    trace!("Setting player position to {:?}", position.as_secs());
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    if state.playback_state == PlaybackState::Stopped {
        return Err(chilen_ipc::Error::PlayerStopped);
    }
    let player = match player_guard.as_ref() {
        Some(player) => player,
        None => {
            warn!("Cannot set player position, player is not connected");
            return Err(chilen_ipc::Error::PlayerNotConnected);
        }
    };
    if player.empty() {
        Err(chilen_ipc::Error::QueueEmpty)
    } else if let Some(track) = state.current()
        && position > track.duration
    {
        skip_next()
    } else if let Err(e) = player.try_seek(position) {
        error!("Could not set player position: {e}");
        // Prevent MPRIS from showing incorrect data if seek is not supported
        #[cfg(feature = "mpris")]
        mpris::set_position(state.player_position);
        Err(chilen_ipc::Error::SeekNotSupported)
    } else {
        state.set_player_position(position);
        Ok(())
    }
}

/// When seeking, if the resulting position would result in a value less than this threshold, set
/// the position to 0s.
///
/// Similarly, if the difference between the duration of the current track and the resulting
/// position would be smaller than this value, skip to the next track.
static SEEK_ROUND_THRESHOLD: LazyLock<Duration> = LazyLock::new(|| Duration::from_secs(1));

/// Mime types supported by the music player.
///
/// Chilen uses the `rodio` crate for audio playback, which itself uses
/// [Symphonia](https://github.com/pdeljanov/Symphonia) for decoding audio files. Chilen supports
/// all audio formats Symphonia does.
#[cfg(feature = "mpris")]
static SUPPORTED_MIME_TYPES: LazyLock<Vec<String>> = LazyLock::new(|| {
    let arr = [
        "audio/aac",
        "audio/32kadpcm",
        "audio/aiff",
        "audio/x-aiff",
        "audio/x-caf",
        "audio/flac",
        "audio/matroska",
        "audio/mpeg",
        "audio/mp4",
        "audio/MPA",
        "audio/mpa-robust",
        "audio/ogg",
        "audio/vorbis",
        "audio/vorbis-config",
        "audio/vnd.wave",
        "audio/wav",
        "audio/wave",
        "audio/x-wav",
        "audio/webm",
    ];
    arr.into_iter().map(|t| t.to_string()).collect()
});

// FIX: Sometimes the MPRIS track metadata remains after the track switches in a seek operation
pub(crate) fn seek(delta: SignedDuration) -> Result<(), chilen_ipc::Error> {
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = match player_guard.as_ref() {
        Some(player) => player,
        None => {
            warn!("Cannot seek, the player is not connected");
            return Err(chilen_ipc::Error::PlayerNotConnected);
        }
    };
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    let pos = match delta {
        SignedDuration::Positive(positive_dur) => {
            if positive_dur == Duration::default() {
                info!("Refusing to seek by 0s");
                return Err(chilen_ipc::Error::InvalidDuration);
            }
            let sum = match positive_dur.checked_add(player.get_pos()) {
                Some(sum) => sum,
                None => {
                    error!("Overflow detected while seeking, aborting");
                    return Err(chilen_ipc::Error::DurationOverflow);
                }
            };
            if let Some(track) = state.current()
                && sum > track.duration - *SEEK_ROUND_THRESHOLD
            {
                drop(state_guard);
                return skip_next();
            } else {
                trace!("Seeking player by {positive_dur:?}");
                sum
            }
        }
        SignedDuration::Negative(negative_dur) => {
            if negative_dur == Duration::default() {
                info!("Refusing to seek by 0s");
                return Err(chilen_ipc::Error::InvalidDuration);
            }
            trace!("Seeking player by -{negative_dur:?}");
            let sub = match player.get_pos().checked_sub(negative_dur) {
                Some(sub) => sub,
                None => {
                    error!("Overflow detected while seeking");
                    return Err(chilen_ipc::Error::DurationOverflow);
                }
            };
            if sub < *SEEK_ROUND_THRESHOLD {
                Duration::default()
            } else {
                sub
            }
        }
    };
    if let Err(e) = player.try_seek(pos) {
        error!("Could not set player position: {e}");
        Err(chilen_ipc::Error::SeekNotSupported)
    } else {
        state.set_player_position(pos);
        Ok(())
    }
}

pub(crate) fn get_player_position() -> Result<Duration, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.player_position)
}

pub(crate) fn set_player_volume(volume: PlayerVolume) -> Result<(), chilen_ipc::Error> {
    trace!("Setting player volume to {:?}", volume);
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = unwrap_player(player_guard.as_ref())?;
    player.set_volume(volume.get());
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.set_player_volume(volume);
    background_save_state(state.clone());
    Ok(())
}

pub(crate) fn get_player_volume() -> Result<PlayerVolume, chilen_ipc::Error> {
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = unwrap_player(player_guard.as_ref())?;
    Ok(PlayerVolume::new(player.volume()))
}

pub(crate) fn get_initial_events() -> Result<Vec<Event>, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.get_initial_events())
}

pub(crate) fn init(config: Config) {
    trace!("Initializing the playback module");

    trace!("Initializing config");
    *CONFIG.write().unwrap() = Some(config.clone());

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
        state.player_position = Duration::default();
        player.append(source);
        player.pause();
        player.set_speed(state.playback_rate.get_value_f32());
        drop(player_guard);
    }
    drop(state_guard);

    #[cfg(feature = "mpris")]
    mpris::launch_server(config);

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
            let track = state.next().unwrap();
            let source = match track.open_source() {
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
                state.set_playback_state(PlaybackState::Stopped);
            } else {
                state.set_playback_state(PlaybackState::Playing);
            }
        }
        drop(state_guard);
        drop(player_guard);
    }
}

#[cfg(feature = "mpris")]
pub(crate) fn can_play() -> Result<bool, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.can_play())
}

#[cfg(feature = "mpris")]
pub(crate) fn can_pause() -> Result<bool, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.playback_state == PlaybackState::Playing)
}

#[cfg(feature = "mpris")]
pub(crate) fn can_go_next() -> Result<bool, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.can_go_next())
}

#[cfg(feature = "mpris")]
pub(crate) fn can_go_previous() -> Result<bool, chilen_ipc::Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.can_go_previous())
}

pub(crate) fn run_command(cmd: Command) -> Result<Response, chilen_ipc::Error> {
    match cmd {
        Command::Play(pos) => play(pos)?,
        Command::Pause => pause()?,
        Command::Stop => stop()?,
        Command::TogglePlaying => toggle_playing()?,
        Command::GetPlaybackState => {
            return Ok(Response::Playback(PlaybackResponse::PlaybackState(
                get_playback_state()?,
            )));
        }
        Command::SetQueue(queue) => set_queue(queue)?,
        Command::AppendToQueue(mut queue) => append_to_queue(&mut queue)?,
        Command::GetCurrentTrack => {
            return Ok(Response::Playback(match get_current_track()? {
                Some(track) => PlaybackResponse::Track(Some(Box::new(track.into()))),
                None => PlaybackResponse::Track(None),
            }));
        }
        Command::Next => skip_next()?,
        Command::Previous => skip_previous()?,
        Command::SetLoopState(loop_state) => set_loop_state(loop_state)?,
        Command::GetLoopState => {
            return Ok(Response::Playback(PlaybackResponse::LoopState(
                get_loop_state()?,
            )));
        }
        Command::SetRate(rate) => set_rate(rate)?,
        Command::GetRate => {
            return Ok(Response::Playback(PlaybackResponse::PlaybackRate(
                get_rate()?,
            )));
        }
        Command::SetShuffleState(shuffle_state) => set_shuffle_state(shuffle_state)?,
        #[cfg(feature = "shuffle")]
        Command::GetShuffleState => {
            return Ok(Response::Playback(PlaybackResponse::ShuffleState(
                get_shuffle_state()?,
            )));
        }
        Command::SetPlayerPosition(position) => set_player_position(position)?,
        Command::Seek(delta) => seek(delta)?,
        Command::GetPlayerPosition => {
            return Ok(Response::Playback(PlaybackResponse::PlayerPosition(
                get_player_position()?,
            )));
        }
        Command::SetPlayerVolume(volume) => set_player_volume(volume)?,
        Command::GetPlayerVolume => {
            return Ok(Response::Playback(PlaybackResponse::PlayerVolume(
                get_player_volume()?,
            )));
        }
        Command::OpenURI(uri) => open_uri(uri.into())?,
    }

    Ok(Response::Ok)
}
