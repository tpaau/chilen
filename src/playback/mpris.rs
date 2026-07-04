use std::{
    sync::{Arc, LazyLock, RwLock},
    thread,
    time::Duration,
};

use chilen_ipc::playback::{LoopState, PlaybackState, PlayerVolume, SignedDuration};
use log::{error, trace};
use mpris_server::{PlayerInterface, Property, RootInterface, Server, Time};

use crate::playback::{
    self, SUPPORTED_MIME_TYPES, open_uri,
    state::{self, PLAYER_STATE},
};

type MprisError = mpris_server::zbus::fdo::Error;
type MprisResult<T> = mpris_server::zbus::fdo::Result<T>;

pub struct MprisInterface {
    pub(crate) identity: String,
}

static SERVER: LazyLock<Arc<RwLock<Option<Server<MprisInterface>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub(crate) fn playback_state_2_mpris(
    playback_state: &PlaybackState,
) -> mpris_server::PlaybackStatus {
    match playback_state {
        PlaybackState::Playing => mpris_server::PlaybackStatus::Playing,
        PlaybackState::Paused => mpris_server::PlaybackStatus::Paused,
        PlaybackState::Stopped => mpris_server::PlaybackStatus::Stopped,
    }
}

pub(crate) fn loop_state_2_mpris(loop_state: &LoopState) -> mpris_server::LoopStatus {
    match loop_state {
        LoopState::Off => mpris_server::LoopStatus::None,
        LoopState::Track => mpris_server::LoopStatus::Track,
        LoopState::Playlist => mpris_server::LoopStatus::Playlist,
    }
}

pub(crate) fn loop_state_from_mpris(loop_state: &mpris_server::LoopStatus) -> LoopState {
    match loop_state {
        mpris_server::LoopStatus::None => LoopState::Off,
        mpris_server::LoopStatus::Track => LoopState::Track,
        mpris_server::LoopStatus::Playlist => LoopState::Playlist,
    }
}

fn get_error(playback_error: chilen_ipc::Error) -> MprisError {
    match playback_error {
        chilen_ipc::Error::SeekNotSupported => MprisError::NotSupported(playback_error.to_string()),
        _ => MprisError::Failed(playback_error.to_string()),
    }
}

fn get_response(result: Result<(), chilen_ipc::Error>) -> MprisResult<()> {
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(get_error(e)),
    }
}

impl RootInterface for MprisInterface {
    async fn raise(&self) -> MprisResult<()> {
        // TODO: Raising
        Err(MprisError::NotSupported("TBD".to_string()))
    }

    async fn quit(&self) -> MprisResult<()> {
        // TODO: Quit
        Err(MprisError::NotSupported("TBD".to_string()))
    }

    async fn can_quit(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn fullscreen(&self) -> mpris_server::zbus::fdo::Result<bool> {
        // TODO: Fullscreen toggling
        Err(mpris_server::zbus::fdo::Error::NotSupported(
            "TBD".to_string(),
        ))
    }

    async fn can_set_fullscreen(&self) -> mpris_server::zbus::fdo::Result<bool> {
        // TODO: Fullscreen toggling
        Err(mpris_server::zbus::fdo::Error::NotSupported(
            "TBD".to_string(),
        ))
    }

    async fn set_fullscreen(&self, fullscreen: bool) -> mpris_server::zbus::Result<()> {
        // TODO: Fullscreen toggling
        Err(mpris_server::zbus::Error::Unsupported)
    }

    async fn can_raise(&self) -> mpris_server::zbus::fdo::Result<bool> {
        // TODO: Raising
        Err(mpris_server::zbus::fdo::Error::NotSupported(
            "TBD".to_string(),
        ))
    }

    async fn has_track_list(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(false) // TODO: Implement track lists for Mpris
    }

    async fn identity(&self) -> mpris_server::zbus::fdo::Result<String> {
        Ok(self.identity.clone())
    }

    async fn desktop_entry(&self) -> mpris_server::zbus::fdo::Result<String> {
        // TODO: Desktop entry
        Err(mpris_server::zbus::fdo::Error::NotSupported(
            "TBD".to_string(),
        ))
    }

    async fn supported_uri_schemes(&self) -> mpris_server::zbus::fdo::Result<Vec<String>> {
        Ok(vec![String::from("file")])
    }

    async fn supported_mime_types(&self) -> mpris_server::zbus::fdo::Result<Vec<String>> {
        Ok(SUPPORTED_MIME_TYPES.to_vec())
    }
}

impl PlayerInterface for MprisInterface {
    async fn next(&self) -> MprisResult<()> {
        get_response(playback::skip_next())
    }

    async fn previous(&self) -> MprisResult<()> {
        get_response(playback::skip_previous())
    }

