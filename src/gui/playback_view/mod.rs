use iced::{Element, Task, padding};
use iced_m3::theme::ColorScheme;
use iced_widget::{column, container, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALL, font, icons,
    widget::cover_image::cover_image,
};

#[derive(Debug, Clone)]
pub enum Message {
    Noop,
}

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let cover_size = 384.0;
    let cover = cover_image(
        state
            .player_state
            .as_ref()
            .and_then(|s| s.current().and_then(|track| track.cover.hires.clone())),
        &icons::MUSIC_NOTE,
        cover_size / 4.0,
        state.theme.on_surface_variant(),
        state.theme.surface_container_high(),
        ROUNDING_LARGE,
    )
    .width(cover_size)
    .height(cover_size);

    // TODO: Should move left and right if it can't fit on screen
    let title = text(if let Some(state) = &state.player_state {
        if let Some(track) = state.current() {
            track.title.clone().unwrap_or("Untitled".to_string())
        } else {
            "Nothing playing".to_string()
        }
    } else {
        "Nothing playing".to_string()
    })
    .size(32.0)
    .wrapping(text::Wrapping::None)
    .color(state.theme.on_surface())
    .font(font::font_bold());

    let artist = state.player_state.as_ref().and_then(|playback_state| {
        playback_state.current().map(|track| {
            text(
                track
                    .artists
                    .clone()
                    .map(|a| a.join(&state.settings.value_separator))
                    .unwrap_or("Unknown artist".to_string()),
            )
        })
    });

    let content = column![title, artist].spacing(SPACING_SMALL);

    let padding = SPACING_SMALL;
    container(column![cover, content].spacing(SPACING_REGULAR))
        .width(cover_size + 2.0 * padding)
        .padding(padding::horizontal(padding).top(padding))
        .into()
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::Noop => {}
    }
    Task::none()
}
