#[cfg(feature = "mpris")]
mod mpris;
mod state;

use std::{
    sync::{Arc, LazyLock, RwLock},
    thread,
    time::Duration,
};

use log::{debug, error, info, trace, warn};
#[cfg(feature = "shuffle")]
use mpipc::ShuffleState;
use mpipc::{
    DaemonEvent, LoopState, MusicLibraryError, PlaybackError, PlaybackRate, PlaybackResponse,
    PlaybackState, PlayerVolume, SignedDuration,
};
use rodio::Player;
use serde::{Deserialize, Serialize};

use crate::{
    data::{
        cache::indexer::index_files,
        music_lib::{Track, get_library, tracks_from_hashes},
    },
    playback::state::{
        PLAYER_STATE, PlayerState, background_save_state, restore_state_from_cache,
        unwrap_state_mut, unwrap_state_ref,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    /// A friendly name to identify the media player to users (eg: “VLC media player”).
    #[cfg(feature = "mpris")]
    pub identity: String,
    /// The suffix of the but name used with mpris.
    #[cfg(feature = "mpris")]
    pub bus_name_suffix: String,
    /// Whether to allow clients to modify the playback rate of the player.
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
}

impl TryFrom<mpipc::PlaybackCommand> for Command {
    type Error = MusicLibraryError;
    fn try_from(value: mpipc::PlaybackCommand) -> Result<Self, Self::Error> {
        match value {
            mpipc::PlaybackCommand::Play(maybe_pos) => Ok(Self::Play(maybe_pos)),
            mpipc::PlaybackCommand::Pause => Ok(Self::Pause),
            mpipc::PlaybackCommand::Stop => Ok(Self::Stop),
            mpipc::PlaybackCommand::TogglePlaying => Ok(Self::TogglePlaying),
            mpipc::PlaybackCommand::GetPlaybackState => Ok(Self::GetPlaybackState),
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
            mpipc::PlaybackCommand::GetCurrentTrack => Ok(Self::GetCurrentTrack),
            mpipc::PlaybackCommand::Next => Ok(Self::Next),
            mpipc::PlaybackCommand::Previous => Ok(Self::Previous),
            mpipc::PlaybackCommand::SetLoopState(loop_state) => Ok(Self::SetLoopState(loop_state)),
            mpipc::PlaybackCommand::GetLoopState => Ok(Self::GetLoopState),
            mpipc::PlaybackCommand::SetRate(rate) => Ok(Self::SetRate(rate)),
            mpipc::PlaybackCommand::GetRate => Ok(Self::GetRate),
            #[cfg(feature = "shuffle")]
            mpipc::PlaybackCommand::SetShuffleState(shuffle_state) => {
                Ok(Self::SetShuffleState(shuffle_state))
            }
            #[cfg(not(feature = "shuffle"))]
            mpipc::PlaybackCommand::SetShuffleState(_) => Ok(Self::SetShuffleState),
            mpipc::PlaybackCommand::GetShuffleState => Ok(Self::GetShuffleState),
            mpipc::PlaybackCommand::SetPlayerPosition(position) => {
                Ok(Self::SetPlayerPosition(position))
            }
            mpipc::PlaybackCommand::Seek(delta) => Ok(Self::Seek(delta)),
            mpipc::PlaybackCommand::GetPlayerPosition => Ok(Self::GetPlayerPosition),
            mpipc::PlaybackCommand::SetPlayerVolume(volume) => Ok(Self::SetPlayerVolume(volume)),
            mpipc::PlaybackCommand::GetPlayerVolume => Ok(Self::GetPlayerVolume),
        }
    }
}

static PLAYER_HANDLE: LazyLock<Arc<RwLock<Option<rodio::Player>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static CONFIG: LazyLock<Arc<RwLock<Option<Config>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub(crate) fn play(position: Option<usize>) -> Result<(), PlaybackError> {
    let player_guard = PLAYER_HANDLE.read().unwrap();
    if let Some(player) = player_guard.as_ref() {
        let mut state_guard = PLAYER_STATE.write().unwrap();
        let state = unwrap_state_mut(state_guard.as_mut())?;
        match position {
            Some(pos) => {
                trace!("Playing a track at position {pos}");
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
                Ok(())
            }
            None => {
                trace!("Playing the current media");
                if !player.is_paused() && !player.empty() {
                    Err(PlaybackError::PlayerPlaying)
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
                        Ok(())
                    } else {
                        Err(PlaybackError::QueueEmpty)
                    }
                } else {
                    player.play();
                    state.set_playback_state(PlaybackState::Playing);
                    Ok(())
                }
            }
        }
    } else {
        warn!("Cannot play, player is not connected");
        Err(PlaybackError::PlayerNotConnected)
    }
}

