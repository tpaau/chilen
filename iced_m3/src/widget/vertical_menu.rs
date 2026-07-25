use std::sync::LazyLock;

use iced::{Alignment, Border, Color, Element, Font, Length, Pixels, border::Radius, padding};
use iced_widget::{button, column, container, row, space, text};

use crate::{
    DIM_ALPHA, HOVER_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY,
    style::{Elevation, shadow},
    theme::ColorScheme,
    widget::drop_down_menu,
};

pub enum Action<'a, Message> {
    Menu(Vec<Group<'a, Message>>),
    Message(Option<Message>),
}

pub enum Entry<'a, Message> {
    Button {
        icon: Option<&'a char>,
        label: &'a str,
        supporting_text: Option<&'a str>,
        action: Action<'a, Message>,
    },
    Separator,
}

pub struct Group<'a, Message> {
    pub label: Option<&'a str>,
    pub entries: Vec<Entry<'a, Message>>,
}

impl<'a, Message> Group<'a, Message> {
    pub fn new(entries: Vec<Entry<'a, Message>>) -> Self {
        Self {
            label: None,
            entries,
        }
    }
}

pub struct Menu<'a, Message> {
    groups: Vec<Group<'a, Message>>,
    vibrant: bool,
    font: Option<Font>,
    icon_font: Option<Font>,
    width: Option<f32>,
    theme: &'a dyn ColorScheme,
    trailing_icon: &'a char,
}

pub static ARROW_RIGHT: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5df).unwrap());

impl<'a, Message> Menu<'a, Message> {
    #[must_use]
    pub fn new(groups: Vec<Group<'a, Message>>, theme: &'a dyn ColorScheme) -> Self {
        Self {
            groups,
            vibrant: false,
            font: None,
            icon_font: None,
            width: Some(192.0),
            theme,
            trailing_icon: &ARROW_RIGHT,
        }
    }

    #[must_use]
    pub fn vibrant(mut self, vibrant: bool) -> Self {
        self.vibrant = vibrant;
        self
    }

    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    #[must_use]
    pub fn icon_font(mut self, icon_font: Font) -> Self {
        self.icon_font = Some(icon_font);
        self
    }

    #[must_use]
    pub fn width(mut self, width: Option<f32>) -> Self {
        self.width = width;
        self
    }
}

const SECTION_PADDING: f32 = 4.0;
const SPACING: f32 = 2.0;
const LABEL_HEIGHT: Length = Length::Fixed(32.0);
const BUTTON_HEIGHT: Length = Length::Fixed(48.0);
const LABEL_TEXT_SIZE: Pixels = Pixels(16.0);
const SUPPORTING_TEXT_SIZE: Pixels = Pixels(12.0);
const ICON_SIZE: Pixels = Pixels(20.0);
static BUTTON_RADIUS: LazyLock<Radius> = LazyLock::new(|| Radius::new(12.0));

