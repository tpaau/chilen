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
    widget::{
        self,
        list::{BUTTON_HEIGHT, BUTTON_SPACING},
    },
};

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, main_view::Message> {
    let highlighted_genre_name = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Genre { name } = &p.queue_source {
            Some(name)
        } else {
            None
        }
    });
    let content = column(lib.genres.iter().enumerate().map(|(index, genre)| {
        let highlighted = highlighted_genre_name
            .map(|name| *name == genre.name)
            .unwrap_or_default();
        virtualize_entry(
            state,
            move || {
                widget::list::genre_button::genre_button(
                    &state.theme,
                    genre.clone(),
                    main_view::Message::PlayGenre(genre.clone()),
                    main_view::Message::ShuffleGenre(genre.clone()),
                    highlighted,
                )
                .on_press_with(|| {
                    main_view::Message::TopView(top_view::Message::Navigate(TopView::Genre(
                        genre.clone(),
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
