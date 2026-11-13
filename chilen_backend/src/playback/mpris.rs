use std::{
    sync::{Arc, LazyLock, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use log::{error, info, trace};
use mpris_server::{PlayerInterface, Property, RootInterface, Server, Time};

use crate::{
    Event,
    music_lib::{Lyrics, Track},
    playback::{
        self, Error, LoopState, PlaybackState, PlayerVolume, SUPPORTED_MIME_TYPES, ShuffleState,
        SignedDuration, open_uri,
        state::{self, PLAYER_STATE},
    },
};

type MprisError = mpris_server::zbus::fdo::Error;
type MprisResult<T> = mpris_server::zbus::fdo::Result<T>;

pub struct MprisInterface {
    pub(crate) identity: String,
}

static SERVER: LazyLock<Arc<RwLock<Option<Server<MprisInterface>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

/// Configuration needed to (re)create the MPRIS server.
#[derive(Clone)]
struct ServerConfig {
    identity: String,
    identifier: String,
}

static SERVER_CONFIG: LazyLock<Arc<Mutex<Option<ServerConfig>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

/// `update_properties` fires every 100 ms during playback and keeps hitting a dead
/// server, so without a cooldown the restart would re-grab the bus name 10x/second.
const RESTART_COOLDOWN: Duration = Duration::from_secs(5);

static LAST_RESTART: LazyLock<Mutex<Option<Instant>>> = LazyLock::new(|| Mutex::new(None));

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

fn get_error(playback_error: Error) -> MprisError {
    match playback_error {
        Error::SeekNotSupported => MprisError::NotSupported(playback_error.to_string()),
        _ => MprisError::Failed(playback_error.to_string()),
    }
}

fn get_response(result: Result<(), Error>) -> MprisResult<()> {
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(get_error(e)),
    }
}

/// Sanitizes a tag-derived string for D-Bus/MPRIS transport.
fn sanitize_mpris_string(s: &str) -> String {
    s.chars().filter(|&c| c != '\0').collect()
}

fn sanitize_mpris_strings(s: &[Box<str>]) -> Vec<String> {
    s.iter().map(|s| sanitize_mpris_string(s)).collect()
}

pub(crate) fn get_meta(track: &Track, position: usize) -> mpris_server::Metadata {
    use mpris_server::{Time, builder::MetadataBuilder};

    let builder = MetadataBuilder::default()
        .length(Time::from_nanos(
            track.duration.as_nanos().try_into().unwrap_or(i64::MAX),
        ))
        .url(track.path.to_string_lossy())
        .artist(
            track
                .artists
                .as_ref()
                .map(|a| sanitize_mpris_strings(a))
                .unwrap_or(vec!["Unknown artist".to_string()]),
        )
        .title(sanitize_mpris_string(
            &track.title.clone().unwrap_or_default(),
        ))
        .album(sanitize_mpris_string(
            &track.album.clone().unwrap_or_default(),
        ))
        .genre(
            track
                .genres
                .as_ref()
                .map(|g| sanitize_mpris_strings(g))
                .unwrap_or(vec!["Unknown genre".to_string()]),
        )
        .comment(track.comment.as_ref().map(|c| sanitize_mpris_string(c)))
        .track_number(track.track.unwrap_or(0).try_into().unwrap_or(0))
        .disc_number(track.disc.unwrap_or(0).try_into().unwrap_or(0))
        .lyrics(sanitize_mpris_string(&match &track.lyrics {
            Some(lyrics) => {
                use lrc_rs::LyricsAccess;

                match lyrics {
                    Lyrics::Synced(synced) => synced.clone().to_unsynced(),
                    Lyrics::Unsynced { lyrics, reason: _ } => lyrics.to_string(),
                }
            }
            None => String::new(),
        }))
        .trackid(track.track_id(position));

    let builder = match &track.cover.cover() {
        Ok(cover) => builder.art_url(cover.to_string_lossy().to_string()),
        Err(_) => builder,
    };

    builder.build()
}

impl RootInterface for MprisInterface {
    async fn raise(&self) -> MprisResult<()> {
        crate::send_event(Event::Raise);
        Ok(())
    }

    async fn quit(&self) -> MprisResult<()> {
        crate::send_event(Event::Quit);
        Ok(())
    }

    async fn can_quit(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn fullscreen(&self) -> mpris_server::zbus::fdo::Result<bool> {
        // TODO: Fullscreen checking
        Err(mpris_server::zbus::fdo::Error::NotSupported(
            "TBD".to_string(),
        ))
    }

    async fn can_set_fullscreen(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn set_fullscreen(&self, fullscreen: bool) -> mpris_server::zbus::Result<()> {
        crate::send_event(Event::SetFullscreen(fullscreen));
        Ok(())
    }

    async fn can_raise(&self) -> mpris_server::zbus::fdo::Result<bool> {
        Ok(true)
    }

    async fn has_track_list(&self) -> mpris_server::zbus::fdo::Result<bool> {
        // TODO: Implement track lists for Mpris
        Ok(false)
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
        Ok(mpris_server::PlaybackRate::from(1))
    }

    async fn set_rate(&self, _rate: mpris_server::PlaybackRate) -> mpris_server::zbus::Result<()> {
        Err(mpris_server::zbus::Error::Unsupported)
    }

    async fn shuffle(&self) -> MprisResult<bool> {
        match playback::get_shuffle_state() {
            Ok(state) => Ok(state.enabled()),
            Err(e) => Err(MprisError::Failed(format!(
                "Cannot get the shuffle state: {e}"
            ))),
        }
    }

    async fn set_shuffle(&self, shuffle: bool) -> mpris_server::zbus::Result<()> {
        let shuffle_state = match shuffle {
            true => ShuffleState::On,
            false => ShuffleState::Off,
        };
        match playback::set_shuffle_state(shuffle_state) {
            Ok(_) => Ok(()),
            Err(e) => Err(mpris_server::zbus::Error::Failure(format!(
                "Cannot set the shuffle state: {e}"
            ))),
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
        Ok(mpris_server::PlaybackRate::from(1))
    }

    async fn maximum_rate(&self) -> MprisResult<mpris_server::PlaybackRate> {
        Ok(mpris_server::PlaybackRate::from(1))
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

/// Whether the error means the D-Bus connection is dead and the server must be restarted.
fn needs_restart(error: &mpris_server::zbus::Error) -> bool {
    use std::io::ErrorKind;
    match error {
        mpris_server::zbus::Error::InputOutput(io_err) => matches!(
            io_err.kind(),
            ErrorKind::BrokenPipe
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::NotConnected
                | ErrorKind::TimedOut
        ),
        mpris_server::zbus::Error::NameTaken => true,
        _ => false,
    }
}

/// Builds a fresh server, pushes the current playback state as the initial properties,
/// and swaps it into `SERVER`.
async fn start_server(identity: String, identifier: &str) -> Result<(), mpris_server::zbus::Error> {
    trace!("Starting the MPRIS server");
    let server = match Server::new(identifier, MprisInterface { identity }).await {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to start the MPRIS server: {e}");
            return Err(e);
        }
    };

    let state = {
        let state_guard = playback::state::PLAYER_STATE.read().unwrap();
        match playback::state::unwrap_state_ref(state_guard.as_ref()) {
            Ok(state) => state.clone(),
            Err(e) => {
                error!("Could not get the initial server properties: {e}");
                playback::state::PlayerState::default()
            }
        }
    };

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
    Ok(())
}

/// Restarts the MPRIS server after a fatal connection error.
fn restart_server() {
    let now = Instant::now();
    {
        let mut last_guard = LAST_RESTART.lock().unwrap();
        if let Some(last) = *last_guard
            && now.duration_since(last) < RESTART_COOLDOWN
        {
            trace!("MPRIS server restart suppressed by cooldown");
            return;
        }
        *last_guard = Some(now);
    }

    let config = match SERVER_CONFIG.lock().unwrap().clone() {
        Some(config) => config,
        None => {
            error!("Cannot restart the MPRIS server as it was never launched");
            return;
        }
    };

    info!("Attempting a restart of the MPRIS server");
    thread::spawn(move || {
        pollster::block_on(async {
            if let Err(e) = start_server(config.identity, &config.identifier).await {
                error!("Cannot restart the MPRIS server: {e}");
            }
        });
    });
}

pub(crate) fn launch_server(identity: String, identifier: String) {
    *SERVER_CONFIG.lock().unwrap() = Some(ServerConfig {
        identity: identity.clone(),
        identifier: identifier.clone(),
    });
    thread::spawn(move || {
        trace!("Starting the MPRIS server");
        pollster::block_on(async {
            if let Err(e) = start_server(identity, &identifier).await {
                error!("Cannot start the MPRIS server: {e}");
            }
        });
    });
}

pub(crate) fn update_properties(properties: Vec<Property>) {
    thread::spawn(move || {
        let result = {
            let mut server_guard = SERVER.write().unwrap();
            match server_guard.as_mut() {
                Some(server) => pollster::block_on(server.properties_changed(properties)),
                None => {
                    error!("Cannot update MPRIS properties, the server is not initialized");
                    return;
                }
            }
        };
        if let Err(e) = result {
            error!("Cannot update MPRIS properties: {e}");
            if needs_restart(&e) {
                restart_server();
            } else {
                info!("Not a connection error - not attempting a server restart");
            }
        }
    });
}

pub(crate) fn set_position(position: Duration) {
    thread::spawn(move || {
        let result = {
            let server_guard = SERVER.read().unwrap();
            match server_guard.as_ref() {
                Some(server) => pollster::block_on(server.emit(mpris_server::Signal::Seeked {
                    position: Time::from_nanos(position.as_nanos().try_into().unwrap_or(i64::MAX)),
                })),
                None => {
                    error!("Cannot update player position, the server is not initialized");
                    return;
                }
            }
        };
        if let Err(e) = result {
            error!("Could not update player position: {e}");
            if needs_restart(&e) {
                restart_server();
            } else {
                info!("Not a connection error - not attempting a server restart");
            }
        }
    });
}
