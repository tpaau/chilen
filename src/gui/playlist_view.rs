use std::{env::home_dir, path::PathBuf, sync::Arc};

use chilen_backend::music_lib::state::Playlist;
use iced::{
    Alignment, Border, Element, Length, Padding, Task,
    border::Radius,
    widget::{column, container, text},
};
use iced_m3::{
    style::shadow,
    theme::ColorScheme,
    widget::drop_down_menu::{DropDownMenu, Placement},
};
use iced_widget::{bottom_right, center, row, space, stack};
use log::{error, info, trace};

use crate::gui::{
    self, Chilen, Dialog, LoadingState, ROUNDING_LARGE, ROUNDING_REGULAR, SPACING_SMALL,
    SPACING_SMALLER, font, icons, playlist_view, widgets::playlist_button::playlist_button,
};

#[derive(Debug, Clone)]
pub enum Message {
    ButtonPoppedIn(usize),
    ButtonPoppedOut(usize),
    OpenPlaylist(Arc<Playlist>),
    CreatePlaylist,
    ExportPlaylist(String),
    ImportPlaylist,
    OpenPlaylistImportDialog(Option<rfd::FileHandle>),
    OpenPlaylistRenameDialog { playlist: String, name: String },
    ConfirmPlaylistDeletion(Arc<Playlist>),
}

pub struct State {
    pub visible: Option<Vec<bool>>,
}

