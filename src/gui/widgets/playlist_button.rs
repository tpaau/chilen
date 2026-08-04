use std::sync::Arc;

use chilen_backend::music_lib::state::Playlist;
use iced::{
    Border, Color, Length, Padding, color,
    widget::{Button, button, column, container, row, space, text},
};
use iced_m3::{
    HOVER_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY,
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};

use crate::gui::{
    Chilen, DIM_TEXT_ALPHA, Message, ROUNDING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
    font::{SIZE_REGULAR, SIZE_SMALL},
    icons,
    main_view::THUMBNAIL_SIZE,
};

pub fn playlist_button<'a>(state: &'a Chilen, playlist: &'a Arc<Playlist>) -> Button<'a, Message> {
    button(
        row![
            container(space())
                .style(|_| {
                    container::Style::default()
                        .background(color!(0xff0000))
                        .border(Border::default().rounded(ROUNDING_REGULAR - SPACING_SMALLER))
                })
                .width(Length::Fixed(THUMBNAIL_SIZE))
                .height(Length::Fixed(THUMBNAIL_SIZE)),
            container(column![
                text(playlist.name.clone())
                    .size(SIZE_REGULAR)
                    .color(state.theme.on_surface())
                    .wrapping(text::Wrapping::None),
                text({
                    if playlist.tracks.is_empty() {
                        "No tracks".to_string()
                    } else {
                        format!("{} tracks", playlist.tracks.len())
                    }
                })
                .size(SIZE_SMALL)
                .color(state.theme.on_surface().scale_alpha(DIM_TEXT_ALPHA)),
            ])
            .width(Length::Fill)
            .clip(true)
            .center_y(Length::Fill),
            container(
                // TODO: Should be more like a button
                drop_down_menu(
                    |_| {
                        text(*icons::MORE_HORIZ)
                            .font(icons::filled())
                            .size(icons::SIZE_REGULAR)
                            .into()
                    },
                    Some(
                        iced_m3::widget::menu(
                            vec![
                                vertical_menu::Group {
                                    label: None,
                                    entries: vec![
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::PLAY_ARROW),
                                            label: "Play",
                                            supporting_text: None,
                                            error: false,
                                            action: vertical_menu::Action::Message(None),
                                        },
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::SHUFFLE),
                                            label: "Shuffle",
                                            supporting_text: None,
                                            error: false,
                                            action: vertical_menu::Action::Message(None),
                                        },
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::ADD_TO_QUEUE),
                                            label: "Add to queue",
                                            supporting_text: None,
                                            error: false,
                                            action: vertical_menu::Action::Message(None),
                                        },
                                    ],
                                },
                                vertical_menu::Group {
                                    label: None,
                                    entries: vec![
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::UPLOAD),
                                            label: "Export",
                                            supporting_text: None,
                                            error: false,
                                            action: vertical_menu::Action::Message(Some(
                                                Message::OpenPlaylistExportPicker(
                                                    playlist.name.clone(),
                                                ),
                                            )),
                                        },
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::IMAGE),
                                            label: "Change image",
                                            supporting_text: None,
                                            error: false,
                                            action: vertical_menu::Action::Message(None),
                                        },
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::EDIT),
                                            label: "Rename",
                                            supporting_text: None,
                                            error: false,
                                            action: vertical_menu::Action::Message(Some(
                                                Message::OpenPlaylistRenameDialog {
                                                    playlist: playlist.name.clone(),
                                                    name: playlist.name.clone(),
                                                },
                                            )),
                                        },
                                        vertical_menu::Entry::Separator,
                                        vertical_menu::Entry::Button {
                                            icon: Some(&icons::DELETE),
                                            label: "Delete",
                                            supporting_text: None,
                                            error: true,
                                            action: vertical_menu::Action::Message(Some(
                                                Message::ConfirmPlaylistDeletion(playlist.clone()),
                                            )),
                                        },
                                    ],
                                },
                            ],
                            &state.theme,
                        )
                        .icon_font(icons::filled()),
                    ),
                    iced_m3::widget::drop_down_menu::Placement::BottomRight,
                ),
            )
            .center_y(Length::Fill),
        ]
        .spacing(SPACING_SMALL),
    )
    .padding(Padding::new(SPACING_SMALLER))
    .style(|_, status| {
        let content_color = state.theme.on_surface();
        iced_widget::button::Style {
            background: Some(iced::Background::Color(match status {
                iced_widget::button::Status::Active => Color::TRANSPARENT,
                iced_widget::button::Status::Hovered => {
                    content_color.scale_alpha(HOVER_STATE_LAYER_OPACITY)
                }
                iced_widget::button::Status::Pressed => {
                    content_color.scale_alpha(PRESSED_STATE_LAYER_OPACITY)
                }
                iced_widget::button::Status::Disabled => {
                    unreachable!("There should be no inactive buttons in the playlist view")
                }
            })),
            text_color: content_color,
            border: Border::default().rounded(ROUNDING_REGULAR),
            ..Default::default()
        }
    })
    .on_press(Message::OpenPlaylist(playlist.clone()))
}
