mod add_track_to_playlist;

use std::sync::Arc;

use chilen_backend::music_lib::{Playlist, Track};
use iced::{Element, Length, border::Radius};
use iced_m3::{theme::ColorScheme, widget::dialog};
use iced_widget::{container, text};

use crate::gui::{
    Chilen,
    Message::{self},
    ROUNDING_SMALL, font, settings,
};

#[derive(Default)]
pub enum Dialog {
    #[default]
    None,
    CreatePlaylist(String),
    ImportPlaylist(String, rfd::FileHandle),
    Error(String),
    RenamePlaylist {
        playlist: String,
        name: String,
    },
    DeletePlaylist(Arc<Playlist>),
    Settings,
    AddTrackToPlaylist(Arc<Track>),
}

/// Appends a single track to the queue.
pub fn add_track_to_playlist(state: &mut Chilen, track: Arc<Track>) {
    state.dialog = Dialog::AddTrackToPlaylist(track)
}

fn cancel_button() -> iced_m3::widget::dialog::Button<Message> {
    iced_m3::widget::dialog::Button {
        on_press: Some(Message::CloseDialog),
        label: String::from("Cancel"),
        style: iced_m3::widget::button::Style::Outlined,
    }
}

fn action_button(
    on_press: Option<Message>,
    label: String,
) -> iced_m3::widget::dialog::Button<Message> {
    iced_m3::widget::dialog::Button {
        on_press,
        label,
        style: iced_m3::widget::button::Style::Filled(iced_m3::theme::Accent::Primary),
    }
}

pub fn view<'a>(state: &'a Chilen) -> Option<Element<'a, Message>> {
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
                Some(Message::CreatePlaylist(name_trimmed.to_string()))
            } else {
                None
            };

            dialog(
                &state.theme,
                iced_m3::widget::text_input::<_, Message>(
                    &state.library.as_ref().unwrap().get_default_playlist_name(),
                    name,
                    &state.theme,
                )
                .error(!name_ok)
                .with_label_text("Playlist name", state.theme.surface_container_high())
                .on_input(Message::PlaylistNameEdited)
                .on_submit_maybe(maybe_message.clone()),
                vec![
                    cancel_button(),
                    action_button(maybe_message, "Create".to_string()),
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

            let maybe_message = if name_ok {
                Some(Message::ImportPlaylist(
                    if name_trimmed.is_empty() {
                        None
                    } else {
                        Some(name_trimmed.to_string())
                    },
                    handle.clone(),
                ))
            } else {
                None
            };

            dialog(
                &state.theme,
                iced_m3::widget::text_input::<_, Message>(default_name, name, &state.theme)
                    .error(!name_ok)
                    .with_label_text("Playlist name", state.theme.surface_container_high())
                    .on_input(Message::PlaylistNameEdited)
                    .on_submit_maybe(maybe_message.clone()),
                vec![
                    cancel_button(),
                    action_button(maybe_message, "Import".to_string()),
                ],
            )
            .title_font(font::bold())
            .title("Import playlist")
            .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
            .into()
        }),
        Dialog::Error(message) => Some({
            dialog(
                &state.theme,
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
                    Some(Message::CloseDialog),
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

            let maybe_message = if name_ok {
                Some(Message::RenamePlaylist {
                    playlist: playlist.clone(),
                    name: name.to_string(),
                })
            } else {
                None
            };

            dialog(
                &state.theme,
                iced_m3::widget::text_input::<_, Message>("Playlist name", name, &state.theme)
                    .error(!name_ok)
                    .with_label_text("Playlist name", state.theme.surface_container_high())
                    .on_input(Message::PlaylistNameEdited)
                    .on_submit_maybe(maybe_message.clone()),
                vec![
                    cancel_button(),
                    action_button(maybe_message, "Rename".to_string()),
                ],
            )
            .title_font(font::bold())
            .title("Rename playlist")
            .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
            .into()
        }),
        Dialog::DeletePlaylist(playlist) => Some({
            dialog(
                &state.theme,
                text(format!(
                    "Delete playlist \"{}\" with {} tracks?\n\nThis cannot be undone.",
                    playlist.name,
                    playlist.tracks.len()
                )),
                vec![
                    cancel_button(),
                    action_button(
                        Some(Message::DeletePlaylist(playlist.clone())),
                        "Delete".to_string(),
                    ),
                ],
            )
            .title_font(font::bold())
            .title("Delete playlist")
            .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
            .into()
        }),
        Dialog::Settings => Some(settings::view(state).map(Message::Settings)),
        Dialog::AddTrackToPlaylist(track) => {
            Some(add_track_to_playlist::view(state, track.clone()))
        }
    }
}