impl<'a, Message, Theme, Renderer> From<Menu<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a + iced::advanced::Renderer + iced::advanced::text::Renderer,
    <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
    Message: 'a + Clone,
    Theme: 'a
        + iced_widget::container::Catalog
        + iced_widget::text::Catalog
        + iced_widget::button::Catalog,
    <Theme as iced_widget::container::Catalog>::Class<'a>:
        From<iced_widget::container::StyleFn<'a, Theme>>,
    <Theme as iced_widget::text::Catalog>::Class<'a>: From<iced_widget::text::StyleFn<'a, Theme>>,
    <Theme as iced_widget::button::Catalog>::Class<'a>:
        From<iced_widget::button::StyleFn<'a, Theme>>,
{
    fn from(menu: Menu<'a, Message>) -> Self {
        let theme = menu.theme;
        let font = menu.font.unwrap_or_default();
        let icon_font = menu.icon_font.unwrap_or_default();
        let shadow_color = menu.theme.shadow();
        let trailing_icon = menu.trailing_icon;
        let container_width = match menu.width {
            Some(pixels) => Length::Fixed(pixels),
            None => Length::Shrink,
        };
        let label_color = if menu.vibrant {
            menu.theme.on_tertiary_container()
        } else {
            menu.theme.on_surface_variant()
        };
        let separator_color = if menu.vibrant {
            menu.theme.on_tertiary_container().scale_alpha(0.3)
        } else {
            menu.theme.outline_variant()
        };
        let bg = if menu.vibrant {
            menu.theme.tertiary_container()
        } else {
            menu.theme.surface_container_low()
        };
        let icon_color = if menu.vibrant {
            menu.theme.on_tertiary_container()
        } else {
            menu.theme.on_surface_variant()
        };
        let button_label_color = if menu.vibrant {
            menu.theme.on_tertiary_container()
        } else {
            menu.theme.on_surface()
        };
        let supporting_text_color = if menu.vibrant {
            menu.theme.on_tertiary_container()
        } else {
            menu.theme.on_surface_variant()
        };
        let state_layer_color = if menu.vibrant {
            menu.theme.on_tertiary_container()
        } else {
            menu.theme.on_surface()
        };
        let button_hover_color = state_layer_color.scale_alpha(HOVER_STATE_LAYER_OPACITY);
        let button_pressed_color = state_layer_color.scale_alpha(PRESSED_STATE_LAYER_OPACITY);

        let children: Vec<Element<'_, Message, Theme, Renderer>> = menu
            .groups
            .into_iter()
            .map(|group| -> Element<'_, Message, Theme, Renderer> {
                let mut children: Vec<Element<'a, Message, Theme, Renderer>> = Vec::new();
                if let Some(label) = group.label {
                    children.push(
                        container(
                            text(label)
                                .size(LABEL_TEXT_SIZE)
                                .font(font)
                                .height(Length::Fill)
                                .center()
                                .style(move |_| iced_widget::text::Style {
                                    color: Some(label_color),
                                }),
                        )
                        .padding(12.0)
                        .height(LABEL_HEIGHT)
                        .width(container_width)
                        .into(),
                    );
                }
                for entry in group.entries {
                    match entry {
                        Entry::Button {
                            icon,
                            label,
                            supporting_text,
                            action,
                        } => {
                            let button_disabled = match &action {
                                Action::Menu(groups) => groups.is_empty(),
                                Action::Message(message) => message.is_none(),
                            };
                            let content_alpha = if button_disabled { DIM_ALPHA } else { 1.0 };
                            let trailing_icon_visible = matches!(action, Action::Menu(_));
                            let content = move || -> Element<'a, Message, Theme, Renderer> {
                                row![
                                    icon.map(|i| text(i).size(ICON_SIZE).font(icon_font).style(
                                        move |_| text::Style {
                                            color: Some(icon_color.scale_alpha(content_alpha))
                                        }
                                    )),
                                    column![
                                        space().height(Length::Fill),
                                        text(label).size(LABEL_TEXT_SIZE).font(font).style(
                                            move |_: &Theme| text::Style {
                                                color: Some(
                                                    button_label_color.scale_alpha(content_alpha)
                                                )
                                            }
                                        ),
                                        supporting_text.map(
                                            |t| -> iced::advanced::widget::Text<'_, _, Renderer> {
                                                text(t).size(SUPPORTING_TEXT_SIZE).font(font).style(
                                                    move |_| text::Style {
                                                        color: Some(
                                                            supporting_text_color
                                                                .scale_alpha(content_alpha),
                                                        ),
                                                    },
                                                )
                                            }
                                        ),
                                        space().height(Length::Fill),
                                    ],
                                    space().width(Length::Fill),
                                    if trailing_icon_visible {
                                        Some(
                                            text(trailing_icon)
                                                .size(ICON_SIZE)
                                                .font(icon_font)
                                                .style(move |_| text::Style {
                                                    color: Some(
                                                        icon_color.scale_alpha(content_alpha),
                                                    ),
                                                }),
                                        )
                                    } else {
                                        None
                                    }
                                ]
                                .spacing(8.0)
                                .align_y(Alignment::Center)
                                .into()
                            };
                            match action {
                                Action::Menu(groups) => children.push(
                                    drop_down_menu(
                                        move |_| container(content()).height(BUTTON_HEIGHT).into(),
                                        if groups.is_empty() {
                                            None
                                        } else {
                                            Some(Menu::new(groups, theme))
                                        },
                                        drop_down_menu::Placement::RightBottom,
                                    )
                                    .into(),
                                ),
                                Action::Message(message) => children.push(
                                    button(content())
                                        .style(move |_, status| button::Style {
                                            background: Some(iced::Background::Color(
                                                match status {
                                                    button::Status::Active
                                                    | button::Status::Disabled => {
                                                        Color::TRANSPARENT
                                                    }
                                                    button::Status::Hovered => button_hover_color,
                                                    button::Status::Pressed => button_pressed_color,
                                                },
                                            )),
                                            text_color: label_color,
                                            border: Border {
                                                radius: *BUTTON_RADIUS,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })
                                        .on_press_maybe(message)
                                        .height(BUTTON_HEIGHT)
                                        .into(),
                                ),
                            }
                        }
                        Entry::Separator => children.push(
                            container(
                                container(space())
                                    .style(move |_| container::Style {
                                        background: Some(iced::Background::Color(separator_color)),
                                        ..Default::default()
                                    })
                                    .width(Length::Fill)
                                    .height(1.0),
                            )
                            .width(Length::Fill)
                            .padding(padding::horizontal(8.0))
                            .into(),
                        ),
                    }
                }

                container(column(children).spacing(2.0))
                    .style(move |_: &Theme| container::Style {
                        background: Some(iced::Background::Color(bg)),
                        border: iced::Border {
                            radius: Radius::new(16),
                            ..Default::default()
                        },
                        shadow: shadow(shadow_color, Elevation::new(0.4)),
                        ..Default::default()
                    })
                    .padding(SECTION_PADDING)
                    .width(if let Length::Fixed(val) = container_width {
                        Length::Fixed(val - 2.0 * SECTION_PADDING)
                    } else {
                        container_width
                    })
                    .into()
            })
            .collect();

        column(children).spacing(SPACING).into()
    }
}
