use std::{collections::HashSet, sync::Arc};

use iced::{
    Border, Element, Length, Padding, Shadow, Task, Vector,
    border::Radius,
    widget::{button, column, container, text},
};
use iced_widget::{bottom_right, stack};
use log::error;

use crate::{
    gui::{
        self, Chilen, LoadingState, ROUNDING_LARGE, ROUNDING_REGULAR, SPACING_SMALL,
        SPACING_SMALLER, font, icons,
        theme::{ColorScheme, styles},
        widgets::playlist_button::playlist_button,
    },
    music_lib::{create_playlist, state::Playlist},
};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleFabMenu,
    PlaylistsChanged(HashSet<Arc<Playlist>>),
    Open(Arc<Playlist>),
}

#[derive(Debug, Clone, Default)]
pub struct State {
    fab_menu_opened: bool,
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
        LoadingState::Loaded => {
            stack!(
                column![
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
                        |_, status| crate::gui::theme::styles::scrollable::scrollable(
                            status,
                            &state.theme
                        )
                    )
                    .height(Length::Fill)
                    .width(Length::Fill),
                ]
                .spacing(SPACING_SMALL)
                .height(Length::Fill)
                .width(Length::Fill),
                bottom_right(
                    button(
                        container(
                            text(match state.playlist_state.fab_menu_opened {
                                true => *icons::CLOSE,
                                false => *icons::ADD,
                            })
                            .font(icons::font())
                            .size(icons::SIZE_LARGE)
                        )
                        .padding(Padding::from(SPACING_SMALLER as f32))
                    )
                    .on_press(Message::ToggleFabMenu)
                    .style(|_, status| {
                        let mut style = styles::button::button(
                            status,
                            &state.theme,
                            styles::button::Style::Primary,
                        );
                        style.shadow = Shadow {
                            color: state.theme.shadow().scale_alpha(0.4),
                            offset: Vector::default(),
                            blur_radius: 6.0,
                        };
                        style.border.radius = Radius::from(ROUNDING_LARGE);
                        style
                    })
                ),
            )
            .into()
        }
    }
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::ToggleFabMenu => {
            state.playlist_state.fab_menu_opened = !state.playlist_state.fab_menu_opened;
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
