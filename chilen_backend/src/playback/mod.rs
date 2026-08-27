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

use log::{debug, error, info, trace, warn};
use rodio::Player;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    Error,
    music_lib::{Track, tracks_from_m3u8, tracks_from_paths},
    playback::state::{
        PLAYER_STATE, background_save_state, restore_state_from_cache, unwrap_state_mut,
        unwrap_state_ref,
    },
};

pub use state::{Event, PlayerState, Queue, QueueSource};

/// Playback state of the player.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaybackState {
    /// The player is playing.
    Playing,
    /// The player is paused.
    Paused,
    /// The player is stopped.
    ///
    /// Playing will play the current track from the beginning.
    #[default]
    Stopped,
}

impl std::fmt::Display for PlaybackState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Playing => write!(f, "Playing"),
            Self::Paused => write!(f, "Paused"),
            Self::Stopped => write!(f, "Stopped"),
        }
    }
}

/// Specifies how the player loops playback.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoopState {
    /// The playback will stop when there are no more tracks to play.
    #[default]
    Off,
    /// The playback will loop through the entire queue.
    Playlist,
    /// The current track will start again from the beginning once it has finished playing.
    Track,
}

impl LoopState {
    pub fn cycle(&self) -> Self {
        match self {
            Self::Off => Self::Playlist,
            Self::Playlist => Self::Track,
            Self::Track => Self::Off,
        }
    }
}
/// Shuffle state of the player.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShuffleState {
    /// Play tracks in their original order.
    ///
    /// For example tracks from a playlist will be played in the order they appear in the playlist.
    #[default]
    Off,
    /// Shuffle the tracks in the queue.
    On,
}

impl ShuffleState {
    pub fn enabled(&self) -> bool {
        self == &Self::On
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Off => Self::On,
            Self::On => Self::Off,
        }
    }
}

impl std::fmt::Display for ShuffleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::On => write!(f, "On"),
        }
    }
}

impl std::fmt::Display for LoopState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Track => write!(f, "Track"),
            Self::Playlist => write!(f, "Playlist"),
        }
    }
}

/// Signed duration type used for seeking.
///
/// This is just a bare bones type that shouldn't be used as a [`Duration`] replacement in most
/// cases.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SignedDuration {
    Positive(Duration),
    Negative(Duration),
}

impl SignedDuration {
    pub fn from_secs<T: Into<i64> + Copy>(secs: T) -> SignedDuration {
        if secs.into() < 0 {
            SignedDuration::Negative(Duration::from_secs(secs.into().abs().try_into().unwrap()))
        } else {
            SignedDuration::Positive(Duration::from_secs(secs.into().abs().try_into().unwrap()))
        }
    }
}

/// Player volume.
///
/// Values passed to this struct will be clamped between `0.0` (no sound at all) and `1.0` (regular
/// volume).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlayerVolume {
    volume: f64,
}

impl Default for PlayerVolume {
    fn default() -> Self {
        Self { volume: 1.0 }
    }
}

impl std::fmt::Display for PlayerVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player volume: {}", self.volume)
    }
}

impl PlayerVolume {
    /// Create a new [`PlayerVolume`] struct with the specified volume.
    ///
    /// The passed `volume` parameter will be clamped between `0.0` and `1.0`.
    pub fn new(volume: f64) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
        }
    }

    /// Set the volume.
    ///
    /// The passed `volume` parameter will be clamped between 0.0 and 1.0.
    pub fn set(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Get the wrapped value.
    pub fn get(&self) -> f64 {
        self.volume
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    /// Minimum allowed player position where skipping to the previous track doesn't reset player
    /// position instead.
    ///
    /// Setting this to [`None`] disables the behavior completely.
    pub skip_previous_threshold: Option<Duration>,
}

pub(crate) static PLAYER_HANDLE: LazyLock<Arc<RwLock<Option<rodio::Player>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub(crate) fn unwrap_player(maybe_player: Option<&rodio::Player>) -> Result<&rodio::Player, Error> {
    match maybe_player {
        Some(player) => Ok(player),
        None => Err(Error::StateNotInitialized),
    }
}

/// Play the current track or a track at a specified index in the queue.
pub fn play(index: Option<usize>) -> Result<(), Error> {
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
                return Err(Error::NoTrackAtIndex(id));
            }
        };
        let source = match track.open_source() {
            Ok(source) => source,
            // FIX: Add better error handling here, so the display doesn't completely explode when this happens
            Err(e) => {
                error!("Could not open audio source: {e}");
                return Err(Error::SourceError);
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
            Err(Error::PlayerPlaying)
        } else if player.empty() {
            if let Some(track) = state.current() {
                let source = match track.open_source() {
                    Ok(source) => source,
                    Err(e) => {
                        error!("Could not open audio source: {e}");
                        return Err(Error::SourceError);
                    }
                };
                player.append(source);
                state.set_playback_state(PlaybackState::Playing);
                Ok(())
            } else {
                Err(Error::QueueEmpty)
            }
        } else {
            player.play();
            state.set_playback_state(PlaybackState::Playing);
            Ok(())
        }
    }
}

