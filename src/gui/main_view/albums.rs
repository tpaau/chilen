use chilen_backend::music_lib::MusicLibrary;
use iced::Element;
use iced_widget::column;

use crate::gui::{
    Chilen,
    main_view::{
        Message,
        top_view::{self, TopView},
    },
    widget::{
        list::{
            BUTTON_HEIGHT, BUTTON_SPACING,
            album_button::{self, AlbumButton},
        },
        virtual_list::VirtualList,
    },
};

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, Message> {
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
                .map(|t| *t == album.title)
                .unwrap_or_default();
            AlbumButton {
                state,
                album: album.clone(),
                info: vec![
                    album_button::Info::TrackCount,
                    album_button::Info::ArtistCount,
                ],
                play: Message::PlayAlbum(album.clone()),
                press: Message::TopView(top_view::Message::Navigate(TopView::Album(album.clone()))),
                shuffle: Message::ShuffleAlbum(album.clone()),
                add_to_queue: Message::AddAlbumToQueue(album.clone()),
                highlighted,
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
