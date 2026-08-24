use chilen_backend::music_lib::MusicLibrary;
use iced::Element;
use iced_widget::column;

use crate::gui::{
    Chilen,
    main_view::{self, virtualize_entry},
    widget::list::{BUTTON_HEIGHT, BUTTON_SPACING, track_button},
};

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, main_view::Message> {
    let content = column(lib.tracks.iter().enumerate().map(|(index, track)| {
        virtualize_entry(
            state,
            move || {
                track_button::track_button(
                    state,
                    track.clone(),
                    track_button::Info::Artist,
                    track_button::Messages {
                        play: Some(main_view::Message::PlayTrack { track_index: index }),
                        shuffle: None,
                        add_to_queue: None,
                        add_to_playlist: None,
                        details: None,
                        remove: None,
                    },
                )
                .on_press(main_view::Message::PlayTrack { track_index: index })
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
