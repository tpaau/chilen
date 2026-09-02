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
        list::{BUTTON_HEIGHT, BUTTON_SPACING, genre_button::GenreButton},
        virtual_list::VirtualList,
    },
};

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, Message> {
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
                .map(|name| *name == genre.name)
                .unwrap_or_default();
            GenreButton {
                theme: &state.theme,
                genre: genre.clone(),
                play: Message::PlayGenre(genre.clone()),
                press: Message::TopView(top_view::Message::Navigate(TopView::Genre(genre.clone()))),
                shuffle: Message::ShuffleGenre(genre.clone()),
                add_to_queue: Message::AddGenreToQueue(genre.clone()),
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
