use chilen_backend::music_lib::{LibraryItem, is_favorite};
use chilen_widget::virtual_list::VirtualList;
use iced::{Element, Length};
use iced_core::text::IntoFragment;
use iced_m3::theme::ColorScheme;
use iced_widget::{center, column, space};

use crate::gui::{
    Chilen, dialog,
    event_loop::{Message, library, playback},
    icons,
    main_view::{self, NavTab, TOOLBAR_OFFSET},
    widget::{
        icon_shape::IconShape,
        list::{
            BUTTON_HEIGHT, BUTTON_SPACING,
            track_button::{self, TrackButton},
        },
    },
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
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
                        play: Message::Playback(playback::Message::PlayAllTracksNoShuffle {
                            initial_position: index,
                        }),
                        press: Message::Playback(playback::Message::PlayAllTracks {
                            initial_position: index,
                        }),
                        toggle_favorite: Message::Library(library::Message::ToggleFavorite(
                            favorite_item,
                        )),
                        shuffle: Some(Message::Playback(playback::Message::ShuffleAllTracks {
                            initial_position: Some(index),
                        })),
                        add_to_queue: Some(Message::Playback(playback::Message::AddTrackToQueue(
                            track.clone(),
                        ))),
                        add_to_playlist: Message::Dialog(dialog::Message::AddTracksToPlaylist(
                            vec![track.clone()],
                        )),
                        details: None,
                        remove: None,
                        toggle_menu: Message::MainView(main_view::Message::ToggleMenu(index)),
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
                    menu_opened: state.main_view.button_menu == Some(index),
                }
                .into()
            }),
            delegate_height: BUTTON_HEIGHT,
            visibilities: state.main_view.visible.as_deref().unwrap_or(&[]),
            list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
            on_show: Box::new(|i| Message::MainView(main_view::Message::ButtonPoppedIn(i))),
            on_hide: Box::new(|i| Message::MainView(main_view::Message::ButtonPoppedOut(i))),
        };

        iced_widget::scrollable(column![content, space().height(*TOOLBAR_OFFSET)])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
            .into()
    } else {
        center(
            IconShape::new(
                NavTab::Tracks.shape(),
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
