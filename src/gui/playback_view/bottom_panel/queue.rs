use iced::Element;
use iced_m3::theme::ColorScheme;
use iced_widget::{column, container, sensor, space, stack, text};

use crate::gui::{
    Chilen, SPACING_SMALL, font,
    playback_view::Message,
    widget::list::{
        BUTTON_HEIGHT,
        track_button::{self, track_button},
    },
};

pub(super) fn view<'a>(state: &'a Chilen, additional_padding: f32) -> Element<'a, Message> {
    let player_state = match &state.player_state {
        Some(p) => p,
        None => {
            return text("Loading...")
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_surface())
                .into();
        }
    };

    let tracks_ordered: Vec<_> = if player_state.shuffle_enabled()
        && player_state.shuffled_track_indices.len() == player_state.tracks.len()
    {
        player_state
            .shuffled_track_indices
            .iter()
            .enumerate()
            .map(|(virtual_index, actual_index)| {
                (virtual_index, player_state.tracks[*actual_index].clone())
            })
            .collect()
    } else {
        player_state
            .tracks
            .iter()
            .enumerate()
            .map(|(i, t)| (i, t.clone()))
            .collect()
    };

    let track_buttons: Vec<_> = tracks_ordered
        .into_iter()
        .map(|(virtual_index, track)| {
            let content: iced::Element<'_, Message> = match state
                .playback_view
                .visible_tracks
                .get(virtual_index)
                .cloned()
                .unwrap_or_default()
            {
                true => track_button(
                    state,
                    track,
                    track_button::Info::Artist,
                    track_button::Messages {
                        play: Message::PlayTrack(virtual_index),
                        shuffle: None,
                        add_to_queue: None,
                        add_to_playlist: None,
                        details: None,
                        remove: None,
                    },
                    virtual_index == player_state.position,
                )
                .on_press(Message::PlayTrack(virtual_index))
                .into(),
                false => space().height(BUTTON_HEIGHT).into(),
            };

            sensor(content)
                .on_show(move |_| Message::TrackButtonPoppedIn(virtual_index))
                .on_hide(Message::TrackButtonPoppedOut(virtual_index))
                .into()
        })
        .collect();

    // FIX: The `stack` works around the state management issue in `scrollable`
    container(stack![
        iced_widget::scrollable(column(track_buttons).spacing(SPACING_SMALL))
            .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
    ])
    .padding(additional_padding)
    .into()
}
