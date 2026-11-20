pub mod album_button;
pub mod artist_button;
pub mod genre_button;
pub mod playlist_button;
pub mod track_button;

use iced::{Border, Color};
use iced_m3::{
    style::{DISABLED_STATE_LAYER_OPACITY, Elevation, StateLayer},
    theme::ColorScheme,
    widget::card,
};

use crate::gui::{ROUNDING_LARGE, SPACING_SMALLER};

pub const THUMBNAIL_SIZE: f32 = 64.0;
pub const BUTTON_ROUNDING: f32 = ROUNDING_LARGE;
pub const BUTTON_PADDING: f32 = SPACING_SMALLER;
pub const BUTTON_HEIGHT: f32 = THUMBNAIL_SIZE + 2.0 * BUTTON_PADDING;
pub const BUTTON_SPACING: f32 = SPACING_SMALLER;

pub fn card_style(theme: &(impl ColorScheme + ?Sized)) -> card::Style {
    card::Style {
        container_color: Color::TRANSPARENT,
        border: Border::default().rounded(BUTTON_ROUNDING),
        state_layer: StateLayer::new(theme.on_surface()),
        shadow_color: theme.shadow(),
        elevation: Elevation::default(),
        container_color_disabled: theme.on_surface().scale_alpha(DISABLED_STATE_LAYER_OPACITY),
    }
}
