use std::{collections::HashSet, sync::Arc};

use iced::{
    Element, Task,
    widget::{button, column, text},
};
use log::error;

use crate::{
    gui::{Chilen, LoadingState},
    music_lib::{create_playlist, state::Playlist},
};

#[derive(Debug, Clone)]
pub enum Message {
    Create,
    PlaylistsChanged(HashSet<Arc<Playlist>>),
}

pub fn view(state: &Chilen) -> Element<'_, Message> {
    match &state.loading_state {
        LoadingState::Loading => text!("Loading...").into(),
        LoadingState::Failed(e) => text!("Loading failed: {e}").into(),
        LoadingState::Loaded => column![
            column(
                state.playlists.iter().map(|p| text!(
                    "Playlist \"{}\", tracks: {}",
                    p.name,
                    p.tracks.len()
                )
                .into())
            )
            .padding(12),
            button("Hello!").on_press(Message::Create)
        ]
        .into(),
    }
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::Create => {
            if let Err(e) = create_playlist(format!("Hello {}", state.playlists.len()), &None) {
                error!(
                    "Could not create a playlist, this shouldn't happen in the finished app: {e}"
                );
            }
            Task::none()
        }
        Message::PlaylistsChanged(playlists) => {
            state.loading_state = LoadingState::Loaded;
            state.playlists = playlists;
            Task::none()
        }
    }
}
