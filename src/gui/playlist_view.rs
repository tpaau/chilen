use std::{collections::HashSet, sync::Arc};

use iced::{
    Border, Element, Length, Padding, Task,
    widget::{button, column, container, text},
};
use log::error;

use crate::{
    gui::{
        self, Chilen, LoadingState, ROUNDING_REGULAR, SPACING_SMALL, SPACING_SMALLER, font,
        widgets::playlist_button::playlist_button,
    },
    music_lib::{create_playlist, state::Playlist},
};

#[derive(Debug, Clone)]
pub enum Message {
    Create,
    PlaylistsChanged(HashSet<Arc<Playlist>>),
    Open(Arc<Playlist>),
}

pub fn view(state: &Chilen) -> Element<'_, Message> {
    match &state.loading_state {
        LoadingState::Loading => text!("Loading...").color(state.theme.on_surface()).into(),
        LoadingState::Failed(e) => {
            container(text!("Load failed: {e}").color(state.theme.on_error()))
                .style(|_| {
                    container::Style::default()
                        .background(state.theme.error_container())
                        .border(Border::default().rounded(ROUNDING_REGULAR))
                })
                .width(Length::Fill)
                .padding(Padding::new(SPACING_SMALLER as f32))
                .into()
        }
        LoadingState::Loaded => column![
            text!("Playlists")
                .color(state.theme.on_surface())
                .size(font::SIZE_LARGE)
                .font(gui::font::font_bold()),
            iced::widget::scrollable(
                column({
                    // TODO: Proper sorting with support for numbers and non-ASCII characters
                    let mut playlists: Vec<_> = state.playlists.iter().collect();
                    playlists.sort_by_key(|pl| pl.name.clone());
                    playlists
                        .into_iter()
                        .map(|p| playlist_button(state, p).width(Length::Fill).into())
                })
                .spacing(SPACING_SMALLER)
            )
            .style(
                |_, status| crate::gui::theme::styles::scrollable::scrollable(status, &state.theme)
            )
            .height(Length::Fill)
            .width(Length::Fill),
            button("Hello!").on_press(Message::Create)
        ]
        .spacing(SPACING_SMALL)
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
        Message::Open(pl) => {
            // TODO: Open the playlist
            Task::none()
        }
    }
}