    async fn pause(&self) -> MprisResult<()> {
        get_response(playback::pause())
    }

    async fn play_pause(&self) -> MprisResult<()> {
        get_response(playback::toggle_playing())
    }

    async fn stop(&self) -> MprisResult<()> {
        get_response(playback::stop())
    }

    async fn play(&self) -> MprisResult<()> {
        get_response(playback::play(None))
    }

    async fn seek(&self, offset: mpris_server::Time) -> MprisResult<()> {
        get_response(playback::seek(SignedDuration::from_secs(offset.as_secs())))
    }

    async fn set_position(
        &self,
        track_id: mpris_server::TrackId,
        position: mpris_server::Time,
    ) -> MprisResult<()> {
        let meta = match self.metadata().await {
            Ok(meta) => meta,
            Err(e) => {
                return Err(MprisError::Failed(format!(
                    "Could not get the metadata: {e}"
                )));
            }
        };
        if Some(track_id) != meta.trackid() {
            return Err(MprisError::InvalidArgs(
                "The track id provided doesn't match with the current track id".to_string(),
            ));
        }
        get_response(playback::set_player_position(Duration::from_millis(
            position
                .as_millis()
                .clamp(0, u32::MAX.into())
                .try_into()
                .unwrap(),
        )))
    }

    async fn open_uri(&self, uri: String) -> MprisResult<()> {
        match open_uri(uri.into()) {
            Ok(_) => Ok(()),
            Err(e) => Err(MprisError::Failed(e.to_string())),
        }
    }

    async fn playback_status(&self) -> MprisResult<mpris_server::PlaybackStatus> {
        match playback::get_playback_state() {
            Ok(state) => Ok(playback_state_2_mpris(&state)),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot get the playback state: {e}"
            ))),
        }
    }

    async fn loop_status(&self) -> MprisResult<mpris_server::LoopStatus> {
        match playback::get_loop_state() {
            Ok(state) => Ok(loop_state_2_mpris(&state)),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot get the loop state: {e}"
            ))),
        }
    }

    async fn set_loop_status(
        &self,
        loop_status: mpris_server::LoopStatus,
    ) -> mpris_server::zbus::Result<()> {
        match playback::set_loop_state(loop_state_from_mpris(&loop_status)) {
            Ok(_) => Ok(()),
            Err(e) => Err(mpris_server::zbus::Error::Failure(format!(
                "Cannot set the loop state: {e}"
            ))),
        }
    }

    async fn rate(&self) -> MprisResult<mpris_server::PlaybackRate> {
        match playback::get_rate() {
            Ok(rate) => Ok(mpris_server::PlaybackRate::from(rate.get_value())),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot set the playback rate: {e}"
            ))),
        }
    }

    async fn set_rate(&self, rate: mpris_server::PlaybackRate) -> mpris_server::zbus::Result<()> {
        match playback::set_rate(rate) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                chilen_ipc::Error::FixedRate => Err(mpris_server::zbus::Error::Unsupported),
                _ => Err(mpris_server::zbus::Error::Failure(format!(
                    "Cannot set the playback rate: {e}"
                ))),
            },
        }
    }

    async fn shuffle(&self) -> MprisResult<bool> {
        match playback::get_shuffle_state() {
            Ok(state) => Ok(state.into()),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot get the shuffle state: {e}"
            ))),
        }
    }

    async fn set_shuffle(&self, shuffle: bool) -> mpris_server::zbus::Result<()> {
        match playback::set_shuffle_state(shuffle.into()) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                chilen_ipc::Error::ShuffleNotSupported => {
                    Err(mpris_server::zbus::Error::Unsupported)
                }
                _ => Err(mpris_server::zbus::Error::Failure(format!(
                    "Cannot set the shuffle state: {e}"
                ))),
            },
        }
    }

    async fn metadata(&self) -> MprisResult<mpris_server::Metadata> {
        match playback::get_current_meta() {
            Ok(maybe_meta) => match maybe_meta {
                Some(meta) => Ok(meta),
                None => Ok(mpris_server::Metadata::new()),
            },
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot get the current metadata: {e}"
            ))),
        }
    }

    async fn volume(&self) -> MprisResult<mpris_server::Volume> {
        match playback::get_player_volume() {
            Ok(volume) => Ok(mpris_server::Volume::from(volume.get())),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot get the player volume: {e}"
            ))),
        }
    }

    async fn set_volume(&self, volume: mpris_server::Volume) -> mpris_server::zbus::Result<()> {
        match playback::set_player_volume(PlayerVolume::new(volume)) {
            Ok(_) => Ok(()),
            Err(e) => Err(mpris_server::zbus::Error::Failure(format!(
                "Cannot set the player volume: {e}"
            ))),
        }
    }

    async fn position(&self) -> MprisResult<mpris_server::Time> {
        match playback::get_player_position() {
            Ok(pos) => Ok(mpris_server::Time::from_nanos(
                pos.as_nanos().try_into().unwrap_or(i64::MAX),
            )),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot get the player position: {e}"
            ))),
        }
    }

    async fn minimum_rate(&self) -> MprisResult<mpris_server::PlaybackRate> {
        match playback::get_rate() {
            Ok(rate) => Ok(mpris_server::PlaybackRate::from(rate.get_min())),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot get the maximum playback rate: {e}"
            ))),
        }
    }

    async fn maximum_rate(&self) -> MprisResult<mpris_server::PlaybackRate> {
        match playback::get_rate() {
            Ok(rate) => Ok(mpris_server::PlaybackRate::from(rate.get_max())),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot get the maximum playback rate: {e}"
            ))),
        }
    }

    async fn can_go_next(&self) -> MprisResult<bool> {
        match playback::can_go_next() {
            Ok(can_go_next) => Ok(can_go_next),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot check whether the player can skip to the next track: {e}"
            ))),
        }
    }

    async fn can_go_previous(&self) -> MprisResult<bool> {
        match playback::can_go_previous() {
            Ok(can_go_previous) => Ok(can_go_previous),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot check whether the player can skip to the previous track: {e}"
            ))),
        }
    }

    async fn can_play(&self) -> MprisResult<bool> {
        match playback::can_play() {
            Ok(can_play) => Ok(can_play),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot check whether the player can play: {e}"
            ))),
        }
    }

    async fn can_pause(&self) -> MprisResult<bool> {
        match playback::can_pause() {
            Ok(can_pause) => Ok(can_pause),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot check whether the player can pause: {e}"
            ))),
        }
    }

    async fn can_seek(&self) -> mpris_server::zbus::fdo::Result<bool> {
        let state_guard = PLAYER_STATE.read().unwrap();
        match state::unwrap_state_ref(state_guard.as_ref()) {
            Ok(state) => Ok(state.can_seek()),
            Err(e) => {
                error!("Could not check if the player can seek: {e}");
                Ok(false)
            }
        }
    }

    async fn can_control(&self) -> MprisResult<bool> {
        Ok(true)
    }
}

