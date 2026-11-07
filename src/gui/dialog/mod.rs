mod add_tracks_to_playlist;
mod loading;

use std::{path::PathBuf, sync::Arc};

use chilen_backend::music_lib::{Playlist, Progress, Track};
use dirs::home_dir;
use iced::{Element, Length, Task, border::Radius};
use iced_m3::{theme::ColorScheme, widget::dialog};
use iced_widget::container;
use log::{error, info, trace};

use crate::gui::{
    Chilen, ROUNDING_SMALL,
    event_loop::{self, library},
    font::{self, text},
    settings,
};

pub enum Dialog {
    None,
    CreatePlaylist(String),
    ImportPlaylist(String, rfd::FileHandle),
    Error(String),
    RenamePlaylist {
        playlist: Arc<Playlist>,
        name: String,
    },
    DeletePlaylist(Arc<Playlist>),
    AddTracksToPlaylist(Vec<Arc<Track>>),
    Loading(Option<Progress>),
    ResetSettings,
}

#[derive(Clone)]
pub enum Message {
    Close,
    CreatePlaylist,
    ImportPlaylist,
    OpenPlaylistImportDialog(Option<rfd::FileHandle>),
    RenamePlaylist {
        playlist: Arc<Playlist>,
        name: String,
    },
    DeletePlaylist(Arc<Playlist>),
    AddTracksToPlaylist(Vec<Arc<Track>>),
    PlaylistNameEdited(String),
}

pub fn update(state: &mut Chilen, message: Message) -> Task<event_loop::Message> {
    match message {
        Message::Close => state.dialog = Dialog::None,
        Message::ImportPlaylist => {
            return Task::perform(
                rfd::AsyncFileDialog::new()
                    .add_filter("M3U8 Playlist File", &["m3u", "m3u8"])
                    .set_directory(home_dir().unwrap_or(PathBuf::from(".")))
                    .pick_file(),
                |handle| event_loop::Message::Dialog(Message::OpenPlaylistImportDialog(handle)),
            );
        }
        Message::OpenPlaylistImportDialog(handle) => match handle {
            Some(handle) => {
                trace!("Showing import dialog for playlist {handle:?}");
                state.dialog = Dialog::ImportPlaylist(String::new(), handle);
            }
            None => info!("Didn't get the file handle, guessing the user cancelled the import"),
        },
        Message::CreatePlaylist => state.dialog = Dialog::CreatePlaylist(String::new()),
        Message::RenamePlaylist { playlist, name } => {
            state.dialog = Dialog::RenamePlaylist { playlist, name }
        }
        Message::DeletePlaylist(playlist) => {
            if playlist.tracks.is_empty() {
                if let Err(e) = chilen_backend::music_lib::delete_playlists(vec![&playlist.name]) {
                    error!("Couldn't delete playlist: {e}");
                    state.dialog = Dialog::Error(format!("Couldn't delete playlist: {e}"))
                }
            } else {
                state.dialog = Dialog::DeletePlaylist(playlist)
            }
        }
        Message::AddTracksToPlaylist(tracks) => state.dialog = Dialog::AddTracksToPlaylist(tracks),
        Message::PlaylistNameEdited(name) => {
            state.dialog = match &state.dialog {
                Dialog::CreatePlaylist(_) => Dialog::CreatePlaylist(name),
                Dialog::ImportPlaylist(_, handle) => Dialog::ImportPlaylist(name, handle.clone()),
                Dialog::RenamePlaylist { playlist, name: _ } => Dialog::RenamePlaylist {
                    playlist: playlist.clone(),
                    name,
                },
                _ => unreachable!(),
            }
        }
    }
    Task::none()
}

fn cancel_button(
    theme: &(impl ColorScheme + ?Sized),
) -> iced_m3::widget::dialog::Button<event_loop::Message> {
    iced_m3::widget::dialog::Button {
        on_press: Some(event_loop::Message::Dialog(Message::Close)),
        label: String::from("Cancel"),
        style: iced_m3::widget::button::Style::outlined(theme),
    }
}

fn action_button(
    theme: &(impl ColorScheme + ?Sized),
    on_press: Option<event_loop::Message>,
    label: String,
) -> iced_m3::widget::dialog::Button<event_loop::Message> {
    iced_m3::widget::dialog::Button {
        on_press,
        label,
        style: iced_m3::widget::button::Style::filled(theme, iced_m3::theme::Accent::Primary),
    }
}

