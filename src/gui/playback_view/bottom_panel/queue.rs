use chilen_backend::music_lib::{LibraryItem, is_favorite};
use chilen_widget::virtual_list::VirtualList;
use iced::Element;
use iced_core::text::IntoFragment;
use iced_m3::{theme::ColorScheme, widget::shapes};
use iced_widget::{center, column, container, stack};

use crate::gui::{
    Chilen, SPACING_SMALL, icons,
    playback_view::Message,
    widget::{
        icon_shape::IconShape,
        list::{
            BUTTON_HEIGHT,
            track_button::{self, TrackButton},
        },
    },
};

pub(super) fn view<'a>(state: &'a Chilen, additional_padding: f32) -> Element<'a, Message> {
    if let Some(player_state) = &state.player_state
        && !player_state.queue_empty()
    {
        let tracks_ordered: Vec<_> = if player_state.shuffle_enabled()
            && player_state.shuffled_track_indices.len() == player_state.tracks.len()
        {
            player_state
                .shuffled_track_indices
                .iter()
                .enumerate()
                .map(|(virtual_index, actual_index)| {
                    (virtual_index, &player_state.tracks[*actual_index])
                })
                .collect()
        } else {
            player_state.tracks.iter().enumerate().collect()
        };

        let track_buttons: Element<'_, Message> = VirtualList {
            model: tracks_ordered,
            delegate: Box::new(|(virtual_index, track)| {
                let favorite_item: LibraryItem = track.clone().into();
                let is_favorite = is_favorite(&favorite_item).unwrap_or_default();
                TrackButton {
                    state,
                    track: track.clone(),
                    is_favorite,
                    messages: track_button::Messages {
                        play: Message::PlayTrack(virtual_index),
                        press: Message::PlayTrack(virtual_index),
                        toggle_favorite: Message::ToggleFavorite(favorite_item),
                        shuffle: None,
                        add_to_queue: None,
                        add_to_playlist: Message::AddTracksToPlaylist(vec![track.hash_self()]),
                        details: None,
                        remove: Some(Message::RemoveFromQueue(virtual_index)),
                    },
                    info: track_button::Info::Artist,
                    status: if virtual_index < player_state.position {
                        track_button::Status::Dimmed
                    } else if virtual_index == player_state.position {
                        track_button::Status::Playing
                    } else {
                        track_button::Status::Idle
                    },
                    vibrant: state.settings.vibrant_widgets,
                }
                .into()
            }),
            delegate_height: BUTTON_HEIGHT,
            visibilities: &state.playback_view.visible_tracks,
            list: Box::new(|buttons| column(buttons).spacing(SPACING_SMALL).into()),
            on_show: Box::new(Message::TrackButtonPoppedIn),
            on_hide: Box::new(Message::TrackButtonPoppedOut),
        }
        .into();

        // FIX: The `stack` works around the state management issue in `scrollable`
        container(stack![iced_widget::scrollable(track_buttons).style(
            |_, status| iced_m3::style::scrollable(status, &state.theme)
        )])
        .padding(additional_padding)
        .into()
    } else {
        center(
            IconShape::new(
                shapes::polymorpher::shapes::cookie4(),
                state.theme.surface_container_high(),
                icons::QUEUE_MUSIC.into_fragment(),
                state.theme.on_surface_variant(),
            )
            .label("The queue is empty", state.theme.on_surface_variant()),
        )
        .into()
    }
}
