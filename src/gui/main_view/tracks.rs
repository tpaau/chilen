use std::sync::Arc;

use chilen_backend::music_lib::state::Track;
use iced::{
    Border, Element, Length, Padding,
    widget::{Button, button, container, row, space},
};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{center, column, image, stack, text};

use crate::gui::{
    Chilen, DIM_TEXT_ALPHA, Message, ROUNDING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
    font::{SIZE_REGULAR, SIZE_SMALL},
    icons,
    main_view::{THUMBNAIL_SIZE, button_style},
};

pub fn track_button<'a>(state: &'a Chilen, track: &'a Arc<Track>) -> Button<'a, Message> {
    button(
        row![
            stack![
                container(space())
                    .style(|_| {
                        container::Style::default()
                            .background(state.theme.surface_container_high())
                            .border(Border::default().rounded(ROUNDING_REGULAR - SPACING_SMALLER))
                    })
                    .width(Length::Fixed(THUMBNAIL_SIZE))
                    .height(Length::Fixed(THUMBNAIL_SIZE)),
                center(
                    text(*icons::MUSIC_NOTE)
                        .font(icons::filled())
                        .color(state.theme.on_surface_variant())
                        .size(icons::SIZE_LARGE)
                ),
                track.cover_path.as_ref().map(|cover| {
                    image(cover)
                        .width(Length::Fixed(THUMBNAIL_SIZE))
                        .height(Length::Fixed(THUMBNAIL_SIZE))
                })
            ],
            container(column![
                text(if let Some(title) = &track.title {
                    title
                } else {
                    "Unknown"
                })
                .size(SIZE_REGULAR)
                .color(state.theme.on_surface())
                .wrapping(text::Wrapping::None),
                text(if let Some(artist) = &track.artist {
                    artist
                } else {
                    "Unknown"
                })
                .size(SIZE_SMALL)
                .color(state.theme.on_surface().scale_alpha(DIM_TEXT_ALPHA))
                .wrapping(text::Wrapping::None),
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
            .center_y(Length::Fill),
        ]
        .spacing(SPACING_SMALL),
    )
    .height(Length::Shrink)
    .padding(Padding::new(SPACING_SMALLER as f32))
    .style(|_, status| button_style(status, &state.theme))
    .on_press(Message::CloseDialog)
}

pub fn view(state: &Chilen) -> Element<'_, Message> {
    if let Some(lib) = &state.library {
        iced_widget::scrollable(column(
            lib.tracks.iter().map(|t| track_button(state, t).into()),
        ))
        .into()
    } else {
        text("Loading...").into()
    }
}
