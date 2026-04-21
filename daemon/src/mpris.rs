use std::time::Duration;

use log::trace;
use mpipc::{
    LoopState, PlaybackError, PlaybackResponse, PlaybackState, PlayerVolume, SignedDuration,
};
use mpris_server::{PlayerInterface, RootInterface};

use crate::playback;

type MprisError = mpris_server::zbus::fdo::Error;
type MprisResult<T> = mpris_server::zbus::fdo::Result<T>;

pub struct MprisInterface {
    pub(crate) identity: String,
}

fn playback_state_2_mpris(playback_state: &PlaybackState) -> mpris_server::PlaybackStatus {
    match playback_state {
        PlaybackState::Playing => mpris_server::PlaybackStatus::Playing,
        PlaybackState::Paused => mpris_server::PlaybackStatus::Paused,
        PlaybackState::Stopped => mpris_server::PlaybackStatus::Stopped,
    }
}

fn loop_state_2_mpris(loop_state: &LoopState) -> mpris_server::LoopStatus {
    match loop_state {
        LoopState::Off => mpris_server::LoopStatus::None,
        LoopState::Track => mpris_server::LoopStatus::Track,
        LoopState::Playlist => mpris_server::LoopStatus::Playlist,
    }
}

fn loop_state_from_mpris(loop_state: &mpris_server::LoopStatus) -> LoopState {
    match loop_state {
        mpris_server::LoopStatus::None => LoopState::Off,
        mpris_server::LoopStatus::Track => LoopState::Track,
        mpris_server::LoopStatus::Playlist => LoopState::Playlist,
    }
}

fn get_error(playback_error: PlaybackError) -> MprisError {
    match playback_error {
        PlaybackError::SeekNotSupported => MprisError::NotSupported(playback_error.to_string()),
        _ => MprisError::Failed(playback_error.to_string()),
    }
}

fn get_response(result: Result<PlaybackResponse, PlaybackError>) -> MprisResult<()> {
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(get_error(e)),
    }
}

impl RootInterface for MprisInterface {
    async fn raise(&self) -> MprisResult<()> {
        // TODO: Implement raising for Mpris
        Err(MprisError::NotSupported(String::from(
            "Raise is not yet implemented",
        )))
    }

    async fn quit(&self) -> MprisResult<()> {
        // TODO: Implement quitting for Mpris clients' request
        Err(MprisError::NotSupported(String::from(
            "Quit is not yet implemented",
        )))
    }

    async fn can_quit(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(false) // TODO: Implement quitting for Mpris clients' request
    }

    async fn fullscreen(&self) -> mpris_server::zbus::fdo::Result<bool> {
        // TODO: Implement going fullscreen for Mpris clients
        Err(MprisError::NotSupported(String::from(
            "Fullscreen is not yet implemented",
        )))
    }

    async fn can_set_fullscreen(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(false) // TODO: Implement going fullscreen for Mpris clients
    }

    async fn set_fullscreen(&self, fullscreen: bool) -> mpris_server::zbus::Result<()> {
        Err(mpris_server::zbus::Error::Unsupported) // TODO: Implement going fullscreen for Mpris
        // clients
    }

    async fn can_raise(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(false) // TODO: Implement raising for Mpris
    }

    async fn has_track_list(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(false) // TODO: Implement track lists for Mpris
    }

    async fn identity(&self) -> mpris_server::zbus::fdo::Result<String> {
        Ok(self.identity.clone())
    }

    async fn desktop_entry(&self) -> mpris_server::zbus::fdo::Result<String> {
        // TODO: Implement returning the desktop entry
        Err(MprisError::NotSupported(String::from(
            "Desktop entries are not supported",
        )))
    }

    async fn supported_uri_schemes(&self) -> mpris_server::zbus::fdo::Result<Vec<String>> {
        Ok(vec![String::from("file")])
    }

    async fn supported_mime_types(&self) -> mpris_server::zbus::fdo::Result<Vec<String>> {
        // TODO: Fill in the mime types list
        Ok(vec![String::from("audio/mp3")])
    }
}

impl PlayerInterface for MprisInterface {
    async fn next(&self) -> MprisResult<()> {
        get_response(playback::run_command(playback::Command::Next))
    }

    async fn previous(&self) -> MprisResult<()> {
        get_response(playback::run_command(playback::Command::Previous))
    }

    async fn pause(&self) -> MprisResult<()> {
        get_response(playback::run_command(playback::Command::Pause))
    }

