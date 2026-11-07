use chilen_backend::music_lib::{LibraryItem, is_favorite};
use chilen_widget::virtual_list::VirtualList;
use iced::{Element, Length};
use iced_core::text::IntoFragment;
use iced_m3::{theme::ColorScheme, widget::shapes};
use iced_widget::{center, column, space};

use crate::gui::{
    Chilen,
    event_loop::{Message, library, playback},
    icons,
    main_view::{
        self, TOOLBAR_OFFSET,
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
            model: lib.albums.iter().enumerate(),
            delegate: Box::new(move |(index, album)| {
                let highlighted = highlighted_album_title
                    .map(|t| t == album.title.as_ref())
                    .unwrap_or_default();
                let favorite_item: LibraryItem = album.clone().into();
                let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                AlbumButton {
                    state,
                    album: album.clone(),
                    info: vec![
                        album_button::Info::TrackCount,
                        album_button::Info::ArtistCount,
                    ],
                    play: Message::Playback(playback::Message::PlayAlbum {
                        album: album.clone(),
                        initial_position: 0,
                    }),
                    press: Message::MainView(main_view::Message::TopView(
                        top_view::Message::Navigate(TopView::Album(album.clone())),
                    )),
                    shuffle: Message::Playback(playback::Message::ShuffleAlbum {
                        album: album.clone(),
                        initial_position: None,
                    }),
                    add_to_queue: Message::Playback(playback::Message::AddAlbumToQueue(
                        album.clone(),
                    )),
                    toggle_favorite: Message::Library(library::Message::ToggleFavorite(
                        favorite_item,
                    )),
                    toggle_menu: Message::MainView(main_view::Message::ToggleMenu(index)),
                    menu_opened: state.main_view.button_menu == Some(index),
                    is_favorite,
                    highlighted,
                }
                .into()
            }),
            delegate_height: BUTTON_HEIGHT,
            visibilities: state.main_view.visible.as_deref().unwrap_or(&[]),
            list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
            on_show: Box::new(|i| Message::MainView(main_view::Message::ButtonPoppedIn(i))),
            on_hide: Box::new(|i| Message::MainView(main_view::Message::ButtonPoppedOut(i))),
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
