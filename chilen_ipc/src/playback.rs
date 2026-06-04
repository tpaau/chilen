use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

use crate::library::Track;

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
    /// The current track will start again from the beginning once it has finished playing.
    Track,
    /// The playback will loop through the entire queue.
    Playlist,
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

impl From<ShuffleState> for bool {
    fn from(s: ShuffleState) -> bool {
        match s {
            ShuffleState::Off => false,
            ShuffleState::On => true,
        }
    }
}

impl From<bool> for ShuffleState {
    fn from(value: bool) -> Self {
        if value { Self::On } else { Self::Off }
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
/// This is just a bare bones type that should only be used for
/// [seeking player position](PlaybackCommand::Seek), and not as a [`Duration`] replacement.
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

/// The speed at which tracks are played.
///
/// Rate of `1.0` will play tracks at their original speed, `2.0` will play them twice as fast, and
/// `0.5` will slow them to half speed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlaybackRate {
    rate: f64,
    min: f64,
    max: f64,
}

impl Default for PlaybackRate {
    fn default() -> Self {
        Self {
            rate: 1.0,
            min: 0.1,
            max: 10.0,
        }
    }
}

impl<T: Into<f64>> From<T> for PlaybackRate {
    fn from(value: T) -> Self {
        let mut def = Self::default();
        def.set_value(value.into());
        def
    }
}

impl std::fmt::Display for PlaybackRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Current playback rate: {}, min: {}, max: {}",
            self.rate, self.min, self.max
        )
    }
}

impl PlaybackRate {
    /// Create a new [PlaybackRate] struct.
    pub fn new(rate: f64, min: f64, max: f64) -> PlaybackRate {
        Self { rate, min, max }
    }

    fn clamp(&mut self) {
        self.rate = self.rate.clamp(self.min, self.max)
    }

    /// Checks if the given rate value is in the allowed range.
    pub fn is_in_range(&self, rate: f64) -> bool {
        rate >= self.min && rate <= self.max
    }

    /// Set the rate multiplier.
    ///
    /// This value will be clamped with the min and max values.
    pub fn set_value(&mut self, rate: f64) {
        self.rate = rate.clamp(self.min, self.max)
    }

    /// Get the rate multiplier.
    pub fn get_value(&self) -> f64 {
        self.rate
    }

    /// Get the rate multiplier clamped to `f32`.
    pub fn get_value_f32(&self) -> f32 {
        if self.rate.is_nan() {
            f32::NAN
        } else if self.rate > f32::MAX as f64 {
            f32::MAX
        } else if self.rate < f32::MIN as f64 {
            f32::MIN
        } else {
            self.rate as f32
        }
    }

    /// Set the minimum acceptable value for the playback rate.
    pub fn set_min(&mut self, min: f64) {
        self.min = min.clamp(0.0, self.max);
        self.clamp();
    }

    /// Get the minimum acceptable value for the playback rate.
    pub fn get_min(&self) -> f64 {
        self.min
    }

    /// Set the maximum acceptable value for the playback rate.
    pub fn set_max(&mut self, max: f64) {
        self.max = max.clamp(self.min, f64::MAX);
        self.clamp();
    }

    /// Get the maximum acceptable value for the playback rate.
    pub fn get_max(&self) -> f64 {
        self.max
    }
}

