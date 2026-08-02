use iced::{Color, Element, Padding, border::Radius, padding};

use crate::theme::ColorScheme;

pub enum Style {
    Primary,
    InversePrimary,
    PrimaryContainer,
    Secondary,
    InverseSecondary,
    SecondaryContainer,
    Tertiary,
    InverseTertiary,
    TertiaryContainer,
    Text { content: Color },
    Outlined,
}

impl Style {
    // surface, content, outline, state layer
    fn colors(&self, theme: &impl ColorScheme) -> (Option<Color>, Color, Color) {
        match self {
            Style::Primary => todo!(),
            Style::InversePrimary => todo!(),
            Style::PrimaryContainer => todo!(),
            Style::Secondary => todo!(),
            Style::InverseSecondary => todo!(),
            Style::SecondaryContainer => todo!(),
            Style::Tertiary => todo!(),
            Style::InverseTertiary => todo!(),
            Style::TertiaryContainer => todo!(),
            Style::Text { content } => (None, *content, Color::TRANSPARENT),
            Style::Outlined => (None, theme.on_surface_variant(), theme.outline_variant()),
        }
    }
}

pub enum Size {
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl Size {
    fn height(&self) -> f32 {
        match self {
            Size::ExtraSmall => 32.0,
            Size::Small => 40.0,
            Size::Medium => 56.0,
            Size::Large => 96.0,
            Size::ExtraLarge => 136.0,
        }
    }

    fn padding(&self) -> Padding {
        padding::horizontal(match self {
            Size::ExtraSmall => 12.0,
            Size::Small => 16.0,
            Size::Medium => 24.0,
            Size::Large => 48.0,
            Size::ExtraLarge => 64.0,
        })
    }
}

pub enum CornerStyle {
    Round,
    Square,
    Custom { regular: Radius, pressed: Radius },
}

impl CornerStyle {
    fn regular(&self, size: &Size) -> Radius {
        match self {
            CornerStyle::Round => Radius::new(f32::MAX),
            CornerStyle::Square => match size {
                Size::ExtraSmall | Size::Small => Radius::new(12.0),
                Size::Medium => Radius::new(16.0),
                Size::Large | Size::ExtraLarge => Radius::new(28.0),
            },
            CornerStyle::Custom {
                regular,
                pressed: _,
            } => *regular,
        }
    }

    fn pressed(&self, size: &Size) -> Radius {
        match self {
            CornerStyle::Round | CornerStyle::Square => match size {
                Size::ExtraSmall | Size::Small => Radius::new(8.0),
                Size::Medium => Radius::new(12.0),
                Size::Large | Size::ExtraLarge => Radius::new(16.0),
            },
            CornerStyle::Custom {
                regular: _,
                pressed,
            } => *pressed,
        }
    }
}

pub fn style(
    status: iced_widget::button::Status,
    button_style: Style,
    corner_style: CornerStyle,
) -> iced_widget::button::Style {
    todo!()
}

pub struct Button<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<&'a Message>,
}

impl<'a, Message> Button<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            on_press: None,
        }
    }
}
