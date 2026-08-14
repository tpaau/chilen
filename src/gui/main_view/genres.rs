use chilen_backend::music_lib::state::MusicLibrary;
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
    let content = column(lib.genres.iter().enumerate().map(|(index, genre)| {
        virtualize_entry(
            state,
            move || {
                widget::list::genre_button::genre_button(&state.theme, genre.clone()).on_press_with(
                    || {
                        main_view::Message::TopView(top_view::Message::Navigate(TopView::Genre(
                            genre.clone(),
                        )))
                    },
                )
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