pub fn view(state: &Chilen) -> Element<'_, playlist_view::Message> {
    let heading = text!("Playlists")
        .color(state.theme.on_surface())
        .size(font::SIZE_LARGE)
        .font(gui::font::font_bold());

    let content = match &state.loading_state {
        LoadingState::Loading => center(text("Loading...")),
        LoadingState::Failed(e) => {
            container(text!("Load failed: {e}").color(state.theme.on_error()))
                .style(|_| {
                    container::Style::default()
                        .background(state.theme.error_container())
                        .border(Border::default().rounded(ROUNDING_REGULAR))
                })
                .width(Length::Fill)
                .padding(Padding::new(SPACING_SMALLER))
        }
        LoadingState::Loaded => {
            center(stack!(
                column![
                    iced::widget::scrollable(
                        column({
                            let mut playlists: Vec<_> =
                                state.library.as_ref().unwrap().playlists.iter().collect();
                            playlists.sort_by_key(|pl| pl.name.clone());
                            playlists
                                .into_iter()
                                .enumerate()
                                .map(|(i, p)| playlist_button(state, p, i))
                        })
                        .spacing(SPACING_SMALLER)
                    )
                    .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
                    .height(Length::Fill)
                    .width(Length::Fill),
                ]
                .spacing(SPACING_SMALL)
                .height(Length::Fill)
                .width(Length::Fill),
                bottom_right(
                    // TODO: The overlay should pass down mouse clicks in transparent mode too
                    DropDownMenu::new(
                        move |opened| container(center(
                            text(if opened { *icons::CLOSE } else { *icons::ADD })
                                .font(icons::filled())
                                .size(icons::SIZE_REGULAR)
                        ))
                        .style(move |_| container::Style {
                            text_color: Some(if opened {
                                state.theme.primary()
                            } else {
                                state.theme.on_primary()
                            }),
                            background: Some(iced::Background::Color(if opened {
                                state.theme.on_primary()
                            } else {
                                state.theme.primary()
                            })),
                            border: Border::default().rounded(if opened {
                                Radius::from(u32::MAX)
                            } else {
                                Radius::from(ROUNDING_LARGE)
                            }),
                            shadow: shadow(state.theme.shadow(), 0.4),
                            snap: true
                        })
                        .width(Length::Fixed(56.0))
                        .height(Length::Fixed(56.0))
                        .into(),
                        Some(
                            column![
                                iced::widget::button(center(
                                    row(vec![
                                        text(*icons::UPLOAD_FILE)
                                            .font(icons::filled())
                                            .size(icons::SIZE_SMALL)
                                            .into(),
                                        text("Import playlist").size(font::SIZE_REGULAR).into()
                                    ])
                                    .align_y(Alignment::Center)
                                    .spacing(8)
                                ))
                                .style(|_, status| {
                                    let mut style = iced_m3::style::button(
                                        status,
                                        &state.theme,
                                        iced_m3::style::Button::Primary,
                                    );
                                    style.border.radius = Radius::from(f32::MAX);
                                    style
                                })
                                .padding(Padding::from(16.0))
                                .height(Length::Fixed(56.0))
                                .width(Length::Shrink)
                                .on_press(Message::ImportPlaylist),
                                iced::widget::button(center(
                                    row(vec![
                                        text(*icons::PLAYLIST_ADD)
                                            .font(icons::filled())
                                            .size(icons::SIZE_REGULAR)
                                            .into(),
                                        text("New playlist").size(font::SIZE_REGULAR).into()
                                    ])
                                    .align_y(Alignment::Center)
                                    .spacing(8)
                                ))
                                .style(|_, status| {
                                    let mut style = iced_m3::style::button(
                                        status,
                                        &state.theme,
                                        iced_m3::style::Button::Primary,
                                    );
                                    style.border.radius = Radius::from(f32::MAX);
                                    style
                                })
                                .padding(Padding::from(16.0))
                                .height(Length::Fixed(56.0))
                                .width(Length::Shrink)
                                .on_press(Message::CreatePlaylist),
                                space().height(4.0)
                            ]
                            .align_x(Alignment::End)
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .spacing(4)
                        ),
                        Placement::TopLeft,
                    )
                    .menu_transparent(true)
                )
            ))
        }
    };
    column![heading, content].into()
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    if state.playlist_view.visible.is_none()
        && let Some(lib) = &state.library
    {
        state.playlist_view.visible = Some(vec![false; lib.playlists.len()]);
    }
    match message {
        Message::ButtonPoppedIn(i) => {
            let visible = state.playlist_view.visible.as_mut().unwrap();
            if let Some(val) = visible.get_mut(i) {
                *val = true
            } else {
                visible.push(true);
            }
        }
        Message::ButtonPoppedOut(i) => state.playlist_view.visible.as_mut().unwrap()[i] = false,
        Message::CreatePlaylist => {
            state.dialog = Dialog::CreatePlaylist(String::new());
        }
        Message::ExportPlaylist(name) => todo!(),
        Message::ImportPlaylist => {
            return Task::perform(
                rfd::AsyncFileDialog::new()
                    .add_filter("M3U8 Playlist File", &["m3u", "m3u8"])
                    .set_directory(home_dir().unwrap_or(PathBuf::from(".")))
                    .pick_file(),
                Message::OpenPlaylistImportDialog,
            );
        }
        Message::OpenPlaylistImportDialog(handle) => match handle {
            Some(handle) => {
                trace!("Showing import dialog for playlist {handle:?}");
                state.dialog = Dialog::ImportPlaylist(String::new(), handle);
            }
            None => info!("Didn't get the file handle, guessing the user cancelled the import"),
        },
        Message::OpenPlaylistRenameDialog { playlist, name } => {
            state.dialog = Dialog::RenamePlaylist { playlist, name }
        }
        Message::ConfirmPlaylistDeletion(playlist) => {
            if playlist.tracks.is_empty() {
                if let Err(e) =
                    chilen_backend::music_lib::delete_playlists(vec![playlist.name.clone()])
                {
                    error!("Couldn't delete playlist: {e}");
                    state.dialog = Dialog::Error(format!("Couldn't delete playlist: {e}"))
                }
            } else {
                state.dialog = Dialog::DeletePlaylist(playlist)
            }
        }
        Message::OpenPlaylist(pl) => todo!(),
    }
    Task::none()
}
