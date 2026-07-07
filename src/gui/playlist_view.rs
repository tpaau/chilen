use std::{collections::HashSet, sync::Arc};

use iced::{
    Border, Element, Length, Padding, Task,
    widget::{button, column, container, scrollable, text},
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
        LoadingState::Failed(e) => container(text!("Load failed: {e}").style(|_| text::Style {
            color: Some(state.theme.current().on_error_container),
        }))
        .style(|_| {
            container::Style::default()
                .background(state.theme.current().error_container)
                .border(Border::default().rounded(state.rounding.regular))
        })
        .width(Length::Fill)
        .padding(Padding::new(state.spacing.smaller as f32))
        .into(),
        LoadingState::Loaded => column![
            scrollable(column(state.playlists.iter().map(|p| {
                text!("Playlist \"{}\", tracks: {}", p.name, p.tracks.len()).into()
            })))
            .height(Length::Fill)
            .width(Length::Fill),
            button("Hello!").on_press(Message::Create)
        ]
        .height(Length::Fill)
        .width(Length::Fill)
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
