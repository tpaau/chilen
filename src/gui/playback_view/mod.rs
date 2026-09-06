mod bottom_panel;
mod playback_control;

use std::{sync::Arc, time::Duration};

use chilen_backend::{music_lib::Track, playback};
use iced::{Element, Padding, Task};
use iced_widget::{column, container};
use log::{error, trace, warn};

use crate::gui::{Chilen, SPACING_REGULAR, SPACING_SMALL, dialog};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleShuffle,
    Previous,
    TogglePlaying,
    Next,
    ToggleLooping,
    SeekSliderMoved(f32),
    SeekSliderReleased,
    SetPlayerPosition(Duration),
    OpenLyrics,
    OpenQueue,
    OpenAlbum(String),
    OpenArtist(String),
    PlayTrack(usize),
    TrackButtonPoppedIn(usize),
    TrackButtonPoppedOut(usize),
    RemoveFromQueue(usize),
    AddTrackToPlaylist(Arc<Track>),
}

// TODO: Save the last opened tab
#[derive(Default, PartialEq, Eq)]
pub enum Tab {
    Lyrics,
    #[default]
    Queue,
}

#[derive(Default)]
pub struct State {
    seek_slider_value: Option<f32>,
    pub visible_tracks: Vec<bool>,
    tab: Tab,
}

pub fn view<'a>(state: &'a Chilen, width: f32) -> Element<'a, Message> {
    let padding = SPACING_SMALL;
    container(
        column![
            playback_control::view(state),
            // FIX: This is a WORKAROUND.
            bottom_panel::view(state)
        ]
        .spacing(SPACING_REGULAR),
    )
    .width(width)
    .padding(Padding::from(padding))
    .into()
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::ToggleShuffle => {
            let _ = chilen_backend::playback::toggle_shuffle_state();
        }
        Message::Previous => {
            let _ = chilen_backend::playback::skip_previous();
        }
        Message::TogglePlaying => {
            if let Some(player_state) = state.player_state.as_mut() {
                match player_state.playback_state {
                    chilen_backend::playback::PlaybackState::Playing => {
                        player_state.playback_state =
                            chilen_backend::playback::PlaybackState::Paused
                    }
                    chilen_backend::playback::PlaybackState::Paused => {
                        player_state.playback_state =
                            chilen_backend::playback::PlaybackState::Playing
                    }
                    chilen_backend::playback::PlaybackState::Stopped => {
                        if player_state.can_play() {
                            player_state.playback_state =
                                chilen_backend::playback::PlaybackState::Playing
                        }
                    }
                }
            }
            let _ = chilen_backend::playback::toggle_playing();
        }
        Message::Next => {
            let _ = chilen_backend::playback::skip_next();
        }
        Message::ToggleLooping => {
            if let Some(player_state) = state.player_state.as_mut() {
                player_state.loop_state = player_state.loop_state.cycle();
            }
            let _ = chilen_backend::playback::cycle_loop_state();
        }
        Message::SeekSliderMoved(value) => state.playback_view.seek_slider_value = Some(value),
        Message::SeekSliderReleased => {
            if let Some(value) = state.playback_view.seek_slider_value {
                let ratio = value.clamp(0.0, 1.0);
                state.playback_view.seek_slider_value = None;
                let track_duration = state
                    .player_state
                    .as_ref()
                    .and_then(|p| p.current().map(|t| t.duration))
                    .unwrap_or_default();
                let position = Duration::from_secs_f32(track_duration.as_secs_f32() * ratio);
                if let Some(player_state) = state.player_state.as_mut() {
                    player_state.player_position = position;
                }
                let _ = chilen_backend::playback::set_player_position(position);
            }
        }
        Message::OpenLyrics => state.playback_view.tab = Tab::Lyrics,
        Message::OpenQueue => state.playback_view.tab = Tab::Queue,
        Message::SetPlayerPosition(position) => {
            let _ = chilen_backend::playback::set_player_position(position);
            if let Some(player_state) = state.player_state.as_mut() {
                player_state.player_position = position;
            }
        }
        Message::OpenAlbum(album) => {
            if let Some(lib) = &state.library {
                match lib.find_album(&album) {
                    Some(album) => state
                        .main_view
                        .nav_stack
                        .set_top(super::main_view::top_view::TopView::Album(album.clone())),
                    None => {
                        error!("Couldn't find the album \"{album}\", this should never happen!");
                    }
                }
            }
        }
        Message::OpenArtist(artist) => {
            if let Some(lib) = &state.library {
                match lib.find_artist(&artist) {
                    Some(artist) => state
                        .main_view
                        .nav_stack
                        .set_top(super::main_view::top_view::TopView::Artist(artist.clone())),
                    None => {
                        error!("Couldn't find the artist \"{artist}\", this should never happen!");
                    }
                }
            }
        }
        Message::PlayTrack(index) => {
            let _ = chilen_backend::playback::play(Some(index));
        }
        Message::TrackButtonPoppedIn(index) => {
            if index < state.playback_view.visible_tracks.len() {
                state.playback_view.visible_tracks[index] = true;
            } else {
                warn!(
                    "Track index {index} exceeds the range of the virtual list: {}",
                    state.playback_view.visible_tracks.len()
                );
            }
        }
        Message::TrackButtonPoppedOut(index) => {
            if index < state.playback_view.visible_tracks.len() {
                state.playback_view.visible_tracks[index] = false;
            } else {
                warn!(
                    "Track index {index} exceeds the range of the virtual list: {}",
                    state.playback_view.visible_tracks.len()
                );
            }
        }
        Message::RemoveFromQueue(index) => {
            let _ = chilen_backend::playback::remove_from_queue(vec![index]);
        }
        Message::AddTrackToPlaylist(track) => dialog::add_track_to_playlist(state, track),
    }
    Task::none()
}

pub fn handle_event(state: &mut Chilen, event: playback::Event) {
    if let Some(player_state) = state.player_state.as_mut() {
        if let playback::Event::TracksChanged {
            position: _,
            tracks,
            shuffled_indices: _,
        } = &event
        {
            state.playback_view.visible_tracks = tracks
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    state
                        .playback_view
                        .visible_tracks
                        .get(i)
                        .cloned()
                        .unwrap_or_default()
                })
                .collect();
        }
        player_state.handle_event(event);
    } else {
        match event {
            chilen_backend::playback::Event::StateInitialized(player_state) => {
                trace!("Initializing player state representation in the GUI");
                state.playback_view.visible_tracks = vec![false; player_state.tracks.len()];
                state.player_state = Some(player_state);
            }
            _ => {
                error!(
                    "Got a non-initializing event before the player state was initialized in the GUI!"
                );
            }
        }
    }
}
