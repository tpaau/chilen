use chilen_backend::music_lib::MusicLibrary;
use iced::Element;
use iced_widget::column;

use crate::gui::{
    Chilen,
    main_view::{
        self,
        top_view::{self, TopView},
        virtualize_entry,
    },
    widget::list::{BUTTON_HEIGHT, BUTTON_SPACING, album_button},
};

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, main_view::Message> {
    let highlighted_album_title = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Album { title } = &p.queue_source {
            Some(title)
        } else {
            None
        }
    });
    let content = column(lib.albums.iter().enumerate().map(|(index, album)| {
        let highlighted = highlighted_album_title
            .map(|t| *t == album.title)
            .unwrap_or_default();
        virtualize_entry(
            state,
            move || {
                album_button::album_button(
                    state,
                    album.clone(),
                    vec![
                        album_button::Info::TrackCount,
                        album_button::Info::ArtistCount,
                    ],
                    main_view::Message::PlayAlbum(album.clone()),
                    main_view::Message::ShuffleAlbum(album.clone()),
                    highlighted,
                )
                .on_press_with(|| {
                    main_view::Message::TopView(top_view::Message::Navigate(TopView::Album(
                        album.clone(),
                    )))
                })
            },
            BUTTON_HEIGHT,
            index,
        )
    }))
    .spacing(BUTTON_SPACING);

    iced_widget::scrollable(content)
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
