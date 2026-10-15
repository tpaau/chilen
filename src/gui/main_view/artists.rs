use iced::Element;
use iced_core::text::IntoFragment;
use iced_m3::{theme::ColorScheme, widget::shapes};
use iced_widget::{center, column};

use crate::gui::{
    Chilen, icons,
    main_view::{
        Message,
        top_view::{self, TopView},
    },
    widget::{
        icon_shape::IconShape,
        list::{BUTTON_HEIGHT, BUTTON_SPACING, artist_button::ArtistButton},
        virtual_list::VirtualList,
    },
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    match &state.library {
        Some(lib) => {
            let highlighted_artist_name = state.player_state.as_ref().and_then(|p| {
                if let chilen_backend::playback::QueueSource::Artist { name } = &p.queue_source {
                    Some(name)
                } else {
                    None
                }
            });

            let content = VirtualList {
                model: lib.artists.iter(),
                delegate: Box::new(move |artist| {
                    let highlighted = highlighted_artist_name
                        .map(|name| name == artist.name.as_ref())
                        .unwrap_or_default();
                    ArtistButton {
                        theme: &state.theme,
                        artist: artist.clone(),
                        play: Message::PlayArtist(artist.clone()),
                        press: Message::TopView(top_view::Message::Navigate(TopView::Artist(
                            artist.clone(),
                        ))),
                        shuffle: Message::ShuffleArtist(artist.clone()),
                        add_to_queue: Message::AddArtistToQueue(artist.clone()),
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

            iced_widget::scrollable(content)
                .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
                .into()
        }
        None => center(
            IconShape::new(
                shapes::polymorpher::shapes::sunny(),
                state.theme.surface_container_high(),
                icons::ARTIST.into_fragment(),
                state.theme.on_surface_variant(),
            )
            .label(
                "Your artists will show up here",
                state.theme.on_surface_variant(),
            ),
        )
        .into(),
    }
}
