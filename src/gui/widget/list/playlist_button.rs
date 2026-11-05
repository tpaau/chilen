use std::sync::Arc;

use chilen_backend::music_lib::Playlist;
use iced::{
    Alignment, Element, Length, Padding,
    widget::{button, column, container, row, text},
};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::ColorScheme,
    widget::{
        Action,
        hybrid_icon::Icon,
        menu::{self, Menu, button::MenuAction},
    },
};

use crate::gui::{
    Chilen, THUMBNAIL_SIZE,
    font::{self, SIZE_REGULAR, SIZE_SMALL},
    formatter::format_playlist_track_count,
    icons::{self},
    widget::{
        cover_image::CoverImage,
        list::{BUTTON_PADDING, BUTTON_ROUNDING, BUTTON_SPACING, button_style},
        menu_button::MenuButton,
        wrappers::marquee,
    },
};

#[derive(Clone)]
pub struct Messages<Message>
where
    Message: Clone,
{
    pub play: Option<Message>,
    pub shuffle: Option<Message>,
    pub add_to_queue: Option<Message>,
    pub export: Option<Message>,
    pub toggle_favorite: Message,
    pub rename: Message,
    pub delete: Message,
}

pub fn menu<'a, Message, Theme, Renderer>(
    state: &'a Chilen,
    is_favorite: bool,
    messages: Messages<Message>,
) -> Menu<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font> + iced::advanced::svg::Renderer,
{
    iced_m3::widget::menu(
        menu::Style::new(&state.theme, state.settings.vibrant_widgets),
        vec![
            menu::Group {
                label: None,
                items: vec![
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::PLAY_ARROW.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Play".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: messages.play.clone().map(Action::Direct),
                        },
                    }),
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::SHUFFLE.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Shuffle".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: messages.shuffle.clone().map(Action::Direct),
                        },
                    }),
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::ADD_TO_QUEUE.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Add to queue".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: messages.add_to_queue.clone().map(Action::Direct),
                        },
                    }),
                ],
            },
            menu::Group {
                label: None,
                items: vec![
                    {
                        let (icon_font, label) = match is_favorite {
                            true => (icons::filled(), "Unfavorite".into()),
                            false => (icons::outlined(), "Favorite".into()),
                        };
                        menu::Item::Button(menu::button::Button {
                            icon: Some(Icon::Text {
                                text: icons::STAR.into_fragment(),
                                font: Some(icon_font),
                            }),
                            label,
                            supporting_text: None,
                            action: MenuAction::Message {
                                is_selected: false,
                                on_press: Some(Action::Direct(messages.toggle_favorite.clone())),
                            },
                        })
                    },
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::UPLOAD.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Export".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: messages.export.clone().map(Action::Direct),
                        },
                    }),
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::IMAGE.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Change image".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: None,
                        },
                    }),
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::EDIT.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Rename".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: Some(Action::Direct(messages.rename.clone())),
                        },
                    }),
                    menu::Item::Separator,
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::DELETE.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Delete".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: Some(Action::Direct(messages.delete.clone())),
                        },
                    }),
                ],
            },
        ],
    )
}

pub struct PlaylistButton<'a, Message>
where
    Message: Clone,
{
    pub state: &'a Chilen,
    pub playlist: &'a Arc<Playlist>,
    pub press: Message,
    pub toggle_menu: Message,
    pub messages: Messages<Message>,
    pub highlighted: bool,
    pub is_favorite: bool,
    pub menu_opened: bool,
}

impl<'a, Message> From<PlaylistButton<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: PlaylistButton<'a, Message>) -> Self {
        let thumbnail_border_radius = BUTTON_ROUNDING - BUTTON_PADDING;

        let font = if value.highlighted {
            font::bold()
        } else {
            font::regular()
        };
        let title = marquee(
            text(value.playlist.name.as_ref())
                .size(SIZE_REGULAR)
                .color(value.state.theme.on_surface())
                .font(font)
                .wrapping(text::Wrapping::None),
        );

        // TODO: Maybe an animated indicator would look better?
        let (icon_color, container_color) = match value.highlighted {
            true => (
                value.state.theme.on_secondary_container(),
                value.state.theme.secondary_container(),
            ),
            false => (
                value.state.theme.on_surface_variant(),
                value.state.theme.surface_container_high(),
            ),
        };
        let image_path = (!value.highlighted)
            .then_some(value.playlist.cover.thumbnail().ok())
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

        let menu_button = MenuButton::new(
            move || menu(value.state, value.is_favorite, value.messages.clone()).into(),
            &value.state.theme,
            value.toggle_menu,
        )
        .opened(value.menu_opened);

        let content = row![
            cover,
            container(column![
                title,
                marquee(
                    text(format_playlist_track_count(value.playlist.tracks.len()))
                        .size(SIZE_SMALL)
                        .wrapping(text::Wrapping::None)
                        .color(value.state.theme.on_surface_variant())
                ),
            ])
            .width(Length::Fill)
            .clip(true)
            .center_y(Length::Fill),
            menu_button,
        ]
        .align_y(Alignment::Center)
        .spacing(BUTTON_SPACING);

        button(content)
            .padding(Padding::new(BUTTON_PADDING))
            .style(|_, status| button_style(status, value.state.theme.on_surface()))
            .on_press(value.press)
            .into()
    }
}
