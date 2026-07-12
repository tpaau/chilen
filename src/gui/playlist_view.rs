use std::{collections::HashSet, sync::Arc};

use iced::{
    Alignment, Border, Element, Length, Padding, Shadow, Task, Vector,
    border::Radius,
    widget::{button, column, container, text},
};
use iced_widget::{bottom_right, center, right, row, stack};

use crate::{
    gui::{
        self, Chilen, LoadingState, ROUNDING_LARGE, ROUNDING_REGULAR, SPACING_SMALL,
        SPACING_SMALLER, font, icons,
        theme::{ColorScheme, styles},
        widgets::playlist_button::playlist_button,
    },
    music_lib::state::Playlist,
};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleFabMenu,
    PlaylistsChanged(HashSet<Arc<Playlist>>),
    Open(Arc<Playlist>),
    CreatePlaylist,
    ImportPlaylist,
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
                bottom_right({
                    let button = button(center(
                        text(match state.playlist_state.fab_menu_opened {
                            true => *icons::CLOSE,
                            false => *icons::ADD,
                        })
                        .font(icons::font())
                        .size(icons::SIZE_REGULAR),
                    ))
                    .on_press(Message::ToggleFabMenu)
                    .width(Length::Fixed(56.0))
                    .height(Length::Fixed(56.0))
                    .style(|_, status| {
                        let mut style = styles::button::button(
                            status,
                            &state.theme,
                            if state.playlist_state.fab_menu_opened {
                                styles::button::Style::InversePrimary
                            } else {
                                styles::button::Style::Primary
                            },
                        );
                        style.shadow = Shadow {
                            color: state.theme.shadow().scale_alpha(0.4),
                            offset: Vector::new(0.0, 2.0),
                            blur_radius: 6.0,
                        };
                        style.border.radius = if state.playlist_state.fab_menu_opened {
                            Radius::from(u32::MAX)
                        } else {
                            Radius::from(ROUNDING_LARGE)
                        };
                        style
                    });
                    // TODO: Move this to a dedicated widget
                    // TODO: Close the FAB menu when the user clicks outside of it
                    match state.playlist_state.fab_menu_opened {
                        true => column![
                            column![
                                right(
                                    iced::widget::button(center(
                                        row(vec![
                                            text(*icons::PLAYLIST_ADD)
                                                .font(icons::font())
                                                .size(icons::SIZE_SMALLER)
                                                .into(),
                                            text("New playlist").size(font::SIZE_REGULAR).into()
                                        ])
                                        .align_y(Alignment::Center)
                                        .spacing(8)
                                    ))
                                    .style(|_, status| {
                                        let mut style = styles::button::button(
                                            status,
                                            &state.theme,
                                            styles::button::Style::Primary,
                                        );
                                        style.border.radius = Radius::from(f32::MAX);
                                        style
                                    })
                                    .padding(Padding::from(16.0))
                                    .height(Length::Fixed(56.0))
                                    .width(Length::Shrink)
                                    .on_press(Message::CreatePlaylist)
                                ),
                                right(
                                    iced::widget::button(center(
                                        row(vec![
                                            text(*icons::UPLOAD_FILE)
                                                .font(icons::font())
                                                .size(icons::SIZE_SMALLER)
                                                .into(),
                                            text("Import playlist").size(font::SIZE_REGULAR).into()
                                        ])
                                        .align_y(Alignment::Center)
                                        .spacing(8)
                                    ))
                                    .style(|_, status| {
                                        let mut style = styles::button::button(
                                            status,
                                            &state.theme,
                                            styles::button::Style::Primary,
                                        );
                                        style.border.radius = Radius::from(f32::MAX);
                                        style
                                    })
                                    .padding(Padding::from(16.0))
                                    .height(Length::Fixed(56.0))
                                    .width(Length::Shrink)
                                    .on_press(Message::ImportPlaylist)
                                ),
                            ]
                            .spacing(4),
                            right(button),
                        ]
                        .spacing(SPACING_SMALL),
                        false => column![button],
                    }
                }),
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
        Message::CreatePlaylist => todo!(),
        Message::ImportPlaylist => todo!(),
    }
}
