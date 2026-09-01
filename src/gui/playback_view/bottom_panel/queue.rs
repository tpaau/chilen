use iced::Element;
use iced_m3::theme::ColorScheme;
use iced_widget::{center, column, container, stack, text};

use crate::gui::{
    Chilen, SPACING_SMALL,
    playback_view::Message,
    widget::list::{
        BUTTON_HEIGHT,
        track_button::{self, TrackButton},
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

        let track_buttons: Element<'_, Message> = crate::gui::widget::virtual_list::VirtualList {
            model: tracks_ordered,
            delegate: Box::new(|(virtual_index, track)| {
                TrackButton {
                    state,
                    track: track.clone(),
                    messages: track_button::Messages {
                        play: Message::PlayTrack(virtual_index),
                        press: Message::PlayTrack(virtual_index),
                        shuffle: None,
                        add_to_queue: None,
                        add_to_playlist: None,
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
            text("The queue is empty")
                .color(state.theme.on_surface_variant())
                .size(crate::gui::widget::lyrics::FONT_SIZE),
        )
        .into()
    }
}
