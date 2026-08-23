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
    let content = column(lib.albums.iter().enumerate().map(|(index, album)| {
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
