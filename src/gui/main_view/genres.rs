use chilen_backend::music_lib::{LibraryItem, is_favorite};
use chilen_widget::virtual_list::VirtualList;
use iced::{Element, Length};
use iced_core::text::IntoFragment;
use iced_m3::{theme::ColorScheme, widget::shapes};
use iced_widget::{center, column, space};

use crate::gui::{
    Chilen, icons,
    main_view::{
        Message, TOOLBAR_OFFSET,
        top_view::{self, TopView},
    },
    widget::{
        icon_shape::IconShape,
        list::{BUTTON_HEIGHT, BUTTON_SPACING, genre_button::GenreButton},
    },
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    if let Some(lib) = &state.library
        && !lib.genres.is_empty()
    {
        let highlighted_genre_name = state.player_state.as_ref().and_then(|p| {
            if let chilen_backend::playback::QueueSource::Genre { name } = &p.queue_source {
                Some(name)
            } else {
                None
            }
        });

        let content = VirtualList {
            model: lib.genres.iter(),
            delegate: Box::new(move |genre| {
                let highlighted = highlighted_genre_name
                    .map(|name| name == genre.name.as_ref())
                    .unwrap_or_default();
                let favorite_item: LibraryItem = genre.clone().into();
                let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                GenreButton {
                    theme: &state.theme,
                    genre: genre.clone(),
                    play: Message::PlayGenre(genre.clone()),
                    press: Message::TopView(top_view::Message::Navigate(TopView::Genre(
                        genre.clone(),
                    ))),
                    shuffle: Message::ShuffleGenre(genre.clone()),
                    add_to_queue: Message::AddGenreToQueue(genre.clone()),
                    toggle_favorite: Message::ToggleFavorite(favorite_item),
                    is_favorite,
                    highlighted,
                    vibrant: state.settings.vibrant_widgets,
                }
                .into()
            }),
            delegate_height: BUTTON_HEIGHT,
            visibilities: state.main_view.visible.as_deref().unwrap_or(&[]),
            list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
            on_show: Box::new(Message::ButtonPoppedIn),
            on_hide: Box::new(Message::ButtonPoppedOut),
        };

        iced_widget::scrollable(column![content, space().height(*TOOLBAR_OFFSET)])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
            .into()
    } else {
        center(
            IconShape::new(
                shapes::polymorpher::shapes::sunny(),
                state.theme.surface_container_high(),
                icons::GENRES.into_fragment(),
                state.theme.on_surface_variant(),
            )
            .label(
                "Your genres will show up here",
                state.theme.on_surface_variant(),
            ),
        )
        .into()
    }
}
