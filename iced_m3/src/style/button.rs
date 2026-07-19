use crate::{style::mix_colors, theme::ColorScheme};
use iced::border::Radius;

pub enum Button {
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
    style: Button,
) -> iced_widget::button::Style {
    let (color_regular, color_hovered, color_pressed, content_color) = match style {
        Button::Primary => (
            theme.primary(),
            mix_colors(theme.primary(), theme.surface(), 0.9),
            mix_colors(theme.primary(), theme.surface(), 0.8),
            theme.on_primary(),
        ),
        Button::InversePrimary => (
            theme.on_primary(),
            mix_colors(theme.on_primary(), theme.surface(), 0.9),
            mix_colors(theme.on_primary(), theme.surface(), 0.8),
            theme.primary(),
        ),
        Button::Secondary => (
            theme.secondary(),
            mix_colors(theme.secondary(), theme.surface(), 0.9),
            mix_colors(theme.secondary(), theme.surface(), 0.8),
            theme.on_secondary(),
        ),
        Button::InverseSecondary => (
            theme.on_secondary(),
            mix_colors(theme.on_secondary(), theme.surface(), 0.9),
            mix_colors(theme.on_secondary(), theme.surface(), 0.8),
            theme.secondary(),
        ),
        Button::Tertiary => (
            theme.tertiary(),
            mix_colors(theme.tertiary(), theme.surface(), 0.9),
            mix_colors(theme.tertiary(), theme.surface(), 0.8),
            theme.on_tertiary(),
        ),
        Button::InverseTertiary => (
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
            radius: Radius::from(f32::MAX),
            ..Default::default()
        },
        text_color: content_color,
        ..Default::default()
    }
}
