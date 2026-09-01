use std::time::Duration;

use chilen_backend::playback::LoopState;
use iced::{Alignment, Element, Length};
use iced_m3::theme::ColorScheme;
use iced_widget::{column, container, mouse_area, responsive, row, space, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALLER, font,
    formatter::{UNKNOWN_TRACK_DURATION, format_track_duration},
    icons,
    playback_view::Message,
    widget::cover_image::cover_image,
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let cover = container(responsive(|size| {
        let cover_size = size.width;
        cover_image(
            state
                .player_state
                .as_ref()
                .and_then(|s| s.current().and_then(|track| track.cover.hires.clone())),
            &icons::MUSIC_NOTE,
            cover_size / 4.0,
            state.theme.on_surface_variant(),
            state.theme.surface_container_high(),
            ROUNDING_LARGE,
            1.0,
        )
        .width(cover_size)
        .height(cover_size)
        .into()
    }))
    .height(Length::Shrink);

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

    let artist: Option<Element<'_, Message>> =
        state.player_state.as_ref().and_then(|playback_state| {
            playback_state.current().map(|track| {
                if let Some(artists) = &track.artists {
                    let artists: Vec<Element<'_, Message>> = artists
                        .iter()
                        .map(|a| {
                            mouse_area(
                                text(a)
                                    .wrapping(text::Wrapping::None)
                                    .size(font::SIZE_REGULAR)
                                    .color(state.theme.on_surface_variant()),
                            )
                            .on_press(Message::OpenArtist(a.to_string()))
                            .interaction(iced::mouse::Interaction::Pointer)
                            .into()
                        })
                        .collect();

                    if artists.is_empty() {
                        todo!()
                    } else {
                        let mut artists_w_separators = Vec::with_capacity(artists.len() * 2 - 1);
                        let len = artists.len();
                        for (i, artist) in artists.into_iter().enumerate() {
                            artists_w_separators.push(artist);
                            if i < len - 1 {
                                artists_w_separators.push(
                                    text(&state.settings.value_separator)
                                        .wrapping(text::Wrapping::None)
                                        .size(font::SIZE_REGULAR)
                                        .color(state.theme.on_surface_variant())
                                        .into(),
                                );
                            }
                        }

                        // TODO: Should scroll left and right if there's not enough space for the list to fit on the screen.
                        row(artists_w_separators).into()
                    }
                } else {
                    text("Unknown artist").into()
                }
            })
        });

    let album = state.player_state.as_ref().and_then(|playback_state| {
        playback_state.current().map(|track| {
            let widget = mouse_area(
                text(track.album.clone().unwrap_or("Unknown album".to_string()))
                    .wrapping(text::Wrapping::None)
                    .size(font::SIZE_REGULAR)
                    .color(state.theme.on_surface_variant()),
            );

            if let Some(album) = track.album.clone() {
                widget
                    .interaction(iced::mouse::Interaction::Pointer)
                    .on_press(Message::OpenAlbum(album))
            } else {
                widget
            }
        })
    });

    let content = column![title, artist, album];

    let max_value = 1000;

    let (player_position, track_duration) = state
        .player_state
        .as_ref()
        .map(|p| {
            if let Some(track) = p.current() {
                (p.player_position, Some(track.duration))
            } else {
                (p.player_position, None)
            }
        })
        .unwrap_or((Duration::default(), None));

    let value = if let Some(held) = state.playback_view.seek_slider_value {
        (held * max_value as f32) as u32
    } else {
        if let Some(track_duration) = track_duration {
            if track_duration.as_secs_f32() != 0.0 {
                ((player_position.as_secs_f32() / track_duration.as_secs_f32()) * max_value as f32)
                    as u32
            } else {
                0
            }
        } else {
            0
        }
    };

    let slider = iced_m3::widget::slider(
        0..=max_value,
        value,
        move |position| {
            Message::SeekSliderMoved(if max_value != 0 {
                position as f32 / max_value as f32
            } else {
                0.0
            })
        },
        &state.theme,
    )
    .on_release(Message::SeekSliderReleased);

    let player_position = track_duration
        .map(|d| {
            let duration = state
                .playback_view
                .seek_slider_value
                .map(|v| Duration::from_secs_f32(d.as_secs_f32() * v))
                .unwrap_or(player_position);
            format_track_duration(duration)
        })
        .unwrap_or(UNKNOWN_TRACK_DURATION.to_string());
    let (player_pos_color, player_pos_font) = state
        .playback_view
        .seek_slider_value
        .map(|_| (state.theme.on_surface(), font::font_bold()))
        .unwrap_or((state.theme.on_surface_variant(), font::font()));
    let track_duration = track_duration
        .map(format_track_duration)
        .unwrap_or(UNKNOWN_TRACK_DURATION.to_string());

    let timestamps = row![
        text(player_position)
            .size(font::SIZE_SMALL)
            .color(player_pos_color)
            .font(player_pos_font),
        space().width(Length::Fill),
        text(track_duration)
            .size(font::SIZE_SMALL)
            .color(state.theme.on_surface_variant()),
    ];

    let slider_stuff = column![slider, timestamps].spacing(SPACING_SMALLER);

    let play_button_icon = state
        .player_state
        .as_ref()
        .map(|p| match p.playback_state {
            chilen_backend::playback::PlaybackState::Playing => &icons::PAUSE,
            chilen_backend::playback::PlaybackState::Paused => &icons::PLAY_ARROW,
            chilen_backend::playback::PlaybackState::Stopped => &icons::STOP,
        })
        .unwrap_or(&icons::STOP);
    let loop_button_icon = state
        .player_state
        .as_ref()
        .map(|p| match p.loop_state {
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
                        .map(|p| p.loop_state != LoopState::Off)
                        .unwrap_or_default()
                )
                .on_press_maybe(state.player_state.as_ref().map(|_| Message::ToggleLooping)),
        ]
        .align_y(Alignment::Center)
        .spacing(SPACING_SMALLER),
    )
    .center_x(Length::Fill);

    column![cover, content, slider_stuff, buttons]
        .spacing(SPACING_REGULAR)
        .into()
}
