use std::{collections::HashSet, sync::Arc};

use iced::{
    Background, Border, Color, Element, Font, Length, Padding, Shadow, Task,
    font::Weight,
    widget::{
        button, column, container,
        scrollable::{self, Rail, Scroller},
        text,
    },
};
use log::error;

use crate::{
    gui::{Chilen, LoadingState, widgets::playlist_button::playlist_button},
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
        LoadingState::Loading => text!("Loading...")
            .color(state.theme.current().on_surface)
            .into(),
        LoadingState::Failed(e) => {
            container(text!("Load failed: {e}").color(state.theme.current().on_error))
                .style(|_| {
                    container::Style::default()
                        .background(state.theme.current().error_container)
                        .border(Border::default().rounded(state.rounding.regular))
                })
                .width(Length::Fill)
                .padding(Padding::new(state.spacing.smaller as f32))
                .into()
        }
        LoadingState::Loaded => column![
            text!("Playlists")
                .color(state.theme.current().on_surface)
                .size(state.font_size.large)
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }),
            iced::widget::scrollable(
                column(
                    state
                        .playlists
                        .iter()
                        .map(|p| { playlist_button(state, p).width(Length::Fill).into() })
                )
                .spacing(state.spacing.smaller)
            )
            .style(
                |_, status| crate::gui::theme::styles::scrollable::scrollable(status, &state.theme)
            )
            .height(Length::Fill)
            .width(Length::Fill),
            button("Hello!").on_press(Message::Create)
        ]
        .spacing(state.spacing.small)
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
        Message::Open(pl) => todo!(),
    }
}