pub(crate) fn pause() -> Result<(), PlaybackError> {
    trace!("Pausing the current media");
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = player_guard.as_ref().unwrap();
    if player.is_paused() {
        Err(PlaybackError::PlayerPaused)
    } else {
        player.pause();
        let mut state_guard = PLAYER_STATE.write().unwrap();
        let state = unwrap_state_mut(state_guard.as_mut())?;
        state.set_playback_state(PlaybackState::Paused);
        Ok(())
    }
}

pub(crate) fn stop() -> Result<(), PlaybackError> {
    trace!("Stopping the playback");
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = player_guard.as_ref().unwrap();
    if player.empty() {
        Err(PlaybackError::PlayerStopped)
    } else {
        player.stop();
        let mut state_guard = PLAYER_STATE.write().unwrap();
        let state = unwrap_state_mut(state_guard.as_mut())?;
        state.set_playback_state(PlaybackState::Stopped);
        Ok(())
    }
}

pub(crate) fn toggle_playing() -> Result<(), PlaybackError> {
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
                Err(PlaybackError::QueueEmpty)
            }
        }
    }
}

pub(crate) fn get_playback_state() -> Result<PlaybackState, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.playback_state)
}

pub(crate) fn set_queue(queue: Vec<Track>) -> Result<(), PlaybackError> {
    trace!("Setting a new queue");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.set_tracks(queue);
    background_save_state(state.clone());
    Ok(())
}

pub(crate) fn append_to_queu(queue: &mut Vec<Track>) -> Result<(), PlaybackError> {
    trace!("Appending tracks to queue");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.append_tracks(queue);
    background_save_state(state.clone());
    Ok(())
}

pub(crate) fn get_current_track() -> Result<Option<Track>, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    match state.current() {
        Some(track) => Ok(Some(track.clone())),
        None => Ok(None),
    }
}

pub(crate) fn skip_next() -> Result<(), PlaybackError> {
    trace!("Skipping to the next track");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
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
            state.set_playback_state(PlaybackState::Playing);
            player.clear();
            player.append(source);
            player.play();
            Ok(())
        } else {
            warn!("Cannot skip to the next track, player is not connected");
            Err(PlaybackError::PlayerNotConnected)
        }
    } else if state.is_empty() {
        info!("Cannot skip to the next track, queue is empty");
        Err(PlaybackError::QueueEmpty)
    } else {
        info!("Cannot skip to the next track");
        Err(PlaybackError::CannotGoNext)
    }
}

pub(crate) fn skip_previous() -> Result<(), PlaybackError> {
    trace!("Skipping to the previous track");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
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
            Ok(())
        } else {
            warn!("Cannot skip to the previous track, player is not connected");
            Err(PlaybackError::PlayerNotConnected)
        }
    } else if state.is_empty() {
        info!("Cannot go to the previous track, queue is empty");
        Err(PlaybackError::QueueEmpty)
    } else {
        info!("Cannot go to the previous track");
        Err(PlaybackError::CannotGoPrevious)
    }
}

