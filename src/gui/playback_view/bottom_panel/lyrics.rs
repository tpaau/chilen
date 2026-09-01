use iced::Element;
use iced_m3::theme::ColorScheme;
use iced_widget::{center, text};

use crate::gui::{Chilen, playback_view::Message};

pub(super) fn view<'a>(state: &'a Chilen, lyrics_padding: f32) -> Element<'a, Message> {
    if let Some(player_state) = &state.player_state
        && let Some(track) = player_state.current()
    {
        crate::gui::widget::lyrics::view(
            &state.theme,
            &track.lyrics,
            player_state.player_position,
            &|position| Message::SetPlayerPosition(position),
            lyrics_padding,
            state.settings.show_lyrics_errors,
        )
    } else {
        center(
            text("Nothing is playing")
                .color(state.theme.on_surface_variant())
                .size(crate::gui::widget::lyrics::FONT_SIZE),
        )
        .into()
    }
}
