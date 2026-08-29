use std::time::Duration;

use chilen_backend::music_lib::{
    Lyrics::{self},
    SyncedLyrics,
};
use iced::Element;
use iced_core::text::LineHeight;
use iced_m3::theme::ColorScheme;
use iced_widget::{center, column, mouse_area, row, scrollable, text};

use crate::gui::{SPACING_REGULAR, font};

const FONT_SIZE: f32 = font::SIZE_LARGER;

// TODO: In case of synced lyrics, the scrollable should follow the currently playing part
fn synced_lyrics<'a, Message: 'a + Clone>(
    theme: &'a impl ColorScheme,
    lyrics: &'a SyncedLyrics,
    player_position: Duration,
    on_segment_pressed: &'a impl Fn(Duration) -> Message,
) -> Element<'a, Message> {
    let current_line = lyrics.current_line(player_position);
    let enhanced_lrc = lyrics.is_enhanced_lrc();
    let lines: Vec<Element<'_, Message>> = lyrics
        .lines
        .iter()
        .map(|l| {
            let current_segment = l.current_segment(player_position);
            let segments: Vec<Element<'_, Message>> = l
                .segments
                .iter()
                .map(|s| {
                    let active = s.is_active(player_position);
                    let current = current_line.map(|current| current == l).unwrap_or_default()
                        && current_segment
                            .map(|current| current == s)
                            .unwrap_or_default();
                    let color = if current && enhanced_lrc {
                        theme.primary()
                    } else if active {
                        theme.on_surface()
                    } else {
                        theme.on_surface_variant().scale_alpha(0.6)
                    };
                    mouse_area(
                        text(&s.content)
                            .size(FONT_SIZE)
                            .color(color)
                            .font(font::font_bold()),
                    )
                    .interaction(iced::mouse::Interaction::Pointer)
                    .on_press(on_segment_pressed(s.timestamp))
                    .into()
                })
                .collect();

            row(segments).wrap().into()
        })
        .collect();

    column(lines).spacing(SPACING_REGULAR).into()
}

pub fn view<'a, Message: 'a + Clone>(
    theme: &'a impl ColorScheme,
    lyrics: &'a Option<chilen_backend::music_lib::Lyrics>,
    player_position: Duration,
    on_segment_pressed: &'a impl Fn(Duration) -> Message,
) -> Element<'a, Message> {
    if let Some(lyrics) = lyrics {
        scrollable(match lyrics {
            Lyrics::Synced(lyrics) => {
                synced_lyrics(theme, lyrics, player_position, on_segment_pressed)
            }
            Lyrics::Unsynced { reason: _, lyrics } => text(lyrics)
                // TODO: Display a dialog if the lyrics parsing error (why the lyrics aren't
                // unsynced) if the parsing failed because the segment timestamp order is not
                // correct. This is maybe something users should be aware of.
                .line_height(LineHeight::Absolute(iced::Pixels(
                    FONT_SIZE + SPACING_REGULAR,
                )))
                .color(theme.on_surface())
                .font(font::font_bold())
                .size(FONT_SIZE)
                .into(),
        })
        .style(|_, status| iced_m3::style::scrollable(status, theme))
        .into()
    } else {
        center(
            text("This track is instrumental")
                .color(theme.on_surface_variant())
                .size(FONT_SIZE),
        )
        .into()
    }
}
