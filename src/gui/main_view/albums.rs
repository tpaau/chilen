use std::sync::Arc;

use chilen_backend::music_lib::state::Album;
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
    self, Chilen, DIM_TEXT_ALPHA, SPACING_SMALL,
    font::{SIZE_REGULAR, SIZE_SMALL},
    icons,
    main_view::{
        self, BUTTON_HEIGHT, BUTTON_PADDING, BUTTON_ROUNDING, BUTTON_SPACING, THUMBNAIL_SIZE,
        button_style,
    },
};

pub fn album_button<'a>(
    state: &'a Chilen,
    index: usize,
    maybe_album: &'a Option<Arc<Album>>,
) -> Element<'a, gui::Message> {
    let content: Element<'a, gui::Message> = if let Some(album) = maybe_album {
        button(
            row![
                stack![
                    container(space())
                        .style(|_| {
                            container::Style::default()
                                .background(state.theme.surface_container_high())
                                .border(Border::default().rounded(BUTTON_ROUNDING - BUTTON_PADDING))
                        })
                        .width(Length::Fixed(THUMBNAIL_SIZE))
                        .height(Length::Fixed(THUMBNAIL_SIZE)),
                    center(
                        text(*icons::ALBUM)
                            .font(icons::filled())
                            .color(state.theme.on_surface_variant())
                            .size(icons::SIZE_LARGE)
                    ),
                    album.tracks.iter().find_map(|t| {
                        t.cover_path.as_ref().map(|path| {
                            image(path.clone())
                                .width(Length::Fixed(THUMBNAIL_SIZE))
                                .height(Length::Fixed(THUMBNAIL_SIZE))
                        })
                    }),
                ],
                container(column![
                    text(&album.name)
                        .size(SIZE_REGULAR)
                        .color(state.theme.on_surface())
                        .wrapping(text::Wrapping::None),
                    text({
                        let len = album.artists.len();
                        match len {
                            0 => "No artists".to_string(),
                            1 => "1 artist".to_string(),
                            _ => format!("{len} artists"),
                        }
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
        .height(BUTTON_HEIGHT)
        .padding(Padding::new(BUTTON_PADDING))
        .style(|_, status| button_style(status, &state.theme))
        .on_press(gui::Message::CloseDialog)
        .into()
    } else {
        space().height(BUTTON_HEIGHT).width(Length::Fill).into()
    };
    sensor(content)
        .on_show(move |_| gui::Message::MainView(main_view::Message::AlbumButtonPoppedIn(index)))
        .on_hide(gui::Message::MainView(
            main_view::Message::AlbumButtonPoppedOut(index),
        ))
        .into()
}

pub fn view(state: &Chilen) -> Element<'_, gui::Message> {
    if let Some(albums) = &state.main_view.albums {
        let content = column(
            albums
                .iter()
                .enumerate()
                .map(|(i, a)| album_button(state, i, a)),
        )
        .spacing(BUTTON_SPACING);

        if state.main_view.view == main_view::View::Albums {
            iced_widget::scrollable(content)
                .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
                .into()
        } else {
            content.into()
        }
    } else {
        text("Loading...").into()
    }
}
