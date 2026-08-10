use std::sync::Arc;

use chilen_backend::music_lib::state::Playlist;
use iced::{
    Border, Color, Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use iced_m3::{
    HOVER_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY,
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{center, sensor, stack};

use crate::gui::{
    BUTTON_PADDING, BUTTON_ROUNDING, BUTTON_SPACING, Chilen, THUMBNAIL_SIZE,
    font::{SIZE_REGULAR, SIZE_SMALL},
    icons, playlist_view,
};

pub fn playlist_button<'a>(
    state: &'a Chilen,
    playlist: &'a Arc<Playlist>,
    index: usize,
) -> Element<'a, playlist_view::Message> {
    sensor(
        button(
            row![
                stack![
                    container(space())
                        .style(|_| {
                            container::Style::default()
                                .background(state.theme.surface_container_highest())
                                .border(Border::default().rounded(BUTTON_ROUNDING - BUTTON_PADDING))
                        })
                        .width(Length::Fixed(THUMBNAIL_SIZE))
                        .height(Length::Fixed(THUMBNAIL_SIZE)),
                    center(
                        text(*icons::PLAYLIST_PLAY)
                            .font(icons::filled())
                            .color(state.theme.on_surface())
                            .size(icons::SIZE_LARGE)
                    ),
                    // TODO: Thumbnail
                ],
                container(column![
                    text(playlist.name.clone())
                        .size(SIZE_REGULAR)
                        .color(state.theme.on_surface())
                        .wrapping(text::Wrapping::None),
                    text({
                        if playlist.tracks.is_empty() {
                            "Empty".to_string()
                        } else {
                            format!("{} tracks", playlist.tracks.len())
                        }
                    })
                    .size(SIZE_SMALL)
                    .color(state.theme.on_surface_variant()),
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
                                                    playlist_view::Message::ExportPlaylist(
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
                                                    playlist_view::Message::OpenPlaylistRenameDialog {
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
                                                    playlist_view::Message::ConfirmPlaylistDeletion(
                                                        playlist.clone()
                                                    ),
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
            .spacing(BUTTON_SPACING),
        )
        .padding(Padding::new(BUTTON_PADDING))
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
                border: Border::default().rounded(BUTTON_ROUNDING),
                ..Default::default()
            }
        })
        .on_press(playlist_view::Message::OpenPlaylist(playlist.clone())),
    )
    .on_show(move |_| playlist_view::Message::ButtonPoppedIn(index))
    .on_hide(playlist_view::Message::ButtonPoppedOut(index))
    .into()
}
