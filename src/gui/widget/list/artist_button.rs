use std::sync::Arc;

use chilen_backend::music_lib::Artist;
use iced::{Alignment, Element, Length};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::ColorScheme,
    widget::{
        Action,
        hybrid_icon::Icon,
        menu::{self, Menu, button::MenuAction},
    },
};
use iced_widget::{button, column, container, row, text};

use crate::{
    THUMBNAIL_SIZE,
    gui::{
        Chilen, SPACING_SMALL, SPACING_SMALLER, font,
        formatter::{format_album_count, format_track_count},
        icons::{self},
        widget::{
            cover_image::CoverImage,
            list::{BUTTON_PADDING, button_style},
            menu_button::MenuButton,
            text_spacer::text_spacer,
            wrappers::marquee,
        },
    },
};

pub fn menu<'a, Message>(
    state: &'a Chilen,
    play: Message,
    shuffle: Message,
    add_to_queue: Message,
    toggle_favorite: Message,
    is_favorite: bool,
) -> Menu<'a, Message>
where
    Message: Clone,
{
    let items = vec![
        menu::Item::Button(menu::button::Button {
            icon: Some(Icon::Text {
                text: icons::PLAY_ARROW.into_fragment(),
                font: Some(icons::filled()),
            }),
            label: "Play".into(),
            supporting_text: None,
            action: MenuAction::Message {
                is_selected: false,
                on_press: Some(Action::Direct(play.clone())),
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
                on_press: Some(Action::Direct(shuffle.clone())),
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
                on_press: Some(Action::Direct(add_to_queue.clone())),
            },
        }),
    ];

    let first_group = menu::Group::new(items);

    let second_group = menu::Group {
        label: None,
        items: vec![{
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
                    on_press: Some(Action::Direct(toggle_favorite.clone())),
                },
            })
        }],
    };

    iced_m3::widget::menu(
        menu::Style::new(&state.theme, state.settings.vibrant_widgets),
        vec![first_group, second_group],
    )
}

pub struct ArtistButton<'a, Message>
where
    Message: 'a + Clone,
{
    pub state: &'a Chilen,
    pub artist: Arc<Artist>,
    pub play: Message,
    pub press: Message,
    pub shuffle: Message,
    pub add_to_queue: Message,
    pub toggle_favorite: Message,
    pub toggle_menu: Message,
    pub is_favorite: bool,
    pub highlighted: bool,
    pub vibrant: bool,
    pub menu_opened: bool,
}

impl<'a, Message> From<ArtistButton<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: ArtistButton<'a, Message>) -> Self {
        let font = if value.highlighted {
            font::bold()
        } else {
            font::regular()
        };
        let title = marquee(
            text(value.artist.name.clone().into_string())
                .size(font::SIZE_REGULAR)
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
            .then_some(value.artist.cover.thumbnail().ok())
            .flatten();

        let cover = CoverImage {
            image_path,
            icon: *icons::ARTIST,
            icon_size: icons::SIZE_LARGE,
            icon_color,
            container_color,
            radius: f32::MAX.into(),
            opacity: 1.0,
            width: THUMBNAIL_SIZE.into(),
            height: THUMBNAIL_SIZE.into(),
        };

        let menu_button = MenuButton::new(
            move || {
                menu(
                    value.state,
                    value.play.clone(),
                    value.shuffle.clone(),
                    value.add_to_queue.clone(),
                    value.toggle_favorite.clone(),
                    value.is_favorite,
                )
                .into()
            },
            &value.state.theme,
            value.toggle_menu,
        )
        .opened(value.menu_opened);

        let content = row![
            cover,
            container(column![
                title,
                marquee(
                    row![
                        text(format_album_count(value.artist.albums.len()))
                            .size(font::SIZE_SMALL)
                            .color(value.state.theme.on_surface_variant())
                            .wrapping(text::Wrapping::None),
                        text_spacer(value.state.theme.on_surface_variant(), font::SIZE_SMALL),
                        text(format_track_count(value.artist.tracks.len()))
                            .size(font::SIZE_SMALL)
                            .color(value.state.theme.on_surface_variant())
                            .wrapping(text::Wrapping::None),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(SPACING_SMALLER)
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
            .padding(BUTTON_PADDING)
            .style(|_, status| button_style(status, value.state.theme.on_surface_variant()))
            .on_press(value.press)
            .into()
    }
}