pub(crate) fn launch_server(identity: String, bus_name_suffix: String) {
    thread::spawn(move || {
        trace!("Starting the MPRIS server");
        let bus_name_suffix = bus_name_suffix.clone();
        let interface = MprisInterface {
            identity: identity.clone(),
        };

        smol::block_on(async {
            let server = match Server::new(&bus_name_suffix, interface).await {
                Ok(server) => server,
                Err(e) => {
                    error!("Cannot start the MPRIS server: {e}");
                    return;
                }
            };

            let state;
            {
                let state_guard = playback::state::PLAYER_STATE.read().unwrap();
                state = match playback::state::unwrap_state_ref(state_guard.as_ref()) {
                    Ok(state) => state.clone(),
                    Err(e) => {
                        error!("Could not get the initial server properties: {e}");
                        playback::state::PlayerState::default()
                    }
                };
                drop(state_guard);
            }

            if let Err(e) = server
                .properties_changed(state.get_mpris_properties())
                .await
            {
                error!("Could not set the initial server properties: {e}");
            } else {
                trace!("Successfully set the initial server properties");
            }

            let mut server_guard = SERVER.write().unwrap();
            *server_guard = Some(server);
        });
    });
}

pub(crate) fn update_properties(properties: Vec<Property>) {
    thread::spawn(move || {
        let mut server_guard = SERVER.write().unwrap();
        let server = match server_guard.as_mut() {
            Some(server) => server,
            None => {
                error!("Cannot update MPRIS properties, the server is not initialized");
                return;
            }
        };
        smol::block_on(async {
            if let Err(e) = server.properties_changed(properties).await {
                error!("Cannot update MPRIS properties: {e}");
            }
        })
    });
}

pub(crate) fn set_position(position: Duration) {
    trace!("Setting position to {position:?}");
    thread::spawn(move || {
        let server_guard = SERVER.write().unwrap();
        let server = match server_guard.as_ref() {
            Some(server) => server,
            None => {
                error!("Cannot update player position, the server is not initialized");
                return;
            }
        };
        smol::block_on(async {
            if let Err(e) = server
                .emit(mpris_server::Signal::Seeked {
                    position: Time::from_nanos(position.as_nanos().try_into().unwrap_or(i64::MAX)),
                })
                .await
            {
                error!("Could not update player position: {e}");
            }
        })
    });
}
