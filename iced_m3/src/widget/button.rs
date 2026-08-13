use iced::{Alignment, Color, Element, Length, Padding, border::Radius, padding};
use iced_widget::{center, row, text};

use crate::{
    DISABLED_STATE_LAYER_OPACITY, HOVER_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY,
    style::mix_colors,
    theme::{Accent, ColorScheme},
};

const DISABLED_CONTAINER_OPACITY: f32 = 0.1;
const DISABLED_LABEL_OPACITY: f32 = DISABLED_STATE_LAYER_OPACITY;

#[derive(Clone, Copy)]
pub enum Style {
    Elevated(Accent),
    Filled(Accent),
    Tonal(Accent),
    Outlined,
    Text(Accent),
    Custom {
        surface: Option<Color>,
        content: Color,
        outline: Option<Color>,
        surface_disabled: Option<Color>,
        content_disabled: Color,
        outline_disabled: Option<Color>,
    },
}

impl Default for Style {
    fn default() -> Self {
        Self::Filled(Accent::default())
    }
}

impl Style {
    // surface, content, outline
    fn colors(
        &self,
        status: iced_widget::button::Status,
        selected: Option<bool>,
        theme: &(impl ColorScheme + ?Sized),
    ) -> (Color, Color, Option<Color>) {
        let state_layer_alpha = match status {
            iced_widget::button::Status::Active => 0.0,
            iced_widget::button::Status::Hovered => HOVER_STATE_LAYER_OPACITY,
            iced_widget::button::Status::Pressed => PRESSED_STATE_LAYER_OPACITY,
            iced_widget::button::Status::Disabled => 0.0,
        };

        match self {
            Style::Elevated(accent) => {
                if status == iced_widget::button::Status::Disabled {
                    (
                        theme.on_surface().scale_alpha(DISABLED_CONTAINER_OPACITY),
                        theme.on_surface().scale_alpha(DISABLED_LABEL_OPACITY),
                        None,
                    )
                } else {
                    let content = match selected.unwrap_or(false) {
                        true => match accent {
                            Accent::Primary => theme.on_primary(),
                            Accent::Secondary => theme.on_secondary(),
                            Accent::Tertiary => theme.on_tertiary(),
                        },
                        false => match accent {
                            Accent::Primary => theme.primary(),
                            Accent::Secondary => theme.secondary(),
                            Accent::Tertiary => theme.tertiary(),
                        },
                    };
                    let surface = match selected.unwrap_or(false) {
                        true => match accent {
                            Accent::Primary => theme.primary(),
                            Accent::Secondary => theme.secondary(),
                            Accent::Tertiary => theme.tertiary(),
                        },
                        false => theme.surface_container_low(),
                    };
                    let surface = mix_colors(surface, content, state_layer_alpha);
                    (surface, content, None)
                }
            }
            Style::Filled(accent) => {
                if status == iced_widget::button::Status::Disabled {
                    (
                        theme.on_surface().scale_alpha(DISABLED_CONTAINER_OPACITY),
                        theme.on_surface().scale_alpha(DISABLED_LABEL_OPACITY),
                        None,
                    )
                } else {
                    let content = match selected.unwrap_or(true) {
                        true => match accent {
                            Accent::Primary => theme.on_primary(),
                            Accent::Secondary => theme.on_secondary(),
                            Accent::Tertiary => theme.on_tertiary(),
                        },
                        false => theme.on_surface_variant(),
                    };
                    let surface = match selected.unwrap_or(true) {
                        true => match accent {
                            Accent::Primary => theme.primary(),
                            Accent::Secondary => theme.secondary(),
                            Accent::Tertiary => theme.tertiary(),
                        },
                        false => theme.surface_container(),
                    };
                    let surface = mix_colors(surface, content, state_layer_alpha);
                    (surface, content, None)
                }
            }
            Style::Tonal(accent) => {
                if status == iced_widget::button::Status::Disabled {
                    (
                        theme.on_surface().scale_alpha(DISABLED_CONTAINER_OPACITY),
                        theme.on_surface().scale_alpha(DISABLED_LABEL_OPACITY),
                        None,
                    )
                } else {
                    let content = match selected.unwrap_or(false) {
                        true => match accent {
                            Accent::Primary => theme.on_primary(),
                            Accent::Secondary => theme.on_secondary(),
                            Accent::Tertiary => theme.on_tertiary(),
                        },
                        false => match accent {
                            Accent::Primary => theme.on_primary_container(),
                            Accent::Secondary => theme.on_secondary_container(),
                            Accent::Tertiary => theme.on_tertiary_container(),
                        },
                    };
                    let surface = match selected.unwrap_or(false) {
                        true => match accent {
                            Accent::Primary => theme.primary(),
                            Accent::Secondary => theme.secondary(),
                            Accent::Tertiary => theme.tertiary(),
                        },
                        false => match accent {
                            Accent::Primary => theme.primary_container(),
                            Accent::Secondary => theme.secondary_container(),
                            Accent::Tertiary => theme.tertiary_container(),
                        },
                    };
                    let surface = mix_colors(surface, content, state_layer_alpha);
                    (surface, content, None)
                }
            }
            Style::Outlined => {
                if status == iced_widget::button::Status::Disabled {
                    (
                        theme.on_surface().scale_alpha(DISABLED_CONTAINER_OPACITY),
                        theme.on_surface().scale_alpha(DISABLED_LABEL_OPACITY),
                        Some(theme.outline_variant()),
                    )
                } else {
                    let content = match selected.unwrap_or(false) {
                        true => theme.inverse_on_surface(),
                        false => theme.on_surface_variant(),
                    };
                    let surface = match selected.unwrap_or(false) {
                        true => mix_colors(theme.inverse_surface(), content, state_layer_alpha),
                        false => content.scale_alpha(state_layer_alpha),
                    };
                    let outline = match selected.unwrap_or(false) {
                        true => None,
                        false => Some(theme.outline_variant()),
                    };
                    (surface, content, outline)
                }
            }
            Style::Text(accent) => {
                if status == iced_widget::button::Status::Disabled {
                    (
                        theme.on_surface().scale_alpha(DISABLED_CONTAINER_OPACITY),
                        theme.on_surface().scale_alpha(DISABLED_LABEL_OPACITY),
                        None,
                    )
                } else {
                    let content = match accent {
                        Accent::Primary => theme.primary(),
                        Accent::Secondary => theme.secondary(),
                        Accent::Tertiary => theme.tertiary(),
                    };
                    let surface = content.scale_alpha(state_layer_alpha);
                    (surface, content, None)
                }
            }
            Style::Custom {
                surface,
                content,
                outline,
                surface_disabled,
                content_disabled,
                outline_disabled,
            } => {
                if status == iced_widget::button::Status::Disabled {
                    (
                        surface_disabled.unwrap_or(Color::TRANSPARENT),
                        *content_disabled,
                        *outline_disabled,
                    )
                } else {
                    let surface = match surface {
                        Some(surface) => mix_colors(*surface, *content, state_layer_alpha),
                        None => content.scale_alpha(state_layer_alpha),
                    };
                    (surface, *content, *outline)
                }
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum Size {
    ExtraSmall,
    #[default]
    Small,
    Medium,
    Large,
    ExtraLarge,
    Custom {
        height: Length,
        width: Length,
        spacing: f32,
        padding: f32,
        icon_size: f32,
        font_size: f32,
    },
}

impl Size {
    fn height(&self) -> Length {
        match self {
            Self::ExtraSmall => Length::Fixed(32.0),
            Self::Small => Length::Fixed(40.0),
            Self::Medium => Length::Fixed(56.0),
            Self::Large => Length::Fixed(96.0),
            Self::ExtraLarge => Length::Fixed(136.0),
            Self::Custom {
                height,
                padding: _,
                width: _,
                spacing: _,
                icon_size: _,
                font_size: _,
            } => *height,
        }
    }

    fn width(&self) -> Length {
        match self {
            Self::Custom {
                height: _,
                width,
                spacing: _,
                padding: _,
                icon_size: _,
                font_size: _,
            } => *width,
            _ => Length::Shrink,
        }
    }

    fn padding(&self) -> Padding {
        padding::horizontal(match self {
            Self::ExtraSmall => 12.0,
            Self::Small => 16.0,
            Self::Medium => 24.0,
            Self::Large => 48.0,
            Self::ExtraLarge => 64.0,
            Self::Custom {
                height: _,
                padding,
                width: _,
                spacing: _,
                icon_size: _,
                font_size: _,
            } => *padding,
        })
    }

    fn spacing(&self) -> f32 {
        match self {
            Size::ExtraSmall => 4.0,
            Size::Small | Size::Medium => 8.0,
            Size::Large => 12.0,
            Size::ExtraLarge => 16.0,
            Size::Custom {
                height: _,
                width: _,
                padding: _,
                spacing,
                icon_size: _,
                font_size: _,
            } => *spacing,
        }
    }

    fn icon_size(&self) -> f32 {
        match self {
            Size::ExtraSmall => 20.0,
            Size::Small => 20.0,
            Size::Medium => 24.0,
            Size::Large => 32.0,
            Size::ExtraLarge => 40.0,
            Size::Custom {
                height: _,
                width: _,
                spacing: _,
                padding: _,
                icon_size,
                font_size: _,
            } => *icon_size,
        }
    }

    // TODO: Implement the typography system and get the sizes from there
    fn font_size(&self) -> f32 {
        match self {
            Size::ExtraSmall => 14.0,
            Size::Medium | Size::Small => 16.0,
            Size::Large | Size::ExtraLarge => 22.0,
            Size::Custom {
                height: _,
                width: _,
                spacing: _,
                padding: _,
                icon_size: _,
                font_size,
            } => *font_size,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum CornerStyle {
    #[default]
    Round,
    Square,
    Custom {
        regular: Radius,
        pressed: Radius,
    },
}

impl CornerStyle {
    fn regular(&self, size: &Size) -> Radius {
        match self {
            CornerStyle::Round => Radius::new(f32::MAX),
            CornerStyle::Square => match size {
                Size::ExtraSmall | Size::Small => Radius::new(12.0),
                Size::Medium
                | Size::Custom {
                    height: _,
                    padding: _,
                    width: _,
                    spacing: _,
                    icon_size: _,
                    font_size: _,
                } => Radius::new(16.0),
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
                Size::Medium
                | Size::Custom {
                    height: _,
                    padding: _,
                    width: _,
                    spacing: _,
                    icon_size: _,
                    font_size: _,
                } => Radius::new(12.0),
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
    selected: Option<bool>,
    button_size: Size,
    button_style: Style,
    corner_style: CornerStyle,
    theme: &(impl ColorScheme + ?Sized),
) -> iced_widget::button::Style {
    let (surface, content, outline) = button_style.colors(status, selected, theme);
    let corner_radius = if status == iced_widget::button::Status::Pressed {
        corner_style.pressed(&button_size)
    } else {
        corner_style.regular(&button_size)
    };
    let border = iced::Border {
        radius: corner_radius,
        color: outline.unwrap_or(Color::TRANSPARENT),
        width: if outline.is_some() { 1.0 } else { 0.0 },
    };

    iced_widget::button::Style {
        background: Some(iced::Background::Color(surface)),
        text_color: content,
        border,
        shadow: iced::Shadow::default(),
        snap: true,
    }
}

enum OnPress<'a, Message> {
    Direct(Message),
    Closure(Box<dyn Fn() -> Message + 'a>),
}

pub struct Button<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: 'a + iced_widget::core::text::Renderer,
{
    on_press: Option<OnPress<'a, Message>>,
    clip: bool,
    theme: &'a dyn ColorScheme,
    label: Option<&'a str>,
    label_font: Option<Renderer::Font>,
    icon: Option<&'a char>,
    icon_font: Option<Renderer::Font>,
    size: Size,
    corner_style: CornerStyle,
    style: Style,
    selected: Option<bool>,
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer>
where
    Renderer: iced::advanced::text::Renderer,
{
    #[must_use]
    pub fn new(theme: &'a dyn ColorScheme) -> Self {
        Self {
            on_press: None,
            clip: false,
            theme,
            label: Some("label"),
            label_font: None,
            icon: None,
            icon_font: None,
            size: Size::default(),
            corner_style: CornerStyle::default(),
            style: Style::default(),
            selected: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    #[must_use]
    pub fn label_maybe(mut self, maybe_label: Option<&'a str>) -> Self {
        self.label = maybe_label;
        self
    }

    #[must_use]
    pub fn label_font(mut self, font: Renderer::Font) -> Self {
        self.label_font = Some(font);
        self
    }

    #[must_use]
    pub fn label_font_maybe(mut self, maybe_font: Option<Renderer::Font>) -> Self {
        self.label_font = maybe_font;
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: &'a char) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub fn icon_maybe(mut self, maybe_icon: Option<&'a char>) -> Self {
        self.icon = maybe_icon;
        self
    }

    #[must_use]
    pub fn icon_font(mut self, font: Renderer::Font) -> Self {
        self.icon_font = Some(font);
        self
    }

    #[must_use]
    pub fn icon_font_maybe(mut self, maybe_font: Option<Renderer::Font>) -> Self {
        self.icon_font = maybe_font;
        self
    }

    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub fn corner_style(mut self, corner_style: CornerStyle) -> Self {
        self.corner_style = corner_style;
        self
    }

    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    #[must_use]
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(OnPress::Direct(message));
        self
    }

    #[must_use]
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message.map(OnPress::Direct);
        self
    }

    #[must_use]
    pub fn on_press_with(mut self, on_press: impl Fn() -> Message + 'a) -> Self {
        self.on_press = Some(OnPress::Closure(Box::new(on_press)));
        self
    }

    #[must_use]
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(button: Button<'a, Message>) -> Self {
        let content = row![
            button.icon.map(|i| text(i)
                .wrapping(text::Wrapping::None)
                .size(button.size.icon_size())
                .font_maybe(button.icon_font)),
            button.label.map(|l| text(l)
                .wrapping(text::Wrapping::None)
                .size(button.size.font_size())
                .font_maybe(button.label_font))
        ]
        .align_y(Alignment::Center)
        .spacing(button.size.spacing());

        let button_widget = iced_widget::button(center(content))
            .width(button.size.width())
            .height(button.size.height())
            .padding(button.size.padding())
            .style(move |_, status| {
                style(
                    status,
                    button.selected,
                    button.size,
                    button.style,
                    button.corner_style,
                    button.theme,
                )
            });

        let button_widget = match button.on_press {
            Some(on_press) => match on_press {
                OnPress::Direct(on_press) => button_widget.on_press(on_press),
                OnPress::Closure(on_press) => button_widget.on_press_with(on_press),
            },
            None => button_widget,
        };

        button_widget.into()
    }
}
