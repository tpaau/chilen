use crate::{
    DIM_ALPHA, FOCUS_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY, style::mix_colors,
    theme::ColorScheme,
};
use iced::{Color, border::Radius};

pub enum Button {
    Primary,
    InversePrimary,
    Secondary,
    InverseSecondary,
    Tertiary,
    InverseTertiary,
    Outlined,
    Error,
}

pub fn button(
    status: iced_widget::button::Status,
    theme: &impl ColorScheme,
    style: Button,
) -> iced_widget::button::Style {
    let (color_regular, color_hovered, color_pressed, content_color, border_color) = match style {
        Button::Primary => (
            theme.primary(),
            mix_colors(
                theme.primary(),
                theme.surface(),
                1.0 - FOCUS_STATE_LAYER_OPACITY,
            ),
            mix_colors(
                theme.primary(),
                theme.surface(),
                1.0 - PRESSED_STATE_LAYER_OPACITY,
            ),
            theme.on_primary(),
            None,
        ),
        Button::InversePrimary => (
            theme.on_primary(),
            mix_colors(
                theme.on_primary(),
                theme.surface(),
                1.0 - FOCUS_STATE_LAYER_OPACITY,
            ),
            mix_colors(
                theme.on_primary(),
                theme.surface(),
                1.0 - PRESSED_STATE_LAYER_OPACITY,
            ),
            theme.primary(),
            None,
        ),
        Button::Secondary => (
            theme.secondary(),
            mix_colors(
                theme.secondary(),
                theme.surface(),
                1.0 - FOCUS_STATE_LAYER_OPACITY,
            ),
            mix_colors(
                theme.secondary(),
                theme.surface(),
                1.0 - PRESSED_STATE_LAYER_OPACITY,
            ),
            theme.on_secondary(),
            None,
        ),
        Button::InverseSecondary => (
            theme.on_secondary(),
            mix_colors(
                theme.on_secondary(),
                theme.surface(),
                1.0 - FOCUS_STATE_LAYER_OPACITY,
            ),
            mix_colors(
                theme.on_secondary(),
                theme.surface(),
                1.0 - PRESSED_STATE_LAYER_OPACITY,
            ),
            theme.secondary(),
            None,
        ),
        Button::Tertiary => (
            theme.tertiary(),
            mix_colors(
                theme.tertiary(),
                theme.surface(),
                1.0 - FOCUS_STATE_LAYER_OPACITY,
            ),
            mix_colors(
                theme.tertiary(),
                theme.surface(),
                1.0 - PRESSED_STATE_LAYER_OPACITY,
            ),
            theme.on_tertiary(),
            None,
        ),
        Button::InverseTertiary => (
            theme.on_tertiary(),
            mix_colors(
                theme.on_tertiary(),
                theme.surface(),
                1.0 - FOCUS_STATE_LAYER_OPACITY,
            ),
            mix_colors(
                theme.on_tertiary(),
                theme.surface(),
                1.0 - PRESSED_STATE_LAYER_OPACITY,
            ),
            theme.tertiary(),
            None,
        ),
        Button::Outlined => (
            Color::TRANSPARENT,
            theme
                .on_surface_variant()
                .scale_alpha(FOCUS_STATE_LAYER_OPACITY),
            theme
                .on_surface_variant()
                .scale_alpha(PRESSED_STATE_LAYER_OPACITY),
            theme.on_surface_variant(),
            Some(theme.on_surface_variant()),
        ),
        Button::Error => (
            theme.error(),
            mix_colors(
                theme.error(),
                theme.on_error(),
                1.0 - FOCUS_STATE_LAYER_OPACITY,
            ),
            mix_colors(
                theme.error(),
                theme.on_error(),
                1.0 - PRESSED_STATE_LAYER_OPACITY,
            ),
            theme.on_error(),
            None,
        ),
    };
    iced_widget::button::Style {
        background: Some(iced::Background::Color(match status {
            iced_widget::button::Status::Active => color_regular,
            iced_widget::button::Status::Hovered => color_hovered,
            iced_widget::button::Status::Pressed => color_pressed,
            iced_widget::button::Status::Disabled => color_regular.scale_alpha(DIM_ALPHA),
        })),
        border: iced::Border {
            radius: Radius::from(f32::MAX),
            color: border_color
                .map(|c| {
                    if status == iced_widget::button::Status::Disabled {
                        c.scale_alpha(DIM_ALPHA)
                    } else {
                        c
                    }
                })
                .unwrap_or_default(),
            width: if border_color.is_some() { 1.0 } else { 0.0 },
        },
        text_color: content_color,
        ..Default::default()
    }
}