    async fn play_pause(&self) -> MprisResult<()> {
        get_response(playback::run_command(playback::Command::TogglePlaying))
    }

    async fn stop(&self) -> MprisResult<()> {
        get_response(playback::run_command(playback::Command::Stop))
    }

    async fn play(&self) -> MprisResult<()> {
        get_response(playback::run_command(playback::Command::Play(None)))
    }

    async fn seek(&self, offset: mpris_server::Time) -> MprisResult<()> {
        get_response(playback::run_command(playback::Command::Seek(
            SignedDuration::from_secs(offset.as_secs()),
        )))
    }

    async fn set_position(
        &self,
        track_id: mpris_server::TrackId,
        position: mpris_server::Time,
    ) -> MprisResult<()> {
        // TODO: Perform track_id validation
        get_response(playback::run_command(playback::Command::SetPlayerPosition(
            Duration::from_millis(
                position
                    .as_millis()
                    .clamp(0, u32::MAX.into())
                    .try_into()
                    .unwrap(),
            ),
        )))
    }

    async fn open_uri(&self, uri: String) -> MprisResult<()> {
        // TODO: Implement opening URIs
        trace!("Opening URI \"{uri}\"");
        Err(MprisError::NotSupported(String::from(
            "Opening URIs is not yet supported",
        )))
    }

