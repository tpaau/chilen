use std::sync::Arc;

use chilen_backend::music_lib::state::{Album, MusicLibrary};
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
    BUTTON_HEIGHT, BUTTON_PADDING, BUTTON_SPACING, Chilen, SPACING_SMALL, SPACING_SMALLER,
    THUMBNAIL_SIZE,
    font::{self},
    icons,
    main_view::{
        self, BUTTON_ROUNDING, button_style,
        top_view::{self, TopView},
    },
    widget::text_spacer::text_spacer,
};

pub fn album_button<'a>(
    state: &'a Chilen,
    index: usize,
    album: &'a Arc<Album>,
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
                        text(*icons::ALBUM)
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
                    album.cover.thumbnail.as_ref().map(|t| {
                        image(t)
                            .content_fit(iced::ContentFit::Cover)
                            .width(Length::Fixed(THUMBNAIL_SIZE))
                            .height(Length::Fixed(THUMBNAIL_SIZE))
                            .border_radius(thumbnail_border_radius)
                    }),
                ],
                container(column![
                    text(&album.title)
                        .size(font::SIZE_REGULAR)
                        .color(state.theme.on_surface())
                        .wrapping(text::Wrapping::None),
                    row![
                        text(match album.tracks.len() {
                            1 => "Single".to_string(),
                            _ => format!("{} tracks", album.tracks.len()),
                        })
                        .size(font::SIZE_SMALL)
                        .color(state.theme.on_surface_variant())
                        .wrapping(text::Wrapping::None),
                        text_spacer(state.theme.on_surface_variant(), font::SIZE_SMALL),
                        text(match album.artists.len() {
                            1 => "1 artist".to_string(),
                            _ => format!("{} artists", album.artists.len()),
                        })
                        .size(font::SIZE_SMALL)
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
                                                icon: Some(&icons::SHUFFLE),
                                                label: "Shuffle",
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
        .on_press_with(|| {
            main_view::Message::TopView(top_view::Message::Navigate(TopView::Album(album.clone())))
        })
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
        lib.albums
            .iter()
            .enumerate()
            .map(|(i, a)| album_button(state, i, a)),
    )
    .spacing(BUTTON_SPACING);

    iced_widget::scrollable(content)
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
