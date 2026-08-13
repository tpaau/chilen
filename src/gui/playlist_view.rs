use std::{env::home_dir, path::PathBuf, sync::Arc};

use chilen_backend::music_lib::state::Playlist;
use iced::{
    Border, Element, Length, Padding, Task, padding,
    widget::{column, container, text},
};
use iced_m3::{theme::ColorScheme, widget::fab_menu};
use iced_widget::{bottom_right, center, stack};
use log::{debug, error, info, trace};

use crate::gui::{
    self, Chilen, Dialog, LoadingState, ROUNDING_REGULAR, SPACING_SMALL, SPACING_SMALLER, font,
    icons, main_view::top_view::TopView, playlist_view, widgets::playlist_button::playlist_button,
};

#[derive(Debug, Clone)]
pub enum Message {
    ButtonPoppedIn(usize),
    ButtonPoppedOut(usize),
    OpenPlaylist(Arc<Playlist>),
    CreatePlaylist,
    ExportPlaylist(String),
    SaveExportedPlaylist(String, Option<rfd::FileHandle>),
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
        LoadingState::Loaded => center(stack!(
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
                fab_menu(
                    vec![
                        iced_m3::widget::fab_menu::Entry {
                            message: Message::ImportPlaylist,
                            label: "Import playlist",
                            icon: Some(&*icons::UPLOAD_FILE)
                        },
                        iced_m3::widget::fab_menu::Entry {
                            message: Message::CreatePlaylist,
                            label: "New playlist",
                            icon: Some(&*icons::PLAYLIST_ADD)
                        }
                    ],
                    &|opened| if opened { *icons::CLOSE } else { *icons::ADD },
                    &state.theme
                )
                .icon_font(icons::filled()),
            )
            .padding(padding::bottom(SPACING_SMALL))
        )),
    };
    column![heading, content].spacing(SPACING_SMALLER).into()
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
        Message::ExportPlaylist(name) => {
            return Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_directory(home_dir().unwrap_or(PathBuf::from(".")))
                    .set_file_name(format!("{name}.m3u8"))
                    .save_file(),
                |maybe_handle| Message::SaveExportedPlaylist(name, maybe_handle),
            );
        }
        Message::SaveExportedPlaylist(name, file_handle) => match file_handle {
            Some(handle) => {
                if let Err(e) =
                    chilen_backend::music_lib::export_playlist_to_m3u8(&name, handle.path())
                {
                    let msg = format!(
                        "Couldn't export playlist {name} to {:?}: {e}",
                        handle.path()
                    );
                    error!("{msg}");
                    state.dialog = Dialog::Error(msg);
                }
            }
            None => debug!("User cancelled the export operation"),
        },
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
        Message::OpenPlaylist(pl) => state.main_view.nav_stack.navigate(TopView::Playlist(pl)),
    }
    Task::none()
}
