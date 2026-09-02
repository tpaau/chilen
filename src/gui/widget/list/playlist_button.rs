use std::sync::Arc;

use chilen_backend::music_lib::Playlist;
use iced::{
    Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::sensor;

use crate::gui::{
    Chilen, THUMBNAIL_SIZE,
    font::{self, SIZE_REGULAR, SIZE_SMALL},
    icons, playlist_view,
    widget::{
        cover_image::cover_image,
        list::{BUTTON_HEIGHT, BUTTON_PADDING, BUTTON_ROUNDING, BUTTON_SPACING, button_style},
    },
};

pub fn playlist_button<'a>(
    state: &'a Chilen,
    playlist: &'a Arc<Playlist>,
    index: usize,
    highlighted: bool,
) -> Element<'a, playlist_view::Message> {
    let content: Element<'_, playlist_view::Message> = if let Some(visible) =
        &state.playlist_view.visible
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
                            icon: Some(&icons::PLAY_ARROW),
                            label: "Play",
                            supporting_text: None,
                            error: false,
                            action: vertical_menu::Action::Message(Some(
                                playlist_view::Message::PlayPlaylist(playlist.clone()),
                            )),
                        },
                        vertical_menu::Entry::Button {
                            icon: Some(&icons::SHUFFLE),
                            label: "Shuffle",
                            supporting_text: None,
                            error: false,
                            action: vertical_menu::Action::Message(Some(
                                playlist_view::Message::ShufflePlaylist(playlist.clone()),
                            )),
                        },
                        vertical_menu::Entry::Button {
                            icon: Some(&icons::ADD_TO_QUEUE),
                            label: "Add to queue",
                            supporting_text: None,
                            error: false,
                            action: vertical_menu::Action::Message(Some(
                                playlist_view::Message::AddPlaylistToQueue(playlist.clone()),
                            )),
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
                                playlist_view::Message::ExportPlaylist(playlist.name.clone()),
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
                                playlist_view::Message::ConfirmPlaylistDeletion(playlist.clone()),
                            )),
                        },
                    ],
                },
            ],
            &state.theme,
        )
        .icon_font(icons::filled());

        let font = if highlighted {
            font::font_bold()
        } else {
            font::font()
        };
        let title = text(playlist.name.clone())
            .size(SIZE_REGULAR)
            .color(state.theme.on_surface())
            .font(font)
            .wrapping(text::Wrapping::None);

        // TODO: Maybe an animated indicator would look better?
        let content_color = if highlighted {
            state.theme.on_secondary_container()
        } else {
            state.theme.on_surface_variant()
        };
        let container_color = if highlighted {
            state.theme.secondary_container()
        } else {
            state.theme.surface_container_high()
        };
        let cover = cover_image(
            (!highlighted)
                .then_some(playlist.cover.thumbnail.clone())
                .flatten(),
            &icons::PLAYLIST_PLAY,
            icons::SIZE_LARGE,
            content_color,
            container_color,
            thumbnail_border_radius,
            1.0,
        )
        .width(Length::Fixed(THUMBNAIL_SIZE))
        .height(Length::Fixed(THUMBNAIL_SIZE));

        button(
            row![
                cover,
                container(column![
                    title,
                    text({
                        if playlist.tracks.is_empty() {
                            "Empty".to_string()
                        } else {
                            format!("{} tracks", playlist.tracks.len())
                        }
                    })
                    .size(SIZE_SMALL)
                    .wrapping(text::Wrapping::None)
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
                                .color(state.theme.on_surface())
                                .into()
                        },
                        Some(menu),
                        iced_m3::widget::drop_down_menu::Placement::BottomRight,
                    ),
                )
                .center_y(Length::Fill),
            ]
            .spacing(BUTTON_SPACING),
        )
        .padding(Padding::new(BUTTON_PADDING))
        .style(|_, status| button_style(status, state.theme.on_surface()))
        .on_press_with(|| playlist_view::Message::OpenPlaylist(playlist.clone()))
        .into()
    } else {
        space().width(Length::Fill).height(BUTTON_HEIGHT).into()
    };

    sensor(content)
        .on_show(move |_| playlist_view::Message::ButtonPoppedIn(index))
        .on_hide(playlist_view::Message::ButtonPoppedOut(index))
        .into()
}
