use std::sync::Arc;

use chilen_backend::music_lib::Album;
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

use crate::gui::{
    Chilen, SPACING_SMALL, SPACING_SMALLER,
    font::{self},
    formatter::{format_album_track_count, format_artist_count, format_date_maybe},
    icons::{self},
    widget::{
        cover_image::CoverImage,
        list::{BUTTON_PADDING, BUTTON_ROUNDING, THUMBNAIL_SIZE, button_style},
        menu_button::MenuButton,
        text_spacer::text_spacer,
        wrappers::marquee,
    },
};

pub enum Info {
    ArtistCount,
    TrackCount,
    Date,
}

pub struct AlbumButton<'a, Message> {
    pub state: &'a Chilen,
    pub album: Arc<Album>,
    pub info: Vec<Info>,
    pub play: Message,
    pub press: Message,
    pub shuffle: Message,
    pub add_to_queue: Message,
    pub toggle_favorite: Message,
    pub toggle_menu: Message,
    pub menu_opened: bool,
    pub is_favorite: bool,
    pub highlighted: bool,
}

pub fn menu<'a, Message, Theme, Renderer>(
    state: &'a Chilen,
    is_favorite: bool,
    play: Message,
    shuffle: Message,
    add_to_queue: Message,
    toggle_favorite: Message,
) -> Menu<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font> + iced::advanced::svg::Renderer,
{
    let (icon_font, label) = match is_favorite {
        true => (icons::filled(), "Unfavorite".into()),
        false => (icons::outlined(), "Favorite".into()),
    };
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
                on_press: Some(Action::Direct(play)),
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
                on_press: Some(Action::Direct(shuffle)),
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
                on_press: Some(Action::Direct(add_to_queue)),
            },
        }),
    ];

    let first_group = menu::Group::new(items);

    let second_group = menu::Group {
        label: None,
        items: vec![menu::Item::Button(menu::button::Button {
            icon: Some(Icon::Text {
                text: icons::STAR.into_fragment(),
                font: Some(icon_font),
            }),
            label,
            supporting_text: None,
            action: MenuAction::Message {
                is_selected: false,
                on_press: Some(Action::Direct(toggle_favorite)),
            },
        })],
    };

    iced_m3::widget::menu(
        menu::Style::new(&state.theme, state.settings.vibrant_widgets),
        vec![first_group, second_group],
    )
}

impl<'a, Message> From<AlbumButton<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: AlbumButton<'a, Message>) -> Self {
        let thumbnail_border_radius = BUTTON_ROUNDING - BUTTON_PADDING;

        let info: Vec<_> = value
            .info
            .into_iter()
            .map(|i| match i {
                Info::ArtistCount => format_artist_count(value.album.artists.len()),
                Info::TrackCount => format_album_track_count(value.album.tracks.len()),
                Info::Date => format_date_maybe(value.album.date),
            })
            .collect();

        let info_len = info.len();
        let mut info_text: Vec<Element<'_, Message>> = Vec::with_capacity(2 * info_len - 1);
        for (i, info) in info.into_iter().enumerate() {
            info_text.push(
                text(info)
                    .size(font::SIZE_SMALL)
                    .color(value.state.theme.on_surface_variant())
                    .wrapping(text::Wrapping::None)
                    .into(),
            );
            if let Some(len) = info_len.checked_sub(1)
                && i < len
            {
                info_text.push(text_spacer(
                    value.state.theme.on_surface_variant(),
                    font::SIZE_SMALL,
                ));
            }
        }

        let font = if value.highlighted {
            font::bold()
        } else {
            font::regular()
        };
        let title = marquee(
            text(value.album.title.clone().into_string())
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
            .then_some(value.album.cover.thumbnail().ok())
            .flatten();

        let cover = CoverImage {
            image_path,
            icon: *icons::ALBUM,
            icon_size: icons::SIZE_LARGE,
            icon_color,
            container_color,
            radius: thumbnail_border_radius.into(),
            opacity: 1.0,
            width: THUMBNAIL_SIZE.into(),
            height: THUMBNAIL_SIZE.into(),
        };

        let menu_button = MenuButton::new(
            move || {
                menu(
                    value.state,
                    value.is_favorite,
                    value.play.clone(),
                    value.shuffle.clone(),
                    value.add_to_queue.clone(),
                    value.toggle_favorite.clone(),
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
                    row(info_text)
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
