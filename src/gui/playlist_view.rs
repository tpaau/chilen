use std::{env::home_dir, path::PathBuf, sync::Arc};

use chilen_backend::music_lib::Playlist;
use iced::{
    Border, Element, Length, Padding, Task, padding,
    widget::{column, container, text},
};
use iced_m3::{theme::ColorScheme, widget::fab_menu};
use iced_widget::{bottom_right, center, responsive, space, stack};
use log::{debug, error, info, trace};

use crate::gui::{
    self, Chilen, Dialog, LoadingState, ROUNDING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
    common_actions, font, icons,
    main_view::top_view::TopView,
    playlist_view,
    widget::{
        list::{BUTTON_HEIGHT, BUTTON_SPACING, playlist_button::playlist_button},
        virtual_list::VirtualList,
    },
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
    PlayPlaylist(Arc<Playlist>),
    ShufflePlaylist(Arc<Playlist>),
    AddPlaylistToQueue(Arc<Playlist>),
}

#[derive(Default)]
pub struct State {
    pub visible: Option<Vec<bool>>,
}

pub fn view(state: &Chilen, width: f32) -> Element<'_, playlist_view::Message> {
    let heading = text!("Playlists")
        .color(state.theme.on_surface())
        .size(font::SIZE_LARGE)
        .font(gui::font::font_bold());

    let content = if let Some(lib) = &state.library {
        let base = responsive(move |size| {
            {
                iced::widget::scrollable({
                    let highlighted_playlist_name = state.player_state.as_ref().and_then(|p| {
                        if let chilen_backend::playback::QueueSource::Playlist { name } =
                            &p.queue_source
                        {
                            Some(name)
                        } else {
                            None
                        }
                    });

                    VirtualList {
                        model: lib.playlists.iter().enumerate(),
                        delegate: Box::new(move |(index, playlist)| {
                            playlist_button(
                                state,
                                playlist,
                                index,
                                highlighted_playlist_name
                                    .map(|name| *name == playlist.name)
                                    .unwrap_or_default(),
                            )
                        }),
                        delegate_height: BUTTON_HEIGHT,
                        visibilities: state.playlist_view.visible.as_deref().unwrap_or(&[]),
                        list: Box::new(move |mut content| {
                            if size.height < (lib.playlists.len() + 1) as f32 * BUTTON_HEIGHT {
                                content.push(space().height(BUTTON_HEIGHT).into())
                            }
                            column(content).spacing(BUTTON_SPACING).into()
                        }),
                        on_show: Box::new(Message::ButtonPoppedIn),
                        on_hide: Box::new(Message::ButtonPoppedOut),
                    }
                })
                .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
                .height(Length::Fill)
                .width(Length::Fill)
                .into()
            }
        });

        let fab = {
            bottom_right(
                fab_menu(
                    vec![
                        iced_m3::widget::fab_menu::Entry {
                            message: Message::ImportPlaylist,
                            label: "Import playlist",
                            icon: Some(&*icons::UPLOAD_FILE),
                        },
                        iced_m3::widget::fab_menu::Entry {
                            message: Message::CreatePlaylist,
                            label: "New playlist",
                            icon: Some(&*icons::PLAYLIST_ADD),
                        },
                    ],
                    &|opened| if opened { *icons::CLOSE } else { *icons::ADD },
                    &state.theme,
                )
                .icon_font(icons::filled()),
            )
            .padding(padding::bottom(SPACING_SMALL))
        };

        center(stack!(base, fab))
    } else {
        match &state.loading_state {
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
            LoadingState::Loaded => container(
                text!(
                    "The loading status is {:?}, but the library is not initialized!",
                    state.loading_state
                )
                .color(state.theme.on_error()),
            )
            .style(|_| {
                container::Style::default()
                    .background(state.theme.error_container())
                    .border(Border::default().rounded(ROUNDING_REGULAR))
            })
            .width(Length::Fill)
            .padding(Padding::new(SPACING_SMALLER)),
        }
    };

    container(column![heading, content].spacing(SPACING_SMALLER))
        .padding(padding::horizontal(SPACING_SMALL).top(SPACING_SMALL))
        .width(width)
        .height(Length::Fill)
        .into()
}

pub fn reload(state: &mut Chilen) {
    if let Some(lib) = &state.library {
        if let Some(visible) = &state.playlist_view.visible {
            state.playlist_view.visible = Some(
                lib.playlists
                    .iter()
                    .enumerate()
                    .map(|(i, _)| visible.get(i).copied().unwrap_or_default())
                    .collect(),
            );
        } else {
            state.playlist_view.visible = Some(vec![false; lib.playlists.len()])
        }
    } else {
        state.playlist_view.visible = None;
    }
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::ButtonPoppedIn(i) => {
            if let Some(visible) = state.playlist_view.visible.as_mut()
                && let Some(val) = visible.get_mut(i)
            {
                *val = true
            } else {
                error!("Visibilities not initialized!")
            }
        }
        Message::ButtonPoppedOut(i) => {
            if let Some(visible) = state.playlist_view.visible.as_mut()
                && let Some(val) = visible.get_mut(i)
            {
                *val = false
            } else {
                error!("Visibilities not initialized!")
            }
        }
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
        Message::OpenPlaylist(pl) => state.main_view.nav_stack.set_top(TopView::Playlist(pl)),
        Message::PlayPlaylist(playlist) => {
            common_actions::play_playlist(playlist, None);
        }
        Message::ShufflePlaylist(playlist) => {
            common_actions::play_playlist(playlist, None);
        }
        Message::AddPlaylistToQueue(playlist) => {
            common_actions::append_tracks_to_queue(playlist.tracks.clone())
        }
    }
    Task::none()
}
