use std::sync::Arc;

use chilen_backend::music_lib::state::Playlist;
use iced::{
    Background, Border, Length, Padding, Shadow, color,
    widget::{Button, Space, button, column, container, row, space, text},
};
use iced_m3::{
    theme::ColorScheme,
    widget::{
        drop_down_menu::{DropDownMenu, Placement},
        vertical_menu,
    },
};

use crate::gui::{
    Chilen, DIM_TEXT_ALPHA, Message, ROUNDING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
    font::{self, SIZE_REGULAR, SIZE_SMALL},
    icons,
};

const THUMBNAIL_SIZE: f32 = 64.0;

pub fn playlist_button<'a>(state: &'a Chilen, playlist: &'a Arc<Playlist>) -> Button<'a, Message> {
    button(
        row(vec![
            container(Space::new())
                .style(|_| {
                    container::Style::default()
                        .background(color!(0xff0000))
                        .border(Border::default().rounded(ROUNDING_REGULAR - SPACING_SMALLER))
                })
                .width(Length::Fixed(THUMBNAIL_SIZE))
                .height(Length::Fixed(THUMBNAIL_SIZE))
                .into(),
            container(column(vec![
                text(playlist.name.clone())
                    .size(SIZE_REGULAR)
                    .color(state.theme.on_surface())
                    .into(),
                text({
                    if playlist.tracks.is_empty() {
                        "No tracks".to_string()
                    } else {
                        format!("{} tracks", playlist.tracks.len())
                    }
                })
                .size(SIZE_SMALL)
                .color(state.theme.on_surface().scale_alpha(DIM_TEXT_ALPHA))
                .into(),
            ]))
            .center_y(Length::Fill)
            .into(),
            space().width(Length::Fill).into(),
            container(
                // TODO: Should be more like a button
                DropDownMenu::new(
                    |_| {
                        text(*icons::MORE_HORIZ)
                            .font(icons::font())
                            .size(icons::SIZE_REGULAR)
                            .into()
                    },
                    iced_m3::widget::menu(
                        vec![
                            vertical_menu::Group {
                                label: Some("Menu!"),
                                entries: vec![
                                    vertical_menu::Entry::Button {
                                        icon: Some(&icons::CLOSE),
                                        label: "Label",
                                        supporting_text: Some("Supporting text"),
                                        action: vertical_menu::Action::Message(Some(
                                            Message::CloseDialog,
                                        )),
                                    },
                                    vertical_menu::Entry::Button {
                                        icon: Some(&icons::ADD),
                                        label: "Label",
                                        supporting_text: None,
                                        action: vertical_menu::Action::Menu(vec![]),
                                    },
                                    vertical_menu::Entry::Button {
                                        icon: Some(&icons::ADD),
                                        label: "Label",
                                        supporting_text: None,
                                        action: vertical_menu::Action::Menu(vec![]),
                                    },
                                    vertical_menu::Entry::Separator,
                                    vertical_menu::Entry::Button {
                                        icon: None,
                                        label: "Label",
                                        supporting_text: None,
                                        action: vertical_menu::Action::Message(None),
                                    },
                                ],
                            },
                            vertical_menu::Group {
                                label: None,
                                entries: vec![vertical_menu::Entry::Button {
                                    icon: None,
                                    label: "Label",
                                    supporting_text: None,
                                    action: vertical_menu::Action::Message(Some(
                                        Message::CloseDialog,
                                    )),
                                }],
                            },
                        ],
                        &state.theme,
                    )
                    .font(font::font())
                    .icon_font(icons::font())
                    .vibrant(false),
                    Placement::BottomLeft,
                ),
            )
            .center_y(Length::Fill)
            .into(),
        ])
        .spacing(SPACING_SMALL),
    )
    .padding(Padding::new(SPACING_SMALLER as f32))
    .style(|_, status| {
        let style = button::Style {
            background: Some(Background::Color(state.theme.surface_container())),
            text_color: state.theme.on_surface(),
            border: Border::default().rounded(ROUNDING_REGULAR),
            shadow: Shadow::default(),
            snap: true,
        };

        match status {
            button::Status::Hovered => {
                style.with_background(Background::Color(state.theme.surface_container_high()))
            }
            _ => style,
        }
    })
    .on_press(Message::OpenPlaylist(playlist.clone()))
}
