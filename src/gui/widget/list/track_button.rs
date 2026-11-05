use std::sync::Arc;

use chilen_backend::music_lib::Track;
use iced::{Alignment, Length};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::ColorScheme,
    widget::{
        Action,
        hybrid_icon::Icon,
        menu::{self, button::MenuAction},
    },
};
use iced_widget::{button, column, container, row, text};

use crate::gui::{
    Chilen, SPACING_SMALL, font,
    formatter::format_track_duration,
    icons::{self},
    widget::{
        cover_image::CoverImage,
        list::{BUTTON_PADDING, BUTTON_ROUNDING, THUMBNAIL_SIZE, button_style},
        menu_button::MenuButton,
        wrappers::marquee,
    },
};

pub enum Info {
    Artist,
    Length,
    Album,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Idle,
    Playing,
    Dimmed,
}

pub struct Messages<Message> {
    pub play: Message,
    pub press: Message,
    pub toggle_favorite: Message,
    pub shuffle: Option<Message>,
    pub add_to_queue: Option<Message>,
    pub add_to_playlist: Message,
    pub details: Option<Message>, // TODO: Shouldn't be optional
    pub remove: Option<Message>,
    pub toggle_menu: Message,
}

pub struct TrackButton<'a, Message>
where
    Message: 'a + Clone,
{
    pub state: &'a Chilen,
    pub track: Arc<Track>,
    pub messages: Messages<Message>,
    pub info: Info,
    pub is_favorite: bool,
    pub status: Status,
    pub vibrant: bool,
    pub menu_opened: bool,
}

impl<'a, Message> From<TrackButton<'a, Message>> for iced::Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: TrackButton<'a, Message>) -> Self {
        let thumbnail_border_radius = BUTTON_ROUNDING - BUTTON_PADDING;
        let opacity = match value.status {
            Status::Idle | Status::Playing => 1.0,
            Status::Dimmed => 0.5,
        };

        let title_font = match value.status {
            Status::Playing => font::bold(),
            Status::Idle | Status::Dimmed => font::regular(),
        };
        let title = value
            .track
            .title
            .as_ref()
            .map(|t| t.clone().into_string())
            .unwrap_or("Unknown".to_string());
        let title = marquee(
            text(title)
                .size(font::SIZE_REGULAR)
                .font(title_font)
                .color(value.state.theme.on_surface().scale_alpha(opacity))
                .wrapping(text::Wrapping::None),
        );

        let info = match value.info {
            Info::Artist => value
                .track
                .artists
                .as_ref()
                .map(|a| a.join(&value.state.settings.value_separator))
                .unwrap_or("Unknown".to_string()),
            Info::Length => format_track_duration(value.track.duration),
            Info::Album => value
                .track
                .album
                .as_ref()
                .map(|a| a.clone().into_string())
                .unwrap_or("Unknown".to_string()),
        };

        // TODO: Maybe an animated indicator would look better?
        let icon_color = if value.status == Status::Playing {
            value.state.theme.on_secondary_container()
        } else {
            value.state.theme.on_surface_variant()
        };
        let container_color = if value.status == Status::Playing {
            value.state.theme.secondary_container()
        } else {
            value.state.theme.surface_container_high()
        };
        let image_path = (value.status != Status::Playing)
            .then_some(value.track.cover.thumbnail().ok())
            .flatten();
        let cover = CoverImage {
            image_path,
            icon: *icons::MUSIC_NOTE,
            icon_size: icons::SIZE_LARGE,
            icon_color,
            container_color,
            radius: thumbnail_border_radius.into(),
            opacity,
            width: THUMBNAIL_SIZE.into(),
            height: THUMBNAIL_SIZE.into(),
        };

        let menu_button = MenuButton::new(
            move || {
                let mut second_group_items = vec![
                    {
                        let (icon_font, label) = match value.is_favorite {
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
                                on_press: Some(Action::Direct(
                                    value.messages.toggle_favorite.clone(),
                                )),
                            },
                        })
                    },
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::PLAYLIST_ADD.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Add to playlist".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: Some(Action::Direct(value.messages.add_to_playlist.clone())),
                        },
                    }),
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::INFO.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Details".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: value.messages.details.clone().map(|m| Action::Direct(m)),
                        },
                    }),
                ];

                if let Some(remove) = value.messages.remove.clone() {
                    second_group_items.push(menu::Item::Separator);
                    second_group_items.push(menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::DELETE.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Remove".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: Some(Action::Direct(remove)),
                        },
                    }));
                }

                // TODO: Optimize allocations
                let mut first_group_items = vec![menu::Item::Button(menu::button::Button {
                    icon: Some(Icon::Text {
                        text: icons::PLAY_ARROW.into_fragment(),
                        font: Some(icons::filled()),
                    }),
                    label: "Play".into(),
                    supporting_text: None,
                    action: MenuAction::Message {
                        is_selected: false,
                        on_press: Some(Action::Direct(value.messages.play.clone())),
                    },
                })];

                if let Some(message) = value.messages.shuffle.clone() {
                    first_group_items.push(menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::SHUFFLE.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Shuffle".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: Some(Action::Direct(message)),
                        },
                    }));
                }

                if let Some(message) = value.messages.add_to_queue.clone() {
                    first_group_items.push(menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::ADD_TO_QUEUE.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Add to queue".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: Some(Action::Direct(message)),
                        },
                    }));
                }

                let menu_groups = vec![
                    menu::Group::new(first_group_items),
                    menu::Group::new(second_group_items),
                ];

                iced_m3::widget::menu(
                    menu::Style::new(&value.state.theme, value.vibrant),
                    menu_groups,
                )
                .into()
            },
            &value.state.theme,
            value.messages.toggle_menu,
        )
        .opened(value.menu_opened);

        let content = row![
            cover,
            container(column![
                title,
                marquee(
                    text(info)
                        .size(font::SIZE_SMALL)
                        .color(value.state.theme.on_surface_variant().scale_alpha(opacity))
                        .wrapping(text::Wrapping::None)
                ),
            ])
            .width(Length::Fill)
            .clip(true)
            .center_y(Length::Fill),
            menu_button,
        ]
        .align_y(Alignment::Center)
        .spacing(SPACING_SMALL);

        button(content)
            .on_press(value.messages.press)
            .padding(BUTTON_PADDING)
            .style(|_, status| button_style(status, value.state.theme.on_surface_variant()))
            .into()
    }
}
