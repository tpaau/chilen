use std::time::Duration;

use chilen_backend::music_lib::{
    Lyrics::{self},
    SyncedLyrics,
};
use iced::{Alignment, Element, Length};
use iced_core::text::LineHeight;
use iced_m3::theme::ColorScheme;
use iced_widget::{center, column, container, mouse_area, row, scrollable, stack, text};

use crate::gui::{ROUNDING_REGULAR, SPACING_REGULAR, SPACING_SMALL, font, icons};

const FONT_SIZE: f32 = font::SIZE_LARGER;

// TODO: In case of synced lyrics, the scrollable should follow the currently playing line
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

    column(lines)
        .width(Length::Fill)
        .spacing(SPACING_REGULAR)
        .into()
}

pub fn view<'a, Message: 'a + Clone>(
    theme: &'a impl ColorScheme,
    lyrics: &'a Option<chilen_backend::music_lib::Lyrics>,
    player_position: Duration,
    on_segment_pressed: &'a impl Fn(Duration) -> Message,
    lyrics_padding: f32,
) -> Element<'a, Message> {
    if let Some(lyrics) = lyrics {
        match lyrics {
            Lyrics::Synced(lyrics) => container(
                scrollable(synced_lyrics(
                    theme,
                    lyrics,
                    player_position,
                    on_segment_pressed,
                ))
                .style(|_, status| iced_m3::style::scrollable(status, theme)),
            )
            .padding(lyrics_padding)
            .into(),
            Lyrics::Unsynced { reason, lyrics } => {
                let lyrics = text(lyrics)
                    .line_height(LineHeight::Absolute(iced::Pixels(
                        FONT_SIZE + SPACING_REGULAR,
                    )))
                    .color(theme.on_surface())
                    .font(font::font_bold())
                    .size(FONT_SIZE);

                let scroll = container(
                    scrollable(lyrics)
                        .style(|_, status| iced_m3::style::scrollable(status, theme))
                        .width(Length::Fill),
                )
                .padding(lyrics_padding)
                .into();

                // TODO: Add an option in app settings to disable showing errors here
                if let chilen_backend::music_lib::LyricsError::Timestamp(e) = reason {
                    let message = format!(
                        "The timestamp order in these lyrics is incorrect. {e}. Showing lyrics without synchronization."
                    );
                    let rounding = ROUNDING_REGULAR;
                    let dialog = container(
                        container(
                            row![
                                text(*icons::ERROR)
                                    .font(icons::outlined())
                                    .size(icons::SIZE_LARGER)
                                    .color(theme.on_error()),
                                text(message).size(font::SIZE_SMALL).color(theme.on_error()),
                            ]
                            .align_y(Alignment::Center)
                            .spacing(rounding),
                        )
                        .padding(SPACING_SMALL)
                        .width(Length::Fill)
                        .style(move |_| {
                            iced_widget::container::Style::default()
                                .background(theme.error())
                                .border(iced::Border::default().rounded(rounding))
                                .shadow(iced_m3::style::shadow(theme.shadow(), 0.4))
                        }),
                    )
                    .align_bottom(Length::Fill);
                    stack![scroll, dialog].into()
                } else {
                    scroll
                }
            }
        }
    } else {
        center(
            text("This is an instrumental")
                .color(theme.on_surface_variant())
                .size(FONT_SIZE),
        )
        .into()
    }
}
