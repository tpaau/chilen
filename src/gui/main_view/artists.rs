use std::sync::Arc;

use chilen_backend::music_lib::state::Artist;
use iced::{
    Alignment, Border, Element, Length, Padding, color,
    widget::{Button, button, container, row, space},
};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{column, text};

use crate::gui::{
    Chilen, DIM_TEXT_ALPHA, Message, SPACING_SMALL, SPACING_SMALLER,
    font::{SIZE_REGULAR, SIZE_SMALL},
    icons,
    main_view::{THUMBNAIL_SIZE, button_style},
};

pub fn artist_button<'a>(state: &'a Chilen, artist: &'a Arc<Artist>) -> Button<'a, Message> {
    button(
        row(vec![
            container(space())
                .style(|_| {
                    container::Style::default()
                        .background(color!(0xff0000))
                        .border(Border::default().rounded(f32::MAX))
                })
                .width(Length::Fixed(THUMBNAIL_SIZE))
                .height(Length::Fixed(THUMBNAIL_SIZE))
                .into(),
            container(column![
                text(&artist.name)
                    .size(SIZE_REGULAR)
                    .color(state.theme.on_surface())
                    .wrapping(text::Wrapping::None),
                row![
                    text(match artist.albums.len() {
                        0 => "No albums".to_string(),
                        1 => "1 album".to_string(),
                        _ => format!("{} albums", artist.albums.len()),
                    })
                    .size(SIZE_SMALL)
                    .color(state.theme.on_surface().scale_alpha(DIM_TEXT_ALPHA))
                    .wrapping(text::Wrapping::None),
                    container(space())
                        .width(Length::Fixed(SIZE_SMALL as f32 / 2.0))
                        .height(Length::Fixed(SIZE_SMALL as f32 / 2.0))
                        .style(|_| container::Style::default()
                            .background(state.theme.on_surface().scale_alpha(DIM_TEXT_ALPHA))
                            .border(Border::default().rounded(f32::MAX))),
                    text(match artist.tracks.len() {
                        1 => "1 track".to_string(),
                        _ => format!("{} tracks", artist.tracks.len()),
                    })
                    .size(SIZE_SMALL)
                    .color(state.theme.on_surface().scale_alpha(DIM_TEXT_ALPHA))
                ]
                .spacing(SPACING_SMALLER)
                .align_y(Alignment::Center),
            ])
            .width(Length::Fill)
            .clip(true)
            .center_y(Length::Fill)
            .into(),
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
                                    entries: vec![vertical_menu::Entry::Button {
                                        icon: Some(&icons::UPLOAD),
                                        label: "Details",
                                        supporting_text: None,
                                        error: false,
                                        action: vertical_menu::Action::Message(None),
                                    }],
                                },
                            ],
                            &state.theme,
                        )
                        .icon_font(icons::filled()),
                    ),
                    iced_m3::widget::drop_down_menu::Placement::BottomRight,
                ),
            )
            .center_y(Length::Fill)
            .into(),
        ])
        .spacing(SPACING_SMALL),
    )
    .padding(Padding::new(SPACING_SMALLER as f32))
    .style(|_, status| button_style(status, &state.theme))
    .on_press(Message::CloseDialog)
}

pub fn view(state: &Chilen) -> Element<'_, Message> {
    let lib = if let Some(lib) = &state.library {
        lib
    } else {
        return text("Loading...").into();
    };

    iced::widget::scrollable(
        column(
            lib.artists
                .iter()
                .map(|a| artist_button(state, a).into())
                .collect::<Vec<_>>(),
        )
        .spacing(SPACING_SMALLER),
    )
    .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}
