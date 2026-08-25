use std::time::Duration;

use chilen_backend::playback::LoopState;
use iced::{Alignment, Element, Length, Task, padding};
use iced_m3::theme::ColorScheme;
use iced_widget::{column, container, row, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALL, SPACING_SMALLER, font, icons,
    widget::cover_image::cover_image,
};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleShuffle,
    Previous,
    TogglePlaying,
    Next,
    ToggleLooping,
    SetPosition(Duration),
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
            .wrapping(text::Wrapping::None)
            .color(state.theme.on_surface_variant())
        })
    });

    let album = state.player_state.as_ref().and_then(|playback_state| {
        playback_state.current().map(|track| {
            text(track.album.clone().unwrap_or("Unknown album".to_string()))
                .wrapping(text::Wrapping::None)
                .color(state.theme.on_surface_variant())
        })
    });

    let content = column![title, artist, album];

    let max_value = 1000;
    let (track_duration, value) = state
        .player_state
        .as_ref()
        .map(|p| {
            if let Some(track) = p.current() {
                if track.duration.as_secs_f32() != 0.0 {
                    (
                        Some(track.duration),
                        ((p.player_position().as_secs_f32() / track.duration.as_secs_f32())
                            * max_value as f32) as u32,
                    )
                } else {
                    (None, 0)
                }
            } else {
                (None, 0)
            }
        })
        .unwrap_or((None, 0));
    let slider = iced_widget::slider(0..=max_value, value, move |position| {
        let position = track_duration
            .map(|d| Duration::from_secs_f32(position as f32 / max_value as f32 * d.as_secs_f32()))
            .unwrap_or(Duration::default());
        Message::SetPosition(position)
    });

    let play_button_icon = state
        .player_state
        .as_ref()
        .map(|p| match p.playback_state() {
            chilen_backend::playback::PlaybackState::Playing => &icons::PAUSE,
            chilen_backend::playback::PlaybackState::Paused => &icons::PLAY_ARROW,
            chilen_backend::playback::PlaybackState::Stopped => &icons::STOP,
        })
        .unwrap_or(&icons::STOP);
    let loop_button_icon = state
        .player_state
        .as_ref()
        .map(|p| match p.loop_state() {
            LoopState::Off | LoopState::Playlist => &icons::REPEAT,
            LoopState::Track => &icons::REPEAT_ONE,
        })
        .unwrap_or(&icons::REPEAT);
    let toggle_size = {
        let size = iced_m3::widget::button::Size::Small;
        size.with_width(size.height()).with_padding(0.0)
    };
    let skip_button_size = {
        let size = iced_m3::widget::button::Size::Medium;
        size.with_width(size.height()).with_padding(0.0)
    };
    let buttons = container(
        row![
            iced_m3::widget::button(&state.theme)
                .size(toggle_size)
                .label_maybe(None)
                .icon_font(icons::filled())
                .icon(&icons::SHUFFLE)
                .style(iced_m3::widget::button::Style::Outlined)
                .selected(
                    state
                        .player_state
                        .as_ref()
                        .map(|p| p.shuffle_enabled())
                        .unwrap_or_default()
                )
                .on_press_maybe(state.player_state.as_ref().map(|_| Message::ToggleShuffle)),
            iced_m3::widget::button(&state.theme)
                .size(skip_button_size)
                .label_maybe(None)
                .icon_font(icons::filled())
                .icon(&icons::SKIP_PREVIOUS)
                .style(iced_m3::widget::button::Style::Tonal(
                    iced_m3::theme::Accent::Tertiary
                ))
                .corner_style(iced_m3::widget::button::CornerStyle::Square)
                .on_press_maybe(
                    state
                        .player_state
                        .as_ref()
                        .and_then(|p| p.can_go_previous().then_some(Message::Previous))
                ),
            iced_m3::widget::button(&state.theme)
                .size(iced_m3::widget::button::Size::Medium)
                .label_maybe(None)
                .icon_font(icons::filled())
                .icon(play_button_icon)
                .style(iced_m3::widget::button::Style::Tonal(
                    iced_m3::theme::Accent::Primary
                ))
                .selected(
                    state
                        .player_state
                        .as_ref()
                        .map(|p| p.is_playing())
                        .unwrap_or_default()
                )
                .on_press_maybe(
                    state
                        .player_state
                        .as_ref()
                        .and_then(|p| p.can_toggle_playing().then_some(Message::TogglePlaying))
                ),
            iced_m3::widget::button(&state.theme)
                .size(skip_button_size)
                .label_maybe(None)
                .icon_font(icons::filled())
                .icon(&icons::SKIP_NEXT)
                .style(iced_m3::widget::button::Style::Tonal(
                    iced_m3::theme::Accent::Tertiary
                ))
                .corner_style(iced_m3::widget::button::CornerStyle::Square)
                .on_press_maybe(
                    state
                        .player_state
                        .as_ref()
                        .and_then(|p| p.can_go_next().then_some(Message::Next))
                ),
            iced_m3::widget::button(&state.theme)
                .size(toggle_size)
                .label_maybe(None)
                .icon_font(icons::filled())
                .icon(loop_button_icon)
                .style(iced_m3::widget::button::Style::Outlined)
                .selected(
                    state
                        .player_state
                        .as_ref()
                        .map(|p| p.loop_state() != LoopState::Off)
                        .unwrap_or_default()
                )
                .on_press_maybe(state.player_state.as_ref().map(|_| Message::ToggleLooping)),
        ]
        .align_y(Alignment::Center)
        .spacing(SPACING_SMALLER),
    )
    .center_x(Length::Fill);

    let padding = SPACING_SMALL;
    container(column![cover, content, slider, buttons].spacing(SPACING_REGULAR))
        .width(cover_size + 2.0 * padding)
        .padding(padding::horizontal(padding).top(padding))
        .into()
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::ToggleShuffle => {
            let _ = chilen_backend::playback::toggle_shuffle_state();
        }
        Message::Previous => {
            let _ = chilen_backend::playback::skip_previous();
        }
        Message::TogglePlaying => {
            let _ = chilen_backend::playback::toggle_playing();
        }
        Message::Next => {
            let _ = chilen_backend::playback::skip_next();
        }
        Message::ToggleLooping => {
            let _ = chilen_backend::playback::cycle_loop_state();
        }
        Message::SetPosition(position) => {
            let _ = chilen_backend::playback::set_player_position(position);
        }
    }
    Task::none()
}
