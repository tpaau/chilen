use chilen_backend::music_lib::{LibraryItem, is_favorite};
use chilen_widget::virtual_list::VirtualList;
use iced::{Element, Length};
use iced_core::text::IntoFragment;
use iced_m3::{theme::ColorScheme, widget::shapes};
use iced_widget::{center, column};

use crate::gui::{
    Chilen, icons,
    main_view::{self},
    widget::{
        icon_shape::IconShape,
        list::{
            BUTTON_HEIGHT, BUTTON_SPACING,
            track_button::{self, TrackButton},
        },
    },
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, main_view::Message> {
    if let Some(lib) = &state.library
        && !lib.tracks.is_empty()
    {
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
                let favorite_item: LibraryItem = track.clone().into();
                let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                TrackButton {
                    state,
                    track: track.clone(),
                    is_favorite,
                    messages: track_button::Messages {
                        play: main_view::Message::PlayTracksNoShuffle {
                            initial_position: index,
                        },
                        press: main_view::Message::PlayTracks {
                            initial_position: index,
                        },
                        toggle_favorite: main_view::Message::ToggleFavorite(favorite_item),
                        shuffle: Some(main_view::Message::ShuffleTracks {
                            initial_position: Some(index),
                        }),
                        add_to_queue: Some(main_view::Message::AddTrackToQueue(track.clone())),
                        add_to_playlist: main_view::Message::AddTrackToPlaylist(track.clone()),
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
                    vibrant: state.settings.vibrant_widgets,
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
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
            .into()
    } else {
        center(
            IconShape::new(
                shapes::polymorpher::shapes::sunny(),
                state.theme.surface_container_high(),
                icons::MUSIC_NOTE.into_fragment(),
                state.theme.on_surface_variant(),
            )
            .label(
                "Your tracks will show up here",
                state.theme.on_surface_variant(),
            ),
        )
        .into()
    }
}
