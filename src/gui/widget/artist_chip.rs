use std::sync::Arc;

use chilen_backend::music_lib::Artist;
use iced::{Alignment, Border, Color, Length, Padding};
use iced_m3::{
    DISABLED_STATE_LAYER_OPACITY, HOVER_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY,
    theme::ColorScheme,
};
use iced_widget::{Button, button, row, text};

use crate::gui::{SPACING_SMALLER, font, icons, widget::cover_image::cover_image};

pub fn artist_chip<'a, Message: 'a>(
    theme: &'a impl ColorScheme,
    artist: Arc<Artist>,
) -> Button<'a, Message> {
    let thumbnail_size = Length::Fixed(32.0);
    button(
        row![
            cover_image(
                artist.cover.thumbnail.clone(),
                &icons::ARTIST,
                icons::SIZE_SMALLER,
                theme.on_surface_variant(),
                theme.surface_container(),
                f32::MAX
            )
            .width(thumbnail_size)
            .height(thumbnail_size),
            text(artist.name.clone())
                .size(font::SIZE_REGULAR)
                .font(font::font_bold())
        ]
        .spacing(SPACING_SMALLER)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from(SPACING_SMALLER / 2.0).right(SPACING_SMALLER))
    .style(move |_, status| {
        let state_layer_color = theme.on_surface();
        button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Active => Color::TRANSPARENT,
                button::Status::Hovered => state_layer_color.scale_alpha(HOVER_STATE_LAYER_OPACITY),
                button::Status::Pressed => {
                    state_layer_color.scale_alpha(PRESSED_STATE_LAYER_OPACITY)
                }
                button::Status::Disabled => {
                    state_layer_color.scale_alpha(DISABLED_STATE_LAYER_OPACITY)
                }
            })),
            text_color: theme.on_surface(),
            border: Border::default().rounded(f32::MAX),
            ..Default::default()
        }
    })
}
