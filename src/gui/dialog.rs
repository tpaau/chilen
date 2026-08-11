use iced::{Element, Length, border::Radius};
use iced_m3::{
    theme::ColorScheme,
    widget::{button, button::Style, dialog},
};
use iced_widget::{container, space, text};

use crate::gui::{
    Chilen, Dialog,
    Message::{self},
    ROUNDING_SMALL, font,
};

pub fn view(state: &Chilen) -> Option<Element<'_, Message>> {
    let cancel_button = match &state.dialog {
        Dialog::None => None,
        _ => Some(
            button(&state.theme)
                .on_press(Message::CloseDialog)
                .label("Cancel")
                .style(Style::Outlined),
        ),
    };

    match &state.dialog {
        Dialog::None => None,
        Dialog::CreatePlaylist(name) => Some({
            let name_trimmed = name.trim();
            let name_ok = if let Some(lib) = &state.library {
                lib.find_playlist(name_trimmed).is_none()
            } else {
                false
            };

            let maybe_message = if name_ok {
                Some(Message::CreatePlaylist(name_trimmed.to_string()))
            } else {
                None
            };

            dialog(
                true,
                space().width(Length::Fill).height(Length::Fill),
                iced_m3::widget::text_input::<_, Message>(
                    &state.library.as_ref().unwrap().get_default_playlist_name(),
                    name,
                    &state.theme,
                )
                .error(!name_ok)
                .with_label_text("Playlist name", state.theme.surface_container_high())
                .on_input(Message::PlaylistNameEdited)
                .on_submit_maybe(maybe_message.clone()),
                state.theme.current(),
            )
            .title("New playlist")
            .font(font::font_bold())
            .push_button(space().width(Length::Fill))
            .push_button(cancel_button)
            .push_button(
                button(&state.theme)
                    .label("Create")
                    .style(Style::Filled(iced_m3::theme::Accent::Primary))
                    .on_press_maybe(maybe_message),
            )
            .width(350)
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
                lib.find_playlist(name).is_none()
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
                true,
                space().width(Length::Fill).height(Length::Fill),
                iced_m3::widget::text_input::<_, Message>(default_name, name, &state.theme)
                    .error(!name_ok)
                    .with_label_text("Playlist name", state.theme.surface_container_high())
                    .on_input(Message::PlaylistNameEdited)
                    .on_submit_maybe(maybe_message.clone()),
                state.theme.current(),
            )
            .title("Import playlist")
            .font(font::font_bold())
            .push_button(space().width(Length::Fill))
            .push_button(cancel_button)
            .push_button(
                button(&state.theme)
                    .label("Import")
                    .style(Style::Filled(iced_m3::theme::Accent::Primary))
                    .on_press_maybe(maybe_message),
            )
            .width(350)
            .into()
        }),
        Dialog::Error(message) => Some({
            dialog(
                true,
                space().width(Length::Fill).height(Length::Fill),
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
                state.theme.current(),
            )
            .title("Error")
            .font(font::font_bold())
            .push_button(space().width(Length::Fill))
            .push_button(
                button(&state.theme)
                    .label("Dismiss")
                    .style(Style::Filled(iced_m3::theme::Accent::Primary))
                    .on_press(Message::CloseDialog),
            )
            .width(350)
            .into()
        }),
        Dialog::RenamePlaylist { playlist, name } => Some({
            let name_trimmed = name.trim();
            let name_ok = !name_trimmed.is_empty()
                && if let Some(lib) = &state.library {
                    lib.find_playlist(name_trimmed).is_none()
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
                true,
                space().width(Length::Fill).height(Length::Fill),
                iced_m3::widget::text_input::<_, Message>("Playlist name", name, &state.theme)
                    .error(!name_ok)
                    .with_label_text("Playlist name", state.theme.surface_container_high())
                    .on_input(Message::PlaylistNameEdited)
                    .on_submit_maybe(maybe_message.clone()),
                state.theme.current(),
            )
            .title("Rename playlist")
            .font(font::font_bold())
            .push_button(space().width(Length::Fill))
            .push_button(cancel_button)
            .push_button(
                button(&state.theme)
                    .label("Rename")
                    .style(Style::Filled(iced_m3::theme::Accent::Primary))
                    .on_press_maybe(maybe_message),
            )
            .width(350)
            .into()
        }),
        Dialog::DeletePlaylist(playlist) => Some({
            dialog(
                true,
                space().width(Length::Fill).height(Length::Fill),
                text(format!(
                    "Delete playlist \"{}\" with {} tracks?\n\nThis cannot be undone.",
                    playlist.name,
                    playlist.tracks.len()
                )),
                state.theme.current(),
            )
            .title("Delete playlist")
            .font(font::font_bold())
            .push_button(space().width(Length::Fill))
            .push_button(cancel_button)
            .push_button(
                button(&state.theme)
                    .label("Delete")
                    .style(Style::Filled(iced_m3::theme::Accent::Primary))
                    .on_press(Message::DeletePlaylist(playlist.clone())),
            )
            .width(350)
            .into()
        }),
    }
}