    async fn playback_status(
        &self,
    ) -> mpris_server::zbus::fdo::Result<mpris_server::PlaybackStatus> {
        match playback::run_command(playback::Command::GetPlaybackState) {
            Ok(response) => match response {
                PlaybackResponse::PlaybackState(state) => Ok(playback_state_2_mpris(&state)),
                _ => Err(MprisError::Failed(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(get_error(e)),
        }
    }

    async fn loop_status(&self) -> MprisResult<mpris_server::LoopStatus> {
        match playback::run_command(playback::Command::GetLoopState) {
            Ok(response) => match response {
                PlaybackResponse::LoopState(state) => Ok(loop_state_2_mpris(&state)),
                _ => Err(MprisError::Failed(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(get_error(e)),
        }
    }

    async fn set_loop_status(
        &self,
        loop_status: mpris_server::LoopStatus,
    ) -> mpris_server::zbus::Result<()> {
        match playback::run_command(playback::Command::SetLoopState(loop_state_from_mpris(
            &loop_status,
        ))) {
            Ok(response) => match response {
                PlaybackResponse::Ok => Ok(()),
                _ => Err(mpris_server::zbus::Error::Failure(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(mpris_server::zbus::Error::Failure(format!(
                "Could not set the loop state: {e}"
            ))),
        }
    }

    async fn rate(&self) -> MprisResult<mpris_server::PlaybackRate> {
        match playback::run_command(playback::Command::GetRate) {
            Ok(response) => match response {
                PlaybackResponse::PlaybackRate(rate) => {
                    Ok(mpris_server::PlaybackRate::from(rate.get_value()))
                }
                _ => Err(MprisError::Failed(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(MprisError::Failed(format!(
                "Could not set the playback rate: {e}"
            ))),
        }
    }

    async fn set_rate(&self, rate: mpris_server::PlaybackRate) -> mpris_server::zbus::Result<()> {
        match playback::run_command(playback::Command::SetRate(rate)) {
            Ok(response) => match response {
                PlaybackResponse::Ok => Ok(()),
                _ => Err(mpris_server::zbus::Error::Failure(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => match e {
                PlaybackError::FixedRate => Err(mpris_server::zbus::Error::Unsupported),
                _ => Err(mpris_server::zbus::Error::Failure(format!(
                    "Could not set the playback rate: {e}"
                ))),
            },
        }
    }

    async fn shuffle(&self) -> MprisResult<bool> {
        match playback::run_command(playback::Command::GetShuffleState) {
            Ok(response) => match response {
                PlaybackResponse::ShuffleState(state) => Ok(state.into()),
                _ => Err(MprisError::Failed(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(MprisError::Failed(format!(
                "Could not get the shuffle state: {e}"
            ))),
        }
    }

    async fn set_shuffle(&self, shuffle: bool) -> mpris_server::zbus::Result<()> {
        match playback::run_command(playback::Command::SetShuffleState(shuffle.into())) {
            Ok(response) => match response {
                PlaybackResponse::Ok => Ok(()),
                _ => Err(mpris_server::zbus::Error::Failure(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => match e {
                PlaybackError::ShuffleNotSupported => Err(mpris_server::zbus::Error::Unsupported),
                _ => Err(mpris_server::zbus::Error::Failure(format!(
                    "Could not set the shuffle state: {e}"
                ))),
            },
        }
    }

    async fn metadata(&self) -> MprisResult<mpris_server::Metadata> {
        match playback::run_command(playback::Command::GetCurrentTrack) {
            Ok(response) => match response {
                PlaybackResponse::Track(maybe_track) => match maybe_track {
                    Some(track) => Ok(mpris_server::Metadata::builder()
                        .url(track.path.to_string_lossy())
                        .build()),
                    None => Ok(mpris_server::Metadata::builder().build()),
                },
                _ => Err(MprisError::Failed(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(MprisError::Failed(format!(
                "Could not get the current track: {e}"
            ))),
        }
    }

    async fn volume(&self) -> MprisResult<mpris_server::Volume> {
        match playback::run_command(playback::Command::GetPlayerVolume) {
            Ok(response) => match response {
                PlaybackResponse::PlayerVolume(volume) => {
                    Ok(mpris_server::Volume::from(volume.get()))
                }
                _ => Err(MprisError::Failed(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(MprisError::Failed(format!(
                "Could not get the player volume: {e}"
            ))),
        }
    }

    async fn set_volume(&self, volume: mpris_server::Volume) -> mpris_server::zbus::Result<()> {
        match playback::run_command(playback::Command::SetPlayerVolume(PlayerVolume::new(
            volume,
        ))) {
            Ok(response) => match response {
                PlaybackResponse::Ok => Ok(()),
                _ => Err(mpris_server::zbus::Error::Failure(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(mpris_server::zbus::Error::Failure(format!(
                "Could not set the player volume: {e}"
            ))),
        }
    }

    async fn position(&self) -> MprisResult<mpris_server::Time> {
        match playback::run_command(playback::Command::GetPlayerPosition) {
            Ok(response) => match response {
                PlaybackResponse::PlayerPosition(position) => Ok(mpris_server::Time::from_nanos(
                    position.as_nanos().try_into().unwrap_or(i64::MAX),
                )),
                _ => Err(MprisError::Failed(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(MprisError::Failed(format!(
                "Could not get the player position: {e}"
            ))),
        }
    }

    async fn minimum_rate(&self) -> MprisResult<mpris_server::PlaybackRate> {
        match playback::run_command(playback::Command::GetRate) {
            Ok(response) => match response {
                PlaybackResponse::PlaybackRate(rate) => {
                    Ok(mpris_server::PlaybackRate::from(rate.get_min()))
                }
                _ => Err(MprisError::Failed(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(MprisError::Failed(format!(
                "Could not get the maximum playback rate: {e}"
            ))),
        }
    }

    async fn maximum_rate(&self) -> MprisResult<mpris_server::PlaybackRate> {
        match playback::run_command(playback::Command::GetRate) {
            Ok(response) => match response {
                PlaybackResponse::PlaybackRate(rate) => {
                    Ok(mpris_server::PlaybackRate::from(rate.get_max()))
                }
                _ => Err(MprisError::Failed(format!(
                    "Got an unexpected response from the playback module: {response:?}"
                ))),
            },
            Err(e) => Err(MprisError::Failed(format!(
                "Could not get the maximum playback rate: {e}"
            ))),
        }
    }

    async fn can_go_next(&self) -> MprisResult<bool> {
        match playback::can_go_next() {
            Ok(can_go_next) => Ok(can_go_next),
            Err(e) => Err(MprisError::Failed(format!(
                "Could not check whether the player can skip to the next track: {e}"
            ))),
        }
    }

    async fn can_go_previous(&self) -> MprisResult<bool> {
        match playback::can_go_previous() {
            Ok(can_go_previous) => Ok(can_go_previous),
            Err(e) => Err(MprisError::Failed(format!(
                "Could not check whether the player can skip to the previous track: {e}"
            ))),
        }
    }

    async fn can_play(&self) -> MprisResult<bool> {
        match playback::can_play() {
            Ok(can_play) => Ok(can_play),
            Err(e) => Err(MprisError::Failed(format!(
                "Could not check whether the player can play: {e}"
            ))),
        }
    }

    async fn can_pause(&self) -> MprisResult<bool> {
        match playback::can_pause() {
            Ok(can_pause) => Ok(can_pause),
            Err(e) => Err(MprisError::Failed(format!(
                "Could not check whether the player can pause: {e}"
            ))),
        }
    }

    async fn can_seek(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn can_control(&self) -> MprisResult<bool> {
        Ok(true)
    }
}
