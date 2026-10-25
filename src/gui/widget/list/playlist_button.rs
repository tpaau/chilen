use std::sync::Arc;

use chilen_backend::music_lib::{LibraryItem, Playlist, is_favorite};
use iced::{
    Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::ColorScheme,
    widget::{hybrid_icon::Icon, vertical_menu},
};
use iced_widget::{hover, sensor};

use crate::gui::{
    Chilen, THUMBNAIL_SIZE,
    font::{self, SIZE_REGULAR, SIZE_SMALL},
    formatter::format_playlist_track_count,
    icons::{self, icon_filled},
    main_view::{self, top_view},
    widget::{
        self,
        cover_image::CoverImage,
        list::{BUTTON_HEIGHT, BUTTON_PADDING, BUTTON_ROUNDING, BUTTON_SPACING, button_style},
        wrappers::marquee,
    },
};

pub fn playlist_button<'a>(
    state: &'a Chilen,
    playlist: &'a Arc<Playlist>,
    index: usize,
    highlighted: bool,
    vibrant: bool,
) -> Element<'a, main_view::Message> {
    let content: Element<'_, main_view::Message> = if let Some(visible) = &state.main_view.visible
        && let Some(val) = visible.get(index)
        && *val
    {
        let thumbnail_border_radius = BUTTON_ROUNDING - BUTTON_PADDING;
        let menu = iced_m3::widget::menu(
            vec![
                vertical_menu::Group {
                    label: None,
                    entries: vec![
                        vertical_menu::Entry::Button {
                            icon: Some(Icon::Text {
                                text: icons::PLAY_ARROW.into_fragment(),
                                font: Some(icons::filled()),
                            }),
                            label: "Play".into(),
                            supporting_text: None,
                            error: false,
                            action: vertical_menu::Action::Message(
                                (!playlist.tracks.is_empty())
                                    .then_some(main_view::Message::PlayPlaylist(playlist.clone())),
                            ),
                        },
                        vertical_menu::Entry::Button {
                            icon: Some(Icon::Text {
                                text: icons::SHUFFLE.into_fragment(),
                                font: Some(icons::filled()),
                            }),
                            label: "Shuffle".into(),
                            supporting_text: None,
                            error: false,
                            action: vertical_menu::Action::Message(
                                (!playlist.tracks.is_empty()).then_some(
                                    main_view::Message::ShufflePlaylist(playlist.clone()),
                                ),
                            ),
                        },
                        vertical_menu::Entry::Button {
                            icon: Some(Icon::Text {
                                text: icons::ADD_TO_QUEUE.into_fragment(),
                                font: Some(icons::filled()),
                            }),
                            label: "Add to queue".into(),
                            supporting_text: None,
                            error: false,
                            action: vertical_menu::Action::Message(
                                (!playlist.tracks.is_empty()).then_some(
                                    main_view::Message::AddPlaylistToQueue(playlist.clone()),
                                ),
                            ),
                        },
                    ],
                },
                vertical_menu::Group {
                    label: None,
                    entries: vec![
                        {
                            let favorite_item: LibraryItem = playlist.clone().into();
                            let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                            let (icon_font, label) = match is_favorite {
                                true => (icons::filled(), "Unfavorite".into()),
                                false => (icons::outlined(), "Favorite".into()),
                            };
                            vertical_menu::Entry::Button {
                                icon: Some(Icon::Text {
                                    text: icons::STAR.into_fragment(),
                                    font: Some(icon_font),
                                }),
                                label,
                                supporting_text: None,
                                error: false,
                                action: vertical_menu::Action::Message(Some(
                                    main_view::Message::ToggleFavorite(favorite_item),
                                )),
                            }
                        },
                        vertical_menu::Entry::Button {
                            icon: Some(Icon::Text {
                                text: icons::UPLOAD.into_fragment(),
                                font: Some(icons::filled()),
                            }),
                            label: "Export".into(),
                            supporting_text: None,
                            error: false,
                            action: vertical_menu::Action::Message(
                                (!playlist.tracks.is_empty()).then_some(
                                    main_view::Message::ExportPlaylist(
                                        playlist.name.clone().into_string(),
                                    ),
                                ),
                            ),
                        },
                        vertical_menu::Entry::Button {
                            icon: Some(Icon::Text {
                                text: icons::IMAGE.into_fragment(),
                                font: Some(icons::filled()),
                            }),
                            label: "Change image".into(),
                            supporting_text: None,
                            error: false,
                            action: vertical_menu::Action::Message(None),
                        },
                        vertical_menu::Entry::Button {
                            icon: Some(Icon::Text {
                                text: icons::EDIT.into_fragment(),
                                font: Some(icons::filled()),
                            }),
                            label: "Rename".into(),
                            supporting_text: None,
                            error: false,
                            action: vertical_menu::Action::Message(Some(
                                main_view::Message::OpenPlaylistRenameDialog {
                                    playlist: playlist.name.clone().into_string(),
                                    name: playlist.name.clone().into_string(),
                                },
                            )),
                        },
                        vertical_menu::Entry::Separator,
                        vertical_menu::Entry::Button {
                            icon: Some(Icon::Text {
                                text: icons::DELETE.into_fragment(),
                                font: Some(icons::filled()),
                            }),
                            label: "Delete".into(),
                            supporting_text: None,
                            error: true,
                            action: vertical_menu::Action::Message(Some(
                                main_view::Message::ConfirmPlaylistDeletion(playlist.clone()),
                            )),
                        },
                    ],
                },
            ],
            vertical_menu::Style::new(&state.theme, vibrant),
            font::regular(),
        );

        let font = if highlighted {
            font::bold()
        } else {
            font::regular()
        };
        let title = marquee(
            text(playlist.name.as_ref())
                .size(SIZE_REGULAR)
                .color(state.theme.on_surface())
                .font(font)
                .wrapping(text::Wrapping::None),
        );

        // TODO: Maybe an animated indicator would look better?
        let (icon_color, container_color) = match highlighted {
            true => (
                state.theme.on_secondary_container(),
                state.theme.secondary_container(),
            ),
            false => (
                state.theme.on_surface_variant(),
                state.theme.surface_container_high(),
            ),
        };
        let image_path = (!highlighted)
            .then_some(playlist.cover.thumbnail().ok())
            .flatten();

        let cover = CoverImage {
            image_path,
            icon: *icons::LIBRARY_MUSIC,
            icon_size: icons::SIZE_LARGE,
            icon_color,
            container_color,
            radius: thumbnail_border_radius.into(),
            opacity: 1.0,
            width: THUMBNAIL_SIZE.into(),
            height: THUMBNAIL_SIZE.into(),
        };

        let content = row![
            cover,
            container(column![
                title,
                marquee(
                    text(format_playlist_track_count(playlist.tracks.len()))
                        .size(SIZE_SMALL)
                        .wrapping(text::Wrapping::None)
                        .color(state.theme.on_surface_variant())
                ),
            ])
            .width(Length::Fill)
            .clip(true)
            .center_y(Length::Fill),
            space().width(widget::list::ICON_SIZE),
        ]
        .spacing(BUTTON_SPACING);

        let button = button(content)
            .padding(Padding::new(BUTTON_PADDING))
            .style(|_, status| button_style(status, state.theme.on_surface()))
            .on_press_with(|| {
                main_view::Message::TopView(top_view::Message::Navigate(playlist.clone().into()))
            });

        // TODO: Should be more like a button
        let menu = container(iced_m3::widget::advanced::drop_down_menu(
            |_| {
                icon_filled(*icons::MORE_HORIZ, widget::list::ICON_SIZE)
                    .color(state.theme.on_surface())
                    .into()
            },
            Some(menu),
            iced_m3::widget::advanced::drop_down_menu::Placement::BottomRight,
        ))
        .center_y(Length::Fill)
        .align_right(Length::Fill)
        .padding(BUTTON_PADDING);

        hover(button, menu)
    } else {
        space().width(Length::Fill).height(BUTTON_HEIGHT).into()
    };

    sensor(content)
        .on_show(move |_| main_view::Message::ButtonPoppedIn(index))
        .on_hide(main_view::Message::ButtonPoppedOut(index))
        .into()
}