pub fn pause() -> Result<(), Error> {
    trace!("Pausing the current media");
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = player_guard.as_ref().unwrap();
    if player.is_paused() {
        Err(Error::PlayerPaused)
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

pub fn stop() -> Result<(), Error> {
    trace!("Stopping the playback");
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = player_guard.as_ref().unwrap();
    if player.empty() {
        Err(Error::PlayerStopped)
    } else {
        player.stop();
        let mut state_guard = PLAYER_STATE.write().unwrap();
        let state = unwrap_state_mut(state_guard.as_mut())?;
        state.set_playback_state(PlaybackState::Stopped);
        Ok(())
    }
}

pub fn toggle_playing() -> Result<(), Error> {
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
                Err(Error::QueueEmpty)
            }
        }
    }
}

#[cfg(feature = "mpris")]
pub(crate) fn get_playback_state() -> Result<PlaybackState, Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.playback_state)
}

#[cfg(feature = "mpris")]
pub(crate) fn get_current_meta() -> Result<Option<mpris_server::Metadata>, Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    match state.current() {
        Some(track) => Ok(Some(track.get_meta(state.position))),
        None => Ok(None),
    }
}

#[cfg(feature = "mpris")]
pub(crate) fn get_loop_state() -> Result<LoopState, Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.loop_state)
}

#[cfg(feature = "mpris")]
pub(crate) fn get_shuffle_state() -> Result<ShuffleState, Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.shuffle_state)
}

#[cfg(feature = "mpris")]
pub(crate) fn get_player_position() -> Result<Duration, Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.player_position)
}

#[cfg(feature = "mpris")]
pub(crate) fn get_player_volume() -> Result<PlayerVolume, Error> {
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = unwrap_player(player_guard.as_ref())?;
    Ok(PlayerVolume::new(player.volume()))
}

#[cfg(feature = "mpris")]
pub(crate) fn can_play() -> Result<bool, Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.can_play())
}

#[cfg(feature = "mpris")]
pub(crate) fn can_pause() -> Result<bool, Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.playback_state == PlaybackState::Playing)
}

#[cfg(feature = "mpris")]
pub(crate) fn can_go_next() -> Result<bool, Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.can_go_next())
}

#[cfg(feature = "mpris")]
pub(crate) fn can_go_previous() -> Result<bool, Error> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.can_go_previous())
}

// TEST: Add tests for this
/// Opens a URI that can either be a track, a directory with tracks, or an M3U8 playlist file.
pub fn open_uri(uri: PathBuf) -> Result<(), Error> {
    trace!("Opening URI {uri:?}");
    match uri.try_exists() {
        Ok(exists) => {
            if !exists {
                return Err(Error::PathDoesNotExist);
            }
        }
        Err(e) => {
            error!("Could not check if the URI {uri:?} exists: {e}");
            return Err(Error::PathInaccessible(uri));
        }
    }
    if uri.is_dir() {
        trace!("The provided URI {uri:?} is a directory, indexing it");

        let mut files = Vec::new();
        let label = uri.to_string_lossy().to_string();
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
            Err(Error::DirectoryWithNoTracks)
        } else {
            set_queue(Queue::Custom { label, tracks })
        }
    } else {
        trace!(
            "The provided URI {uri:?} is a file, so we're either opening an audio file or a playlist"
        );
        if let Ok(paths) = tracks_from_m3u8(&uri) {
            trace!("Loaded M3U8 playlist {uri:?}");
            let tracks = tracks_from_paths(&paths, true)?;
            return set_queue(Queue::Custom {
                label: uri.to_string_lossy().to_string(),
                tracks,
            });
        } else {
            trace!("The file does not appear to be an M3U8 playlist");
        }
        let label = uri.to_string_lossy().to_string();
        let track = tracks_from_paths(&[uri], false)?;
        set_queue(Queue::Custom {
            label,
            tracks: track,
        })
    }
}

