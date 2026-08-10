use std::sync::Arc;

use chilen_backend::music_lib::state::{Artist, MusicLibrary};
use iced::{
    Alignment, Border, Element, Length, Padding,
    widget::{button, container, row, space},
};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{center, column, image, sensor, stack, text};

use crate::gui::{
    self, BUTTON_HEIGHT, BUTTON_PADDING, BUTTON_SPACING, Chilen, SPACING_SMALL, SPACING_SMALLER,
    THUMBNAIL_SIZE,
    font::{SIZE_REGULAR, SIZE_SMALL},
    icons,
    main_view::{self, button_style},
};

pub fn artist_button<'a>(
    state: &'a Chilen,
    index: usize,
    artist: &'a Arc<Artist>,
) -> Element<'a, gui::Message> {
    let content: Element<'a, gui::Message> = if let Some(visible) = &state.main_view.visible
        && index < visible.len()
        && visible[index]
    {
        button(
            row![
                stack![
                    container(center(
                        text(*icons::ARTIST)
                            .font(icons::filled())
                            .color(state.theme.on_surface_variant())
                            .size(icons::SIZE_LARGE)
                    ))
                    .style(|_| {
                        container::Style::default()
                            .background(state.theme.surface_container_high())
                            .border(Border::default().rounded(f32::MAX))
                    })
                    .width(Length::Fixed(THUMBNAIL_SIZE))
                    .height(Length::Fixed(THUMBNAIL_SIZE)),
                    artist.cover.thumbnail.as_ref().map(|t| {
                        image(t)
                            .content_fit(iced::ContentFit::Cover)
                            .width(Length::Fixed(THUMBNAIL_SIZE))
                            .height(Length::Fixed(THUMBNAIL_SIZE))
                            .border_radius(f32::MAX)
                    }),
                ],
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
                        .color(state.theme.on_surface_variant())
                        .wrapping(text::Wrapping::None),
                        container(
                            space()
                                .width(Length::Fixed(SIZE_SMALL / 3.0))
                                .height(Length::Fixed(SIZE_SMALL / 3.0))
                        )
                        .style(|_| container::Style::default()
                            .border(Border::default().rounded(f32::MAX))
                            .background(state.theme.on_surface_variant())),
                        text(match artist.tracks.len() {
                            1 => "1 track".to_string(),
                            _ => format!("{} tracks", artist.tracks.len()),
                        })
                        .size(SIZE_SMALL)
                        .color(state.theme.on_surface_variant())
                        .wrapping(text::Wrapping::None),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(SPACING_SMALLER),
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
        .on_press(gui::Message::CloseDialog)
        .into()
    } else {
        space().height(BUTTON_HEIGHT).width(Length::Fill).into()
    };
    sensor(content)
        .on_show(move |_| gui::Message::MainView(main_view::Message::ButtonPoppedIn(index)))
        .on_hide(gui::Message::MainView(main_view::Message::ButtonPoppedOut(
            index,
        )))
        .into()
}

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, gui::Message> {
    let content = column(
        lib.artists
            .iter()
            .enumerate()
            .map(|(i, a)| artist_button(state, i, a)),
    )
    .spacing(BUTTON_SPACING);

    iced_widget::scrollable(content)
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
