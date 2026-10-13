use iced::Element;
use iced_core::text::IntoFragment;
use iced_m3::{theme::ColorScheme, widget::shapes};
use iced_widget::center;

use crate::gui::{Chilen, icons, playback_view::Message, widget::icon_shape::IconShape};

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
            IconShape::new(
                shapes::polymorpher::shapes::oval(),
                state.theme.surface_container_high(),
                icons::LYRICS.into_fragment(),
                state.theme.on_surface_variant(),
            )
            .label("Nothing is playing", state.theme.on_surface_variant()),
        )
        .into()
    }
}
