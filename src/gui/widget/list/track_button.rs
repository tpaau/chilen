use std::sync::Arc;

use chilen_backend::music_lib::Track;
use iced::Length;
use iced_m3::{
    theme::ColorScheme,
    widget::{drop_down_menu, vertical_menu},
};
use iced_widget::{button, column, container, row, text};

use crate::gui::{
    Chilen, SPACING_SMALL, font,
    formatter::format_track_duration,
    icons,
    widget::{
        cover_image::CoverImage,
        list::{BUTTON_PADDING, BUTTON_ROUNDING, THUMBNAIL_SIZE, button_style},
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
    pub shuffle: Option<Message>,
    pub add_to_queue: Option<Message>,
    pub add_to_playlist: Message,
    // TODO: Shouldn't be optional
    pub details: Option<Message>,
    pub remove: Option<Message>,
}

pub struct TrackButton<'a, Message>
where
    Message: 'a + Clone,
{
    pub state: &'a Chilen,
    pub track: Arc<Track>,
    pub messages: Messages<Message>,
    pub info: Info,
    pub status: Status,
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

        let mut second_group_entries = vec![
            vertical_menu::Entry::Button {
                icon: Some(&icons::PLAYLIST_PLAY),
                label: "Add to playlist",
                supporting_text: None,
                error: false,
                action: vertical_menu::Action::Message(Some(value.messages.add_to_playlist)),
            },
            vertical_menu::Entry::Button {
                icon: Some(&icons::INFO),
                label: "Details",
                supporting_text: None,
                error: false,
                action: vertical_menu::Action::Message(value.messages.details),
            },
        ];

        if let Some(remove) = value.messages.remove {
            second_group_entries.push(vertical_menu::Entry::Separator);
            second_group_entries.push(vertical_menu::Entry::Button {
                icon: Some(&icons::DELETE),
                label: "Remove",
                supporting_text: None,
                error: true,
                action: vertical_menu::Action::Message(Some(remove)),
            });
        }

        let mut first_group_entries = vec![vertical_menu::Entry::Button {
            icon: Some(&icons::PLAY_ARROW),
            label: "Play",
            supporting_text: None,
            error: false,
            action: vertical_menu::Action::Message(Some(value.messages.play)),
        }];

        if let Some(message) = value.messages.shuffle {
            first_group_entries.push(vertical_menu::Entry::Button {
                icon: Some(&icons::SHUFFLE),
                label: "Shuffle",
                supporting_text: None,
                error: false,
                action: vertical_menu::Action::Message(Some(message)),
            });
        }

        if let Some(message) = value.messages.add_to_queue {
            first_group_entries.push(vertical_menu::Entry::Button {
                icon: Some(&icons::ADD_TO_QUEUE),
                label: "Add to queue",
                supporting_text: None,
                error: false,
                action: vertical_menu::Action::Message(Some(message)),
            });
        }

        let menu_groups = vec![
            vertical_menu::Group {
                label: None,
                entries: first_group_entries,
            },
            vertical_menu::Group {
                label: None,
                entries: second_group_entries,
            },
        ];

        let menu =
            iced_m3::widget::menu(menu_groups, &value.state.theme).icon_font(icons::filled());

        let title_font = match value.status {
            Status::Playing => font::font_bold(),
            Status::Idle | Status::Dimmed => font::font(),
        };
        let title = text(if let Some(title) = value.track.title.clone() {
            title
        } else {
            "Unknown".to_string()
        })
        .size(font::SIZE_REGULAR)
        .font(title_font)
        .color(value.state.theme.on_surface().scale_alpha(opacity))
        .wrapping(text::Wrapping::None);

        let info = match value.info {
            Info::Artist => {
                if let Some(artists) = &value.track.artists {
                    artists.join(&value.state.settings.value_separator)
                } else {
                    "Unknown".to_string()
                }
            }
            Info::Length => format_track_duration(value.track.duration),
            Info::Album => value.track.album.clone().unwrap_or("Unknown".to_string()),
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
            .then_some(value.track.cover.thumbnail.clone())
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

        button(
            row![
                cover,
                container(column![
                    title,
                    text(info)
                        .size(font::SIZE_SMALL)
                        .color(value.state.theme.on_surface_variant().scale_alpha(opacity))
                        .wrapping(text::Wrapping::None),
                ])
                .width(Length::Fill)
                .clip(true)
                .center_y(Length::Fill),
                container(
                    // TODO: Should be more like a button
                    drop_down_menu(
                        move |_| {
                            text(*icons::MORE_HORIZ)
                                .font(icons::filled())
                                .size(icons::SIZE_REGULAR)
                                .color(value.state.theme.on_surface())
                                .into()
                        },
                        Some(menu),
                        iced_m3::widget::drop_down_menu::Placement::BottomRight,
                    ),
                )
                .center_y(Length::Fill),
            ]
            .spacing(SPACING_SMALL),
        )
        .on_press(value.messages.press)
        .padding(BUTTON_PADDING)
        .style(|_, status| button_style(status, value.state.theme.on_surface_variant()))
        .into()
    }
}