pub(crate) fn set_loop_state(loop_state: LoopState) -> Result<(), PlaybackError> {
    trace!("Setting loop state to {loop_state:?}");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    state.set_loop_state(loop_state);
    background_save_state(state.clone());
    Ok(())
}

pub(crate) fn get_loop_state() -> Result<LoopState, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.loop_state)
}

pub(crate) fn set_rate(rate: f64) -> Result<(), PlaybackError> {
    let conf = CONFIG.read().unwrap();
    if !conf.as_ref().unwrap().allow_rate_modification {
        return Err(PlaybackError::FixedRate);
    }
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    if !state.playback_rate.is_in_range(rate) {
        Err(PlaybackError::RateOutOfRange)
    } else if let Some(player) = player_guard.as_ref() {
        player.set_speed(PlaybackRate::from(rate).get_value_f32()); // Not the cleanest
        state.set_rate(rate);
        background_save_state(state.clone());
        Ok(())
    } else {
        warn!("Cannot set playback rate, player is not connected");
        Err(PlaybackError::PlayerNotConnected)
    }
}

pub(crate) fn get_rate() -> Result<PlaybackRate, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.playback_rate)
}

pub(crate) fn set_shuffle_state(shuffle_state: ShuffleState) -> Result<(), PlaybackError> {
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
        return Err(PlaybackError::ShuffleNotSupported);
    }
}

pub(crate) fn get_shuffle_state() -> Result<ShuffleState, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.shuffle_state)
}

pub(crate) fn set_player_position(position: Duration) -> Result<(), PlaybackError> {
    trace!("Setting player position to {:?}", position.as_secs());
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    if state.playback_state == PlaybackState::Stopped {
        return Err(PlaybackError::PlayerStopped);
    }
    if let Some(player) = player_guard.as_ref() {
        if player.empty() {
            return Err(PlaybackError::QueueEmpty);
        }
        if let Err(e) = player.try_seek(position) {
            error!("Could not set player position: {e}");
            Err(PlaybackError::SeekNotSupported)
        } else {
            state.set_player_position(position);
            Ok(())
        }
    } else {
        warn!("Cannot set player position, player is not connected");
        Err(PlaybackError::PlayerNotConnected)
    }
}

pub(crate) fn seek(delta: SignedDuration) -> Result<(), PlaybackError> {
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = match player_guard.as_ref() {
        Some(player) => player,
        None => {
            warn!("Cannot seek, the player is not connected");
            return Err(PlaybackError::PlayerNotConnected);
        }
    };
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = unwrap_state_mut(state_guard.as_mut())?;
    // TODO: Clamp the duration here so if a seek would result in setting the player position under
    // one second, set the player position to 0, and if it would result in the player being less
    // than one second from the end of the track, skip to the next track.
    let pos = match delta {
        SignedDuration::Positive(positive_dur) => {
            if positive_dur == Duration::default() {
                info!("Refusing to seek by 0s");
                return Err(PlaybackError::InvalidDuration);
            }
            if let Some(track) = state.current()
                && player.get_pos() + positive_dur > track.duration
            {
                return skip_next();
            } else {
                trace!("Seeking player by {positive_dur:?}");
                player.get_pos() + positive_dur
            }
        }
        SignedDuration::Negative(negative_dur) => {
            if negative_dur == Duration::default() {
                info!("Refusing to seek by 0s");
                return Err(PlaybackError::InvalidDuration);
            }
            trace!("Seeking player by -{negative_dur:?}");
            if negative_dur > player.get_pos() {
                Duration::default()
            } else {
                player.get_pos() - negative_dur
            }
        }
    };
    if let Err(e) = player.try_seek(pos) {
        error!("Could not set player position: {e}");
        Err(PlaybackError::SeekNotSupported)
    } else {
        state.set_player_position(pos);
        Ok(())
    }
}

pub(crate) fn get_player_position() -> Result<Duration, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.player_position)
}

