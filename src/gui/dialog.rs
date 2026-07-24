use iced::{Element, Length};
use iced_m3::{theme::ColorScheme, widget::dialog};
use iced_widget::{button, space};

use crate::gui::{Chilen, Dialog, Message, font};

pub fn view(state: &Chilen) -> Option<Element<'_, Message>> {
    match &state.dialog {
        Dialog::None => None,
        Dialog::CreatePlaylist(name) => Some({
            let playlist_exists = if let Some(lib) = &state.library {
                lib.find_playlist(name).is_some()
            } else {
                false
            };

            dialog(
                true,
                space().width(Length::Fill).height(Length::Fill),
                iced_m3::widget::text_input::<_, Message>(
                    &state.library.as_ref().unwrap().get_default_playlist_name(),
                    name,
                    &state.theme,
                )
                .error(playlist_exists)
                .with_label_text("Playlist name", state.theme.surface_container_high())
                .on_input(Message::PlaylistNameEdited)
                .on_submit_maybe(if playlist_exists {
                    None
                } else {
                    Some(Message::CreatePlaylist(name.clone()))
                }),
                state.theme.current(),
            )
            .title("New playlist")
            .font(font::font_bold())
            .push_button(space().width(Length::Fill))
            .push_button(
                button("Cancel")
                    .style(|_, status| {
                        iced_m3::style::button(
                            status,
                            state.theme.current(),
                            iced_m3::style::Button::Outlined,
                        )
                    })
                    .padding(12)
                    .on_press(Message::CloseDialog),
            )
            .push_button(
                button("Create")
                    .style(|_, status| {
                        iced_m3::style::button(
                            status,
                            state.theme.current(),
                            iced_m3::style::Button::Primary,
                        )
                    })
                    .padding(12)
                    .on_press_maybe(if playlist_exists {
                        None
                    } else {
                        Some(Message::CreatePlaylist(name.clone()))
                    }),
            )
            .width(350)
            .into()
        }),
        Dialog::ImportPlaylist(name, handle) => Some({
            let file_name = handle.file_name();
            let default_name = file_name.strip_suffix(".m3u8").unwrap_or(&file_name);
            let playlist_exists = if let Some(lib) = &state.library {
                let name = if name.is_empty() { default_name } else { name };
                lib.find_playlist(name).is_some()
            } else {
                false
            };

            dialog(
                true,
                space().width(Length::Fill).height(Length::Fill),
                iced_m3::widget::text_input::<_, Message>(default_name, name, &state.theme)
                    .error(playlist_exists)
                    .with_label_text("Playlist name", state.theme.surface_container_high())
                    .on_input(Message::PlaylistNameEdited)
                    .on_submit_maybe(if playlist_exists {
                        None
                    } else {
                        Some(Message::ImportPlaylist(
                            if name.is_empty() {
                                None
                            } else {
                                Some(name.clone())
                            },
                            handle.clone(),
                        ))
                    }),
                state.theme.current(),
            )
            .title("Import playlist")
            .font(font::font_bold())
            .push_button(space().width(Length::Fill))
            .push_button(
                button("Cancel")
                    .style(|_, status| {
                        iced_m3::style::button(
                            status,
                            state.theme.current(),
                            iced_m3::style::Button::Outlined,
                        )
                    })
                    .padding(12)
                    .on_press(Message::CloseDialog),
            )
            .push_button(
                button("Import")
                    .style(|_, status| {
                        iced_m3::style::button(
                            status,
                            state.theme.current(),
                            iced_m3::style::Button::Primary,
                        )
                    })
                    .padding(12)
                    .on_press_maybe(if playlist_exists {
                        None
                    } else {
                        Some(Message::ImportPlaylist(
                            if name.is_empty() {
                                None
                            } else {
                                Some(name.clone())
                            },
                            handle.clone(),
                        ))
                    }),
            )
            .width(350)
            .into()
        }),
    }
}