/// Set a new queue and play a track at a specific index in that queue.
///
/// If the index is unspecified, then it will be reset to 0. Additionally, if shuffle is enabled,
/// the queue will be shuffled fully, meaning that setting a new queue without specifying the index
/// while shuffle is enabled will start the queue from a random track.
///
/// If the index is specified, then playback will start from the track at index in the queue, even
/// if shuffle is enabled.
pub fn play_new_queue(queue: Queue, index: Option<usize>) -> Result<(), Error> {
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.play_new_queue(queue, index);

    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = unwrap_player(player_guard.as_ref())?;
    player.stop();
    if let Some(track) = state.current() {
        let source = match track.open_source() {
            Ok(source) => source,
            Err(e) => {
                error!("Could not open audio source: {e}");
                return Err(Error::SourceError);
            }
        };
        player.append(source);
        player.play();
        state.set_playback_state(PlaybackState::Playing);
    }
    background_save_state(state.clone());
    Ok(())
}

/// Set the queue to a list of track.
///
/// Note that if shuffle is enabled, the queue will be shuffled immediately after setting it. For
/// that reason, please use the `[play_new_queue]` function for setting the queue and then playing a
/// track at a specific index in that queue.
pub fn set_queue(queue: Queue) -> Result<(), Error> {
    trace!("Setting a new queue");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.set_queue(queue);
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = unwrap_player(player_guard.as_ref())?;
    player.stop();
    if let Some(track) = state.current() {
        let source = match track.open_source() {
            Ok(source) => source,
            Err(e) => {
                error!("Could not open audio source: {e}");
                return Err(Error::SourceError);
            }
        };
        player.append(source);
        player.pause();
    }
    background_save_state(state.clone());
    Ok(())
}

pub fn append_to_queue(queue: &mut Vec<Arc<Track>>) -> Result<(), Error> {
    trace!("Appending tracks to queue");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.append_tracks(queue);
    background_save_state(state.clone());
    Ok(())
}

