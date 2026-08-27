use iced::Element;
use iced_m3::theme::ColorScheme;
use iced_widget::text;

use crate::gui::{Chilen, font, playback_view::Message};

pub(super) fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    if let Some(player_state) = &state.player_state
        && let Some(track) = player_state.current()
    {
        crate::gui::widget::lyrics::view(
            &state.theme,
            &track.lyrics,
            player_state.player_position,
            &|position| Message::SetPlayerPosition(position),
        )
    } else {
        text("Nothing is playing")
            .size(font::SIZE_REGULAR)
            .color(state.theme.on_surface())
            .into()
    }
}
