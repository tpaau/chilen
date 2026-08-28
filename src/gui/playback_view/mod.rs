mod bottom_panel;
mod playback_control;

use std::time::Duration;

use iced::{Element, Padding, Task};
use iced_widget::{column, container};
use log::error;

use crate::gui::{Chilen, SPACING_REGULAR, SPACING_SMALL};

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
}

// TODO: Save the last opened tab
// TODO: Add a setting to remove lyrics from the panel
#[derive(Default, PartialEq, Eq)]
pub enum Tab {
    Lyrics,
    #[default]
    Queue,
}

#[derive(Default)]
pub struct State {
    seek_slider_value: Option<f32>,
    tab: Tab,
}

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let padding = SPACING_SMALL;
    let width = 400.0;
    container(
        column![playback_control::view(state), bottom_panel::view(state)].spacing(SPACING_REGULAR),
    )
    .width(width + 2.0 * padding)
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
            let ratio = state
                .playback_view
                .seek_slider_value
                .unwrap_or_default()
                .clamp(0.0, 1.0);
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
    }
    Task::none()
}