pub fn skip_next() -> Result<(), Error> {
    trace!("Skipping to the next track");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    if state.can_go_next() {
        let track = state.next_track().unwrap();
        let source = match track.open_source() {
            Ok(source) => source,
            Err(e) => {
                error!("Could not open audio source: {e}");
                return Err(Error::SourceError);
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
    } else if state.queue_empty() {
        info!("Cannot skip to the next track, queue is empty");
        Err(Error::QueueEmpty)
    } else {
        info!("Cannot skip to the next track");
        Err(Error::CannotGoNext)
    }
}

pub fn skip_previous() -> Result<(), Error> {
    trace!("Skipping to the previous track");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;

    let guard = crate::CONFIG.read().unwrap();
    let config = guard.as_ref().unwrap();
    if let Some(threshold) = config.playback.skip_previous_threshold
        && state.player_position > threshold
    {
        trace!("Player position above skip threshold, resetting player position instead");
        drop(state_guard);
        return set_player_position(Duration::default());
    }

    if state.can_go_previous() {
        let track = state.previous_track().unwrap();
        let source = match track.open_source() {
            Ok(source) => source,
            Err(e) => {
                error!("Could not open audio source: {e}");
                return Err(Error::SourceError);
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
    } else if state.queue_empty() {
        info!("Cannot go to the previous track, queue is empty");
        Err(Error::QueueEmpty)
    } else {
        info!("Cannot go to the previous track");
        Err(Error::CannotGoPrevious)
    }
}

pub fn set_loop_state(loop_state: LoopState) -> Result<(), Error> {
    trace!("Setting loop state to {loop_state:?}");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.set_loop_state(loop_state);
    background_save_state(state.clone());
    Ok(())
}

pub fn cycle_loop_state() -> Result<(), Error> {
    trace!("Cycling loop state");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.set_loop_state(state.loop_state.cycle());
    background_save_state(state.clone());
    Ok(())
}

pub fn set_shuffle_state(shuffle_state: ShuffleState) -> Result<(), Error> {
    trace!("Setting shuffle state to {shuffle_state:?}");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.set_shuffle_state(shuffle_state);
    background_save_state(state.clone());
    Ok(())
}

pub fn toggle_shuffle_state() -> Result<(), Error> {
    trace!("Toggling shuffle state");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    let shuffle_state = state.shuffle_state.toggle();
    trace!("Setting shuffle state to {shuffle_state:?}");
    state.set_shuffle_state(shuffle_state);
    background_save_state(state.clone());
    Ok(())
}

pub fn set_player_position(position: Duration) -> Result<(), Error> {
    trace!("Setting player position to {:?}", position.as_secs());
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    if state.playback_state == PlaybackState::Stopped {
        return Err(Error::PlayerStopped);
    }
    let player = match player_guard.as_ref() {
        Some(player) => player,
        None => {
            warn!("Cannot set player position, player is not connected");
            return Err(Error::PlayerNotConnected);
        }
    };
    if player.empty() {
        Err(Error::QueueEmpty)
    } else if let Some(track) = state.current()
        && position > track.duration
    {
        drop(state_guard);
        skip_next()
    } else if let Err(e) = player.try_seek(position) {
        error!("Could not set player position: {e}");
        // Prevent MPRIS from showing incorrect data if seek is not supported
        #[cfg(feature = "mpris")]
        mpris::set_position(state.player_position);
        Err(Error::SeekNotSupported)
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

// TODO: Add mime type(s) for M3U8 files (if it makes sense)
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
pub fn seek(delta: SignedDuration) -> Result<(), Error> {
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = match player_guard.as_ref() {
        Some(player) => player,
        None => {
            warn!("Cannot seek, the player is not connected");
            return Err(Error::PlayerNotConnected);
        }
    };
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    let pos = match delta {
        SignedDuration::Positive(positive_dur) => {
            if positive_dur == Duration::default() {
                info!("Refusing to seek by 0s");
                return Err(Error::InvalidDuration);
            }
            let sum = match positive_dur.checked_add(player.get_pos()) {
                Some(sum) => sum,
                None => {
                    error!("Overflow detected while seeking, aborting");
                    return Err(Error::DurationOverflow);
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
                return Err(Error::InvalidDuration);
            }
            trace!("Seeking player by -{negative_dur:?}");
            let sub = match player.get_pos().checked_sub(negative_dur) {
                Some(sub) => sub,
                None => {
                    error!("Overflow detected while seeking");
                    return Err(Error::DurationOverflow);
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
        Err(Error::SeekNotSupported)
    } else {
        state.set_player_position(pos);
        Ok(())
    }
}

pub fn set_player_volume(volume: PlayerVolume) -> Result<(), Error> {
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

pub(crate) fn init(
    #[cfg(feature = "mpris")] identity: String,
    #[cfg(feature = "mpris")] identifier: String,
) {
    trace!("Initializing the playback module");

    let state = match restore_state_from_cache() {
        Ok(state) => {
            debug!("Restored player state from cache");
            state
        }
        Err(e) => {
            // FIX: This breaks the state in the player somehow
            error!("Could not restore player state from cache: {e}");
            let state = PlayerState::default();
            debug!("Creating a new state and attempting to save it in cache");
            background_save_state(state.clone());
            state
        }
    };
    crate::send_event(crate::Event::Playback(Event::StateInitialized(
        state.clone(),
    )));
    trace!("Player state ready!");

    // TODO: Allow setting custom sinks
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
        if let Err(e) = player.try_seek(state.player_position) {
            error!("Couldn't set initial player position: {e}");
            state.set_player_position(Duration::default());
        }
        drop(player_guard);
    }
    drop(state_guard);

    #[cfg(feature = "mpris")]
    mpris::launch_server(identity, identifier);

    trace!("Playback module initialized");

    let mut init = true;
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
            let track = match state.next_track() {
                Some(track) => track,
                None => {
                    state.set_playback_state(PlaybackState::Stopped);
                    continue;
                }
            };
            let source = match track.open_source() {
                Ok(source) => source,
                Err(e) => {
                    error!("Could not open audio source: {e}");
                    continue;
                }
            };
            player.append(source);
            if init {
                player.pause();
                init = false;
            } else {
                state.set_playback_state(PlaybackState::Playing);
            }
        }
        drop(state_guard);
        drop(player_guard);
    }
}
