use iced::{Border, Color, Length};
use iced_m3::{
    DISABLED_STATE_LAYER_OPACITY, HOVER_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY,
};

use crate::gui::{ROUNDING_LARGE, SPACING_SMALLER, THUMBNAIL_SIZE};

pub mod track_button;

pub const BUTTON_ROUNDING: f32 = ROUNDING_LARGE;
pub const BUTTON_PADDING: f32 = SPACING_SMALLER;
pub const BUTTON_HEIGHT: Length = Length::Fixed(THUMBNAIL_SIZE + 2.0 * BUTTON_PADDING);
pub const BUTTON_SPACING: f32 = SPACING_SMALLER;

pub fn button_style(
    status: iced_widget::button::Status,
    color: Color,
) -> iced_widget::button::Style {
    iced_widget::button::Style {
        background: Some(iced::Background::Color(match status {
            iced_widget::button::Status::Active => Color::TRANSPARENT,
            iced_widget::button::Status::Hovered => color.scale_alpha(HOVER_STATE_LAYER_OPACITY),
            iced_widget::button::Status::Pressed => color.scale_alpha(PRESSED_STATE_LAYER_OPACITY),
            iced_widget::button::Status::Disabled => {
                color.scale_alpha(DISABLED_STATE_LAYER_OPACITY)
            }
        })),
        text_color: color,
        border: Border::default().rounded(BUTTON_ROUNDING),
        ..Default::default()
    }
}