pub(crate) fn set_player_volume(volume: PlayerVolume) -> Result<(), PlaybackError> {
    trace!("Setting player volume to {:?}", volume);
    let player_guard = PLAYER_HANDLE.read().unwrap();
    if let Some(player) = player_guard.as_ref() {
        player.set_volume(volume.get());
        let mut state_guard = PLAYER_STATE.write().unwrap();
        let state = unwrap_state_mut(state_guard.as_mut())?;
        state.set_player_volume(volume);
        background_save_state(state.clone());
        Ok(())
    } else {
        warn!("Cannot set player volume, the player is not connected");
        Err(PlaybackError::PlayerNotConnected)
    }
}

pub(crate) fn get_player_volume() -> Result<PlayerVolume, PlaybackError> {
    let player_guard = PLAYER_HANDLE.read().unwrap();
    if let Some(player) = player_guard.as_ref() {
        Ok(PlayerVolume::new(player.volume()))
    } else {
        warn!("Cannot set player volume, the player is not connected");
        Err(PlaybackError::PlayerNotConnected)
    }
}

pub(crate) fn get_initial_events() -> Result<Vec<DaemonEvent>, PlaybackError> {
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
pub(crate) fn can_play() -> Result<bool, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.can_play())
}

#[cfg(feature = "mpris")]
pub(crate) fn can_pause() -> Result<bool, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.playback_state == PlaybackState::Playing)
}

#[cfg(feature = "mpris")]
pub(crate) fn can_go_next() -> Result<bool, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.can_go_next())
}

#[cfg(feature = "mpris")]
pub(crate) fn can_go_previous() -> Result<bool, PlaybackError> {
    let state_guard = PLAYER_STATE.read().unwrap();
    let state = unwrap_state_ref(state_guard.as_ref())?;
    Ok(state.can_go_previous())
}

pub(crate) fn run_command(cmd: Command) -> Result<PlaybackResponse, PlaybackError> {
    match cmd {
        Command::Play(pos) => play(pos)?,
        Command::Pause => pause()?,
        Command::Stop => stop()?,
        Command::TogglePlaying => toggle_playing()?,
        Command::GetPlaybackState => {
            return Ok(PlaybackResponse::PlaybackState(get_playback_state()?));
        }
        Command::SetQueue(queue) => set_queue(queue)?,
        Command::AppendToQueue(mut queue) => append_to_queu(&mut queue)?,
        Command::GetCurrentTrack => {
            return match get_current_track()? {
                Some(track) => Ok(PlaybackResponse::Track(Some(Box::new(track.into())))),
                None => Ok(PlaybackResponse::Track(None)),
            };
        }
        Command::Next => skip_next()?,
        Command::Previous => skip_previous()?,
        Command::SetLoopState(loop_state) => set_loop_state(loop_state)?,
        Command::GetLoopState => return Ok(PlaybackResponse::LoopState(get_loop_state()?)),
        Command::SetRate(rate) => set_rate(rate)?,
        Command::GetRate => return Ok(PlaybackResponse::PlaybackRate(get_rate()?)),
        Command::SetShuffleState(shuffle_state) => set_shuffle_state(shuffle_state)?,
        #[cfg(feature = "shuffle")]
        Command::GetShuffleState => {
            return Ok(PlaybackResponse::ShuffleState(get_shuffle_state()?));
        }
        Command::SetPlayerPosition(position) => set_player_position(position)?,
        Command::Seek(delta) => seek(delta)?,
        Command::GetPlayerPosition => {
            return Ok(PlaybackResponse::PlayerPosition(get_player_position()?));
        }
        Command::SetPlayerVolume(volume) => set_player_volume(volume)?,
        Command::GetPlayerVolume => {
            return Ok(PlaybackResponse::PlayerVolume(get_player_volume()?));
        }
    }

    Ok(PlaybackResponse::Ok)
}
