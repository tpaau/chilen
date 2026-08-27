use chilen_backend::music_lib::MusicLibrary;
use iced::Element;
use iced_widget::column;

use crate::gui::{
    Chilen,
    main_view::{self, virtualize_entry},
    widget::list::{BUTTON_HEIGHT, BUTTON_SPACING, track_button},
};

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, main_view::Message> {
    let highlighted_index = state.player_state.as_ref().and_then(|p| {
        if p.queue_source == chilen_backend::playback::QueueSource::AllTracks {
            p.real_track_index(p.position)
        } else {
            None
        }
    });
    let content = column(lib.tracks.iter().enumerate().map(|(i, track)| {
        virtualize_entry(
            state,
            move || {
                track_button::track_button(
                    state,
                    track.clone(),
                    track_button::Info::Artist,
                    track_button::Messages {
                        play: main_view::Message::PlayTracksNoShuffle {
                            initial_position: i,
                        },
                        shuffle: main_view::Message::ShuffleTracks {
                            initial_position: i,
                        },
                        add_to_queue: None,
                        add_to_playlist: None,
                        details: None,
                        remove: None,
                    },
                    highlighted_index
                        .map(|index| index == i)
                        .unwrap_or_default(),
                )
                .on_press(main_view::Message::PlayTracks {
                    initial_position: i,
                })
            },
            BUTTON_HEIGHT,
            i,
        )
    }))
    .spacing(BUTTON_SPACING);

    iced_widget::scrollable(content)
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
