use iced::{
    Element, Task,
    widget::{button, column, text},
};
use log::error;

use crate::music_lib::create_playlist;

#[derive(Debug, Clone)]
pub enum Event {
    PlaylistsChanged(Vec<Playlist>),
    LoadFailed(String),
}

#[derive(Default)]
pub enum LoadingState {
    #[default]
    Loading,
    Failed(String),
    Loaded,
}

#[derive(Default)]
pub struct State {
    pub playlists: Vec<Playlist>,
    pub loading_state: LoadingState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Create,
    Event(Event),
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub name: String,
    pub num_tracks: usize,
}

pub fn view(state: &State) -> Element<'_, Message> {
    match &state.loading_state {
        LoadingState::Loading => text!("Loading...").into(),
        LoadingState::Failed(e) => text!("Loading failed: {e}").into(),
        LoadingState::Loaded => column![
            column(
                state.playlists.iter().map(|p| text!(
                    "Playlist \"{}\", tracks: {}",
                    p.name,
                    p.num_tracks
                )
                .into())
            )
            .padding(12),
            button("Hello!").on_press(Message::Create)
        ]
        .into(),
    }
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Create => {
            if let Err(e) = create_playlist(format!("Hello {}", state.playlists.len()), &None) {
                error!(
                    "Could not create a playlist, this shouldn't happen in the finished app: {e}"
                );
            }
            Task::none()
        }
        Message::Event(event) => match event {
            Event::PlaylistsChanged(playlists) => {
                state.loading_state = LoadingState::Loaded;
                state.playlists = playlists;
                Task::none()
            }
            Event::LoadFailed(e) => {
                state.loading_state = LoadingState::Failed(e);
                Task::none()
            }
        },
    }
}
