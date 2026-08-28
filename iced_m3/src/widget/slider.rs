use std::ops::RangeInclusive;

use iced::{Background, Border, Color, Element, Length, border::Radius};
use iced_widget::slider::Status;

use crate::theme::ColorScheme;

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum Size {
    #[default]
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    Custom {
        track_height: f32,
        handle_height: f32,
        corner_radius: f32,
    },
}

impl Size {
    pub fn track_height(&self) -> f32 {
        match self {
            Size::ExtraSmall => 16.0,
            Size::Small => 24.0,
            Size::Medium => 40.0,
            Size::Large => 56.0,
            Size::ExtraLarge => 96.0,
            Size::Custom {
                track_height,
                handle_height: _,
                corner_radius: _,
            } => *track_height,
        }
    }

    pub fn handle_height(&self) -> f32 {
        match self {
            Size::ExtraSmall | Size::Small => 44.0,
            Size::Medium => 52.0,
            Size::Large => 68.0,
            Size::ExtraLarge => 108.0,
            Size::Custom {
                track_height: _,
                handle_height,
                corner_radius: _,
            } => *handle_height,
        }
    }

    pub fn corner_radius(&self) -> f32 {
        match self {
            Size::ExtraSmall | Size::Small => 8.0,
            Size::Medium => 12.0,
            Size::Large => 16.0,
            Size::ExtraLarge => 28.0,
            Size::Custom {
                track_height: _,
                handle_height: _,
                corner_radius,
            } => *corner_radius,
        }
    }
}

pub struct Slider<'a, T, Message> {
    range: RangeInclusive<T>,
    step: T,
    shift_step: Option<T>,
    value: T,
    default: Option<T>,
    on_change: Box<dyn Fn(T) -> Message + 'a>,
    on_release: Option<Message>,
    width: Length,
    theme: &'a dyn ColorScheme,
    size: Size,
}

impl<'a, T, Message> Slider<'a, T, Message>
where
    T: Copy + From<u8> + PartialOrd,
    Message: Clone,
{
    pub fn new<F>(
        range: RangeInclusive<T>,
        value: T,
        on_change: F,
        theme: &'a impl ColorScheme,
    ) -> Self
    where
        F: 'a + Fn(T) -> Message,
    {
        let value = if value >= *range.start() {
            value
        } else {
            *range.start()
        };

        let value = if value <= *range.end() {
            value
        } else {
            *range.end()
        };

        Slider {
            value,
            default: None,
            range,
            step: T::from(1),
            shift_step: None,
            on_change: Box::new(on_change),
            on_release: None,
            width: Length::Fill,
            theme,
            size: Size::default(),
        }
    }

    /// Sets the optional default value for the [`Slider`].
    ///
    /// If set, the [`Slider`] will reset to this value when ctrl-clicked or command-clicked.
    pub fn default(mut self, default: impl Into<T>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Sets the release message of the [`Slider`].
    /// This is called when the mouse is released from the slider.
    ///
    /// Typically, the user's interaction with the slider is finished when this message is produced.
    /// This is useful if you need to spawn a long-running task from the slider's result, where
    /// the default on_change message could create too many events.
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    /// Sets the width of the [`Slider`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the step size of the [`Slider`].
    pub fn step(mut self, step: impl Into<T>) -> Self {
        self.step = step.into();
        self
    }

    /// Sets the optional "shift" step for the [`Slider`].
    ///
    /// If set, this value is used as the step while the shift key is pressed.
    pub fn shift_step(mut self, shift_step: impl Into<T>) -> Self {
        self.shift_step = Some(shift_step.into());
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

// TODO: The styling of the iced slider widget is very limited, I'd have to create a fully custom
// widget
pub fn style(
    theme: &(impl ColorScheme + ?Sized),
    status: Status,
    size: Size,
) -> iced_widget::slider::Style {
    let rail = iced_widget::slider::Rail {
        backgrounds: (
            Background::Color(theme.primary()),
            Background::Color(theme.secondary_container()),
        ),
        width: size.track_height(),
        border: Border::default().rounded(size.corner_radius()),
    };

    let handle_width = match status {
        Status::Active | Status::Hovered => 4,
        Status::Dragged => 2,
    };
    let handle = iced_widget::slider::Handle {
        shape: iced_widget::slider::HandleShape::Rectangle {
            width: handle_width,
            border_radius: Radius::from(f32::MAX),
        },
        background: Background::Color(theme.primary()),
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    };

    iced_widget::slider::Style { rail, handle }
}

impl<'a, T, Message> From<Slider<'a, T, Message>> for Element<'a, Message>
where
    T: 'a + Copy + From<u8> + PartialOrd + num_traits::cast::FromPrimitive,
    Message: 'a + Clone,
    f64: std::convert::From<T>,
{
    fn from(value: Slider<'a, T, Message>) -> Self {
        let slider = iced_widget::slider(value.range, value.value, value.on_change)
            .width(value.width)
            .step(value.step)
            .style(move |_, status| style(value.theme, status, value.size))
            .height(value.size.handle_height());

        let slider = match value.default {
            Some(default) => slider.default(default),
            None => slider,
        };

        let slider = match value.on_release {
            Some(message) => slider.on_release(message),
            None => slider,
        };

        let slider = match value.shift_step {
            Some(shift_step) => slider.shift_step(shift_step),
            None => slider,
        };

        slider.into()
    }
}
