use chilen_backend::music_lib::MusicLibrary;
use iced::Element;
use iced_widget::column;

use crate::gui::{
    Chilen,
    main_view::{self},
    widget::{
        list::{
            BUTTON_HEIGHT, BUTTON_SPACING,
            track_button::{self, TrackButton},
        },
        virtual_list::VirtualList,
    },
};

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, main_view::Message> {
    let highlighted_index = state.player_state.as_ref().and_then(|p| {
        if p.queue_source == chilen_backend::playback::QueueSource::AllTracks {
            p.real_track_index(p.position)
        } else {
            None
        }
    });

    let content = VirtualList {
        model: lib.tracks.iter().enumerate(),
        delegate: Box::new(move |(index, track)| {
            TrackButton {
                state,
                track: track.clone(),
                messages: track_button::Messages {
                    play: main_view::Message::PlayTracksNoShuffle {
                        initial_position: index,
                    },
                    press: main_view::Message::PlayTracks {
                        initial_position: index,
                    },
                    shuffle: Some(main_view::Message::ShuffleTracks {
                        initial_position: index,
                    }),
                    add_to_queue: None,
                    add_to_playlist: None,
                    details: None,
                    remove: None,
                },
                info: track_button::Info::Artist,
                status: if highlighted_index
                    .map(|highlighted| highlighted == index)
                    .unwrap_or_default()
                {
                    track_button::Status::Playing
                } else {
                    track_button::Status::Idle
                },
            }
            .into()
        }),
        delegate_height: BUTTON_HEIGHT,
        visibilities: state.main_view.visible.as_deref().unwrap_or(&[]),
        list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
        on_show: Box::new(main_view::Message::ButtonPoppedIn),
        on_hide: Box::new(main_view::Message::ButtonPoppedOut),
    };

    iced_widget::scrollable(content)
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
