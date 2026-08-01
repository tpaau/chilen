use iced::{
    Alignment, Border, Element, Font, Length, Size,
    advanced::{Text, text::Paragraph},
    padding,
};
use iced_widget::{
    button, center, center_x, column, responsive, row,
    text::{LineHeight, Shaping, Wrapping},
};

use crate::{style::mix_colors, theme::ColorScheme};

const COMPACT_BUTTON_SIZE: Size<f32> = Size {
    width: 56.0,
    height: 32.0,
};
const LARGE_BUTTON_SIZE: Size<f32> = Size {
    width: 100.0,
    height: 40.0,
};
const LARGE_BUTTON_SPACING: f32 = 24.0;
const DEFAULT_LABEL_SIZE: f32 = 14.0;
const DEFAULT_ICON_SIZE: f32 = 24.0;
const COMPACT_VERTICAL_PADDING: f32 = 6.0;
const LARGE_VERTICAL_PADDING: f32 = 12.0;
const LARGE_BUTTON_INTERNAL_SPACING: f32 = 4.0;
const LARGE_BUTTON_PADDING: f32 = 16.0;

#[derive(Default)]
pub enum Mode {
    Compact,
    Large,
    #[default]
    Auto,
}

pub struct Item<'a, Message> {
    pub icon: &'a char,
    pub label: &'a str,
    pub message: Message,
}

pub struct Navbar<'a, Message> {
    compact: bool,
    items: Vec<Item<'a, Message>>,
    theme: &'a dyn ColorScheme,
    font: Option<Font>,
    icon_font_active: Option<Font>,
    icon_font_inactive: Option<Font>,
    mode: Mode,
    focused_index: usize,
    icon_size: f32,
    label_size: f32,
}

impl<'a, Message> Navbar<'a, Message> {
    #[must_use]
    pub fn new(items: Vec<Item<'a, Message>>, theme: &'a impl ColorScheme) -> Self {
        Self {
            compact: false,
            items,
            theme,
            font: None,
            icon_font_active: None,
            icon_font_inactive: None,
            mode: Mode::default(),
            focused_index: 0,
            icon_size: DEFAULT_ICON_SIZE,
            label_size: DEFAULT_LABEL_SIZE,
        }
    }

    #[must_use]
    pub fn compact(mut self, val: bool) -> Self {
        self.compact = val;
        self
    }

    #[must_use]
    pub fn item(mut self, item: Item<'a, Message>) -> Self {
        self.items.push(item);
        self
    }

    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    #[must_use]
    pub fn icon_font_active(mut self, font: Font) -> Self {
        self.icon_font_active = Some(font);
        self
    }

    #[must_use]
    pub fn icon_font_inactive(mut self, font: Font) -> Self {
        self.icon_font_inactive = Some(font);
        self
    }

    #[must_use]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub fn focused_index(mut self, index: usize) -> Self {
        self.focused_index = index;
        self
    }

    #[must_use]
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    #[must_use]
    pub fn label_size(mut self, size: f32) -> Self {
        self.label_size = size;
        self
    }
}

