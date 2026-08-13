use std::sync::Arc;

use chilen_backend::music_lib::state::{MusicLibrary, Track};
use iced::{
    Border, Element, Length, Padding,
    widget::{button, container, row, space},
};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{center, column, image, sensor, stack, text};

use crate::gui::{
    BUTTON_HEIGHT, BUTTON_PADDING, BUTTON_SPACING, Chilen, SPACING_SMALL, THUMBNAIL_SIZE,
    font::{SIZE_REGULAR, SIZE_SMALL},
    icons,
    main_view::{self, BUTTON_ROUNDING, button_style},
};

pub fn track_button<'a>(
    state: &'a Chilen,
    index: usize,
    track: &'a Arc<Track>,
) -> Element<'a, main_view::Message> {
    let content: Element<'a, main_view::Message> = if let Some(visible) = &state.main_view.visible
        && index < visible.len()
        && visible[index]
    {
        let thumbnail_border_radius = BUTTON_ROUNDING - BUTTON_PADDING;
        button(
            row![
                stack![
                    container(center(
                        text(*icons::MUSIC_NOTE)
                            .font(icons::filled())
                            .color(state.theme.on_surface_variant())
                            .size(icons::SIZE_LARGE)
                    ))
                    .style(move |_| {
                        container::Style::default()
                            .background(state.theme.surface_container_high())
                            .border(Border::default().rounded(thumbnail_border_radius))
                    })
                    .width(Length::Fixed(THUMBNAIL_SIZE))
                    .height(Length::Fixed(THUMBNAIL_SIZE)),
                    track.cover.thumbnail.as_ref().map(|thumbnail| {
                        image(thumbnail)
                            .content_fit(iced::ContentFit::Cover)
                            .width(Length::Fixed(THUMBNAIL_SIZE))
                            .height(Length::Fixed(THUMBNAIL_SIZE))
                            .border_radius(thumbnail_border_radius)
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
                    text(
                        track
                            .artists
                            .as_ref()
                            .map(|a| a.join(&state.settings.value_separator))
                            .unwrap_or("Unknown".to_string())
                    )
                    .size(SIZE_SMALL)
                    .color(state.theme.on_surface_variant())
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
        .padding(Padding::new(BUTTON_PADDING))
        .style(|_, status| button_style(status, &state.theme))
        .on_press(main_view::Message::Noop)
        .into()
    } else {
        space().height(BUTTON_HEIGHT).width(Length::Fill).into()
    };
    sensor(content)
        .on_show(move |_| main_view::Message::ButtonPoppedIn(index))
        .on_hide(main_view::Message::ButtonPoppedOut(index))
        .into()
}

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, main_view::Message> {
    let content = column(
        lib.tracks
            .iter()
            .enumerate()
            .map(|(i, t)| track_button(state, i, t)),
    )
    .spacing(BUTTON_SPACING);

    iced_widget::scrollable(content)
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
