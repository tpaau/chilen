use chilen_widget::virtual_list::VirtualList;
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
        list::{
            BUTTON_HEIGHT, BUTTON_SPACING,
            album_button::{self, AlbumButton},
        },
    },
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    if let Some(lib) = &state.library
        && !lib.albums.is_empty()
    {
        let highlighted_album_title = state.player_state.as_ref().and_then(|p| {
            if let chilen_backend::playback::QueueSource::Album { title } = &p.queue_source {
                Some(title)
            } else {
                None
            }
        });
        let content = VirtualList {
            model: lib.albums.iter(),
            delegate: Box::new(move |album| {
                let highlighted = highlighted_album_title
                    .map(|t| t == album.title.as_ref())
                    .unwrap_or_default();
                AlbumButton {
                    state,
                    album: album.clone(),
                    info: vec![
                        album_button::Info::TrackCount,
                        album_button::Info::ArtistCount,
                    ],
                    play: Message::PlayAlbum(album.clone()),
                    press: Message::TopView(top_view::Message::Navigate(TopView::Album(
                        album.clone(),
                    ))),
                    shuffle: Message::ShuffleAlbum(album.clone()),
                    add_to_queue: Message::AddAlbumToQueue(album.clone()),
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
    } else {
        center(
            IconShape::new(
                shapes::polymorpher::shapes::sunny(),
                state.theme.surface_container_high(),
                icons::ALBUM.into_fragment(),
                state.theme.on_surface_variant(),
            )
            .label(
                "Your albums will show up here",
                state.theme.on_surface_variant(),
            ),
        )
        .into()
    }
}