impl<'a, Message> From<Navbar<'a, Message>> for Element<'a, Message, iced::Theme, iced::Renderer>
where
    Message: 'a + Clone,
{
    fn from(navbar: Navbar<'a, Message>) -> Self {
        let font = navbar.font.unwrap_or_default();

        let mut max_width = 0.0;
        for item in &navbar.items {
            let p = iced::advanced::graphics::text::Paragraph::with_text(Text {
                content: item.label,
                bounds: Size::INFINITE,
                size: navbar.label_size.into(),
                line_height: LineHeight::default(),
                font,
                align_x: iced_widget::text::Alignment::Left,
                align_y: iced::alignment::Vertical::Top,
                shaping: Shaping::Advanced,
                wrapping: Wrapping::None,
            });
            let width = p.min_bounds().width;

            if width > max_width {
                max_width = width
            }
        }

        // There seems to be a slight mismatch between the calculated and actual text size. The one
        // I estimate here is sometimes less than one pixel off than the actual size, but that
        // already causes the text to wrap which just looks bad.
        //
        // So I apply the padding here because that less than 1px won't even make a difference
        // visually.
        max_width += navbar.icon_size + LARGE_BUTTON_INTERNAL_SPACING + 2.0 * LARGE_BUTTON_PADDING;

        responsive(move |size| {
            let compact = match navbar.mode {
                Mode::Compact => true,
                Mode::Large => false,
                Mode::Auto => {
                    navbar.items.len() as f32 * max_width
                        + (navbar.items.len() as i64 - 1) as f32 * LARGE_BUTTON_SPACING
                        > size.width
                }
            };

            let font = navbar.font.unwrap_or_default();
            let icon_font_active = navbar
                .icon_font_active
                .unwrap_or(navbar.icon_font_inactive.unwrap_or_default());
            let icon_font_inactive = navbar
                .icon_font_inactive
                .unwrap_or(navbar.icon_font_active.unwrap_or_default());
            let buttons = if navbar.items.is_empty() {
                Vec::new()
            } else {
                if compact {
                    let mut buttons = Vec::with_capacity(navbar.items.len());
                    for (i, item) in navbar.items.iter().enumerate() {
                        let active = i == navbar.focused_index;
                        let label_color = if active {
                            navbar.theme.secondary()
                        } else {
                            navbar.theme.on_surface_variant()
                        };
                        let icon_color = if active {
                            navbar.theme.on_secondary_container()
                        } else {
                            navbar.theme.on_surface_variant()
                        };
                        let icon_font = if active {
                            icon_font_active
                        } else {
                            icon_font_inactive
                        };

                        buttons.push(
                            center_x(
                                column![
                                    button(center(
                                        iced_widget::text(item.icon)
                                            .font(icon_font)
                                            .color(icon_color)
                                            .size(navbar.icon_size)
                                            .align_x(Alignment::Center)
                                            .align_y(Alignment::Center)
                                    ))
                                    .style(move |_, status| {
                                        let state_layer_color =
                                            navbar.theme.on_secondary_container();
                                        let layer_opacity = match status {
                                            button::Status::Active => 0.0,
                                            button::Status::Hovered => 0.08,
                                            button::Status::Pressed => 0.1,
                                            button::Status::Disabled => unreachable!(),
                                        };
                                        let color = if active {
                                            mix_colors(
                                                state_layer_color,
                                                navbar.theme.secondary_container(),
                                                layer_opacity,
                                            )
                                        } else {
                                            state_layer_color.scale_alpha(layer_opacity)
                                        };
                                        button::Style {
                                            background: Some(iced::Background::Color(color)),
                                            border: Border {
                                                radius: f32::MAX.into(),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        }
                                    })
                                    .on_press(item.message.clone())
                                    .width(Length::Fixed(COMPACT_BUTTON_SIZE.width))
                                    .height(Length::Fixed(COMPACT_BUTTON_SIZE.height)),
                                    iced_widget::text(item.label)
                                        .font(font)
                                        .color(label_color)
                                        .size(navbar.label_size)
                                        .wrapping(Wrapping::None),
                                ]
                                .align_x(Alignment::Center)
                                .spacing(4.0),
                            )
                            .into(),
                        );
                    }
                    buttons
                } else {
                    let mut buttons = Vec::with_capacity(navbar.items.len());
                    for (i, item) in navbar.items.iter().enumerate() {
                        let active = i == navbar.focused_index;
                        let label_color = if active {
                            navbar.theme.secondary()
                        } else {
                            navbar.theme.on_surface_variant()
                        };
                        let icon_color = if active {
                            navbar.theme.on_secondary_container()
                        } else {
                            navbar.theme.on_surface_variant()
                        };
                        let icon_font = if active {
                            icon_font_active
                        } else {
                            icon_font_inactive
                        };

                        buttons.push(
                            button(center(
                                row![
                                    iced_widget::text(item.icon)
                                        .font(icon_font)
                                        .color(icon_color)
                                        .size(navbar.icon_size),
                                    iced_widget::text(item.label)
                                        .font(font)
                                        .color(label_color)
                                        .size(navbar.label_size)
                                ]
                                .align_y(Alignment::Center)
                                .spacing(LARGE_BUTTON_INTERNAL_SPACING),
                            ))
                            .style(move |_, status| {
                                let state_layer_color = navbar.theme.on_secondary_container();
                                let layer_opacity = match status {
                                    button::Status::Active => 0.0,
                                    button::Status::Hovered => 0.08,
                                    button::Status::Pressed => 0.1,
                                    button::Status::Disabled => unreachable!(),
                                };
                                let color = if active {
                                    mix_colors(
                                        state_layer_color,
                                        navbar.theme.secondary_container(),
                                        layer_opacity,
                                    )
                                } else {
                                    state_layer_color.scale_alpha(layer_opacity)
                                };
                                button::Style {
                                    background: Some(iced::Background::Color(color)),
                                    border: Border {
                                        radius: f32::MAX.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            })
                            .on_press(item.message.clone())
                            .width(Length::Fixed(max_width))
                            .height(Length::Fixed(LARGE_BUTTON_SIZE.height))
                            .into(),
                        )
                    }
                    buttons
                }
            };

            let padding = if compact {
                COMPACT_VERTICAL_PADDING
            } else {
                LARGE_VERTICAL_PADDING
            };
            let spacing = if compact { 0.0 } else { LARGE_BUTTON_SPACING };
            center_x(row(buttons).spacing(spacing))
                .padding(padding::vertical(padding))
                .into()
        })
        .height(Length::Shrink)
        .into()
    }
}
