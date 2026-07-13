use iced::border::Radius;
use iced_m3::theme::ColorScheme;

use crate::gui::{ROUNDING_REGULAR, theme::styles::mix_colors};

pub enum Style {
    Primary,
    InversePrimary,
    Secondary,
    InverseSecondary,
    Tertiary,
    InverseTertiary,
}

pub fn button(
    status: iced_widget::button::Status,
    theme: &impl ColorScheme,
    style: Style,
) -> iced_widget::button::Style {
    let (color_regular, color_hovered, color_pressed, content_color) = match style {
        Style::Primary => (
            theme.primary(),
            mix_colors(theme.primary(), theme.surface(), 0.9),
            mix_colors(theme.primary(), theme.surface(), 0.8),
            theme.on_primary(),
        ),
        Style::InversePrimary => (
            theme.on_primary(),
            mix_colors(theme.on_primary(), theme.surface(), 0.9),
            mix_colors(theme.on_primary(), theme.surface(), 0.8),
            theme.primary(),
        ),
        Style::Secondary => (
            theme.secondary(),
            mix_colors(theme.secondary(), theme.surface(), 0.9),
            mix_colors(theme.secondary(), theme.surface(), 0.8),
            theme.on_secondary(),
        ),
        Style::InverseSecondary => (
            theme.on_secondary(),
            mix_colors(theme.on_secondary(), theme.surface(), 0.9),
            mix_colors(theme.on_secondary(), theme.surface(), 0.8),
            theme.secondary(),
        ),
        Style::Tertiary => (
            theme.tertiary(),
            mix_colors(theme.tertiary(), theme.surface(), 0.9),
            mix_colors(theme.tertiary(), theme.surface(), 0.8),
            theme.on_tertiary(),
        ),
        Style::InverseTertiary => (
            theme.on_tertiary(),
            mix_colors(theme.on_tertiary(), theme.surface(), 0.9),
            mix_colors(theme.on_tertiary(), theme.surface(), 0.8),
            theme.tertiary(),
        ),
    };
    iced_widget::button::Style {
        background: Some(iced::Background::Color(match status {
            iced_widget::button::Status::Active => color_regular,
            iced_widget::button::Status::Hovered => color_hovered,
            iced_widget::button::Status::Pressed => color_pressed,
            iced_widget::button::Status::Disabled => color_regular.scale_alpha(0.7),
        })),
        border: iced::Border {
            radius: Radius::from(ROUNDING_REGULAR as f32),
            ..Default::default()
        },
        text_color: content_color,
        ..Default::default()
    }
}