/// Subcommand of [`Command`](crate::Command) for managing audio playback in the daemon.
///
/// The expected response may be different depending on the command sent. If it isn't specified in
/// the variant documentation, assume [`Response::Ok`](crate::Response::Ok) is the expected
/// response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackCommand {
    /// Play the current track or play a track at a specific index in the queue.
    Play(Option<usize>),
    /// Pause the player.
    Pause,
    /// Stop the player.
    Stop,
    /// Toggle between play/pause.
    TogglePlaying,
    /// Get the [`PlaybackState`] of the player.
    ///
    /// The daemon will respond with [`PlaybackResponse::PlaybackState`] if successful.
    GetPlaybackState,
    /// Set a new queue for the player.
    ///
    /// If any of the provided tracks are not registered by chilen (added after last library
    /// reload), the daemon will return [`Error::UnknownTrack`](crate::Error::UnknownTrack).
    SetQueue(Vec<PathBuf>),
    /// Append tracks to the queue.
    ///
    /// If any of the provided tracks are not registered by chilen (added after last library
    /// reload), the daemon will return [`Error::UnknownTrack`](crate::Error::UnknownTrack).
    AppendToQueue(Vec<PathBuf>),
    /// Load a playlist and append its tracks to the queue.
    ///
    /// If there's no [playlist](crate::library::Playlist) with the provided name present in the
    /// [music library](crate::library::MusicLibrary),
    /// [`Error::UnknownPlaylist`](crate::Error::UnknownPlaylist) will be returned, and no changes
    /// to the queue will be made.
    AppendPlaylist(String),
    /// Load a playlist and put its tracks in the queue.
    ///
    /// If there's no [playlist](crate::library::Playlist) with the provided name present in the
    /// [music library](crate::library::MusicLibrary),
    /// [`Error::UnknownPlaylist`](crate::Error::UnknownPlaylist) will be returned, and no changes
    /// to the queue will be made.
    SetPlaylist(String),
    /// Get the current [track](Track).
    ///
    /// The daemon will respond to this with [`PlaybackResponse::Track`] if successful.
    GetCurrentTrack,
    /// Skip to the next track.
    Next,
    /// Skip to the previous track.
    Previous,
    /// Set the [loop state](LoopState) of the player.
    SetLoopState(LoopState),
    /// Get the [loop state](LoopState) of the player.
    ///
    /// The daemon will respond to this with [`PlaybackResponse::LoopState`] if successful.
    GetLoopState,
    /// Set the [playback rate](PlaybackRate) of the player.
    ///
    /// This command will fail if the daemon is configured to not allow playback rate
    /// modification or if the specified rate value was out of the acceptable range.
    ///
    /// If the daemon is configured not to allow playback rate modification,
    /// [`Error::FixedRate`](crate::Error::FixedRate) will be returned.
    ///
    /// If the provided rate value is out of the allowed range,
    /// [`Error::RateOutOfRange`](crate::Error::RateOutOfRange) will be returned.
    SetRate(f64),
    /// Get the [playback rate](PlaybackRate) of the player.
    ///
    /// The daemon will respond to this with [`PlaybackResponse::PlaybackRate`] if
    /// successful.
    GetRate,
    /// Set the [shuffle state](ShuffleState) of the player.
    ///
    /// The daemon will always respond to this command with
    /// [`Error::ShuffleNotSupported`](crate::Error::ShuffleNotSupported) if it was built without
    /// shuffle support.
    SetShuffleState(ShuffleState),
    /// Get the [shuffle state](ShuffleState) of the player.
    ///
    /// If the daemon was built without shuffle support, it will always respond to this command with
    /// [`ShuffleState::Off`]. Otherwise, it will return [`PlaybackResponse::ShuffleState`].
    GetShuffleState,
    /// Set the position of the player.
    SetPlayerPosition(Duration),
    /// Change the player position by a time delta.
    Seek(SignedDuration),
    /// Get the position of the player.
    ///
    /// The daemon will respond to this with [`PlaybackResponse::PlayerPosition`] if
    /// successful.
    GetPlayerPosition,
    /// Set the volume of the player.
    SetPlayerVolume(PlayerVolume),
    /// Get the volume of the player.
    ///
    /// The daemon will respond to this with [`PlaybackResponse::PlayerVolume`] if
    /// successful.
    GetPlayerVolume,
    /// Set a track, a directory with tracks or an M3U8 playlist as the current queue.
    ///
    /// Tracks outside of the music library will be ignored.
    OpenURI(String),
}

/// Response originating from the playback module of the daemon used in
/// ed as an enum value for th[`Response`](crate::Response).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackResponse {
    PlaybackState(PlaybackState),
    LoopState(LoopState),
    PlaybackRate(PlaybackRate),
    ShuffleState(ShuffleState),
    Track(Option<Box<Track>>),
    PlayerVolume(PlayerVolume),
    PlayerPosition(Duration),
}

impl std::fmt::Display for PlaybackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaybackState(state) => write!(f, "{state}"),
            Self::LoopState(state) => write!(f, "{state}"),
            Self::PlaybackRate(rate) => write!(f, "{rate}"),
            Self::ShuffleState(state) => write!(f, "{state}"),
            Self::Track(track) => match track {
                Some(track) => write!(f, "{track}"),
                None => write!(f, "None"),
            },
            Self::PlayerVolume(volume) => write!(f, "{volume}"),
            Self::PlayerPosition(position) => write!(f, "{position:?}"),
        }
    }
}

/// Event originating from the playback module of the daemon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackEvent {
    /// Sent when the [playback state](PlaybackState) changes, for instance when the player is
    /// paused.
    PlaybackStateChanged(PlaybackState),
    /// Sent when the [loop state](LoopState) of the player changes.
    LoopStateChanged(LoopState),
    /// Sent when the [shuffle state](ShuffleState) of the player changes.
    ShuffleStateChanged(ShuffleState),
    /// Sent when the track queue changes.
    QueueChanged(Vec<Track>),
    /// Sent when the current track changes.
    PositionChanged(usize),
    /// Sent when the position of the player changes.
    PlayerPositionChanged(Duration),
    /// Sent when the [volume](PlayerVolume) of the player changes.
    PlayerVolumeChanged(PlayerVolume),
    /// Sent when the [playback rate](PlaybackRate) of the player changes.
    RateChanged(PlaybackRate),
}