pub fn view<'a>(state: &'a Chilen) -> Option<Element<'a, event_loop::Message>> {
    match &state.dialog {
        Dialog::None => None,
        Dialog::CreatePlaylist(name) => Some({
            let name_trimmed = name.trim();
            let name_ok = if let Some(lib) = &state.library {
                lib.find_playlist_by_name(name_trimmed).is_none()
            } else {
                false
            };

            let maybe_message = if name_ok {
                Some(event_loop::Message::Library(
                    library::Message::CreatePlaylist(name_trimmed.to_string()),
                ))
            } else {
                None
            };

            dialog(
                dialog::Style::new(&state.theme),
                iced_m3::widget::text_input::<_, Message>(
                    &state.library.as_ref().unwrap().get_default_playlist_name(),
                    name,
                    &state.theme,
                )
                .error(!name_ok)
                .with_label_text("Playlist name", state.theme.surface_container_high())
                .on_input(|i| event_loop::Message::Dialog(Message::PlaylistNameEdited(i)))
                .on_submit_maybe(maybe_message.clone()),
                vec![
                    cancel_button(&state.theme),
                    action_button(&state.theme, maybe_message, "Create".to_string()),
                ],
            )
            .title_font(font::bold())
            .title("Create playlist")
            .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
            .into()
        }),
        Dialog::ImportPlaylist(name, handle) => Some({
            let file_name = handle.file_name();
            let name_trimmed = name.trim();
            let default_name = file_name
                .strip_suffix(".m3u8")
                .unwrap_or(file_name.strip_suffix(".m3u").unwrap_or(&file_name))
                .trim();

            let name_ok = if let Some(lib) = &state.library {
                let name = if name_trimmed.is_empty() {
                    default_name
                } else {
                    name_trimmed
                };
                lib.find_playlist_by_name(name).is_none()
            } else {
                false
            };

            let maybe_message = name_ok.then_some(event_loop::Message::Library(
                library::Message::ImportPlaylist(
                    if name_trimmed.is_empty() {
                        None
                    } else {
                        Some(name_trimmed.to_string())
                    },
                    handle.clone(),
                ),
            ));

            dialog(
                dialog::Style::new(&state.theme),
                iced_m3::widget::text_input::<_, event_loop::Message>(
                    default_name,
                    name,
                    &state.theme,
                )
                .error(!name_ok)
                .with_label_text("Playlist name", state.theme.surface_container_high())
                .on_input(|i| event_loop::Message::Dialog(Message::PlaylistNameEdited(i)))
                .on_submit_maybe(maybe_message.clone()),
                vec![
                    cancel_button(&state.theme),
                    action_button(&state.theme, maybe_message, "Import".to_string()),
                ],
            )
            .title_font(font::bold())
            .title("Import playlist")
            .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
            .into()
        }),
        Dialog::Error(message) => Some({
            dialog(
                dialog::Style::new(&state.theme),
                container(text(message))
                    .style(|_| container::Style {
                        text_color: Some(state.theme.on_error_container()),
                        background: Some(iced::Background::Color(state.theme.error_container())),
                        border: iced::Border {
                            color: state.theme.on_error_container(),
                            width: 1.0,
                            radius: Radius::from(ROUNDING_SMALL),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill)
                    .padding(8.0),
                vec![action_button(
                    &state.theme,
                    Some(event_loop::Message::Dialog(Message::Close)),
                    "Dismiss".to_string(),
                )],
            )
            .title_font(font::bold())
            .title("Error")
            .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
            .into()
        }),
        Dialog::RenamePlaylist { playlist, name } => Some({
            let name_trimmed = name.trim();
            let name_ok = !name_trimmed.is_empty()
                && if let Some(lib) = &state.library {
                    lib.find_playlist_by_name(name_trimmed).is_none()
                } else {
                    true
                };

            let maybe_message = name_ok.then_some(event_loop::Message::Library(
                library::Message::RenamePlaylist {
                    playlist: playlist.clone(),
                    name: name.to_string(),
                },
            ));

            dialog(
                dialog::Style::new(&state.theme),
                iced_m3::widget::text_input::<_, Message>("Playlist name", name, &state.theme)
                    .error(!name_ok)
                    .with_label_text("Playlist name", state.theme.surface_container_high())
                    .on_input(|i| event_loop::Message::Dialog(Message::PlaylistNameEdited(i)))
                    .on_submit_maybe(maybe_message.clone()),
                vec![
                    cancel_button(&state.theme),
                    action_button(&state.theme, maybe_message, "Rename".to_string()),
                ],
            )
            .title_font(font::bold())
            .title("Rename playlist")
            .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
            .into()
        }),
        Dialog::DeletePlaylist(playlist) => Some({
            dialog(
                dialog::Style::new(&state.theme),
                text(format!(
                    "Delete playlist \"{}\" with {} tracks?\n\nThis cannot be undone.",
                    playlist.name,
                    playlist.tracks.len()
                )),
                vec![
                    cancel_button(&state.theme),
                    action_button(
                        &state.theme,
                        Some(event_loop::Message::Library(
                            library::Message::DeletePlaylist(playlist.clone()),
                        )),
                        "Delete".to_string(),
                    ),
                ],
            )
            .title_font(font::bold())
            .title("Delete playlist")
            .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
            .into()
        }),
        Dialog::AddTracksToPlaylist(tracks) => {
            Some(add_tracks_to_playlist::view(state, tracks.clone()))
        }
        Dialog::Loading(progress) => Some(loading::view(&state.theme, progress.clone())),
        Dialog::ResetSettings => Some(
            dialog(
                dialog::Style::new(&state.theme),
                text("Are you absolutely sure you want to reset all of your settings?"),
                vec![
                    cancel_button(&state.theme),
                    action_button(
                        &state.theme,
                        Some(event_loop::Message::Settings(settings::Message::Reset)),
                        "Reset".into(),
                    ),
                ],
            )
            .title_font(font::bold())
            .title("Reset settings")
            .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
            .into(),
        ),
    }
}
