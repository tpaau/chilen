use chilen_backend::music_lib::MusicLibrary;
use iced::Element;
use iced_widget::column;

use crate::gui::{
    Chilen,
    main_view::{self, virtualize_entry},
    widget::{
        self,
        list::{BUTTON_HEIGHT, BUTTON_SPACING},
    },
};

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, main_view::Message> {
    let content = column(lib.tracks.iter().enumerate().map(|(index, track)| {
        virtualize_entry(
            state,
            move || {
                widget::list::track_button::track_button(state, track.clone())
                    .on_press(main_view::Message::Noop)
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
