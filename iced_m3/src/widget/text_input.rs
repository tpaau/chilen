use std::sync::Arc;

use iced::{Background, Color, Element, Padding, Pixels, alignment::Horizontal, border::Radius};
use iced_widget::{
    Renderer, Theme,
    core::{Length, Size, Widget},
    row, space, stack,
    text::LineHeight,
    text_input::Icon,
};

use crate::{DIM_ALPHA, theme::ColorScheme};

#[derive(Copy, Clone)]
pub enum Style {
    Outlined,
    Filled,
}

pub struct TextInput<'a, Message>
where
    Message: 'a + Clone,
{
    value: &'a str,
    content: iced_widget::TextInput<'a, Message, Theme, Renderer>,
    style: Style,
    theme: Arc<dyn ColorScheme>,
    background: Option<Color>,
    label_text: Option<&'a str>,
    supporting_text: Option<&'a str>,
}

impl<'a, Message: Clone> From<TextInput<'a, Message>>
    for Element<'a, Message, Theme, iced_widget::Renderer>
{
    fn from(text_input: TextInput<'a, Message>) -> Self {
        text_input.view()
    }
}

impl<'a, Message> TextInput<'a, Message>
where
    Message: 'a + Clone,
{
    fn style_internal(
        status: iced_widget::text_input::Status,
        theme: &dyn ColorScheme,
        style: Style,
    ) -> iced_widget::text_input::Style {
        match style {
            Style::Outlined => iced_widget::text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                placeholder: if status == iced_widget::text_input::Status::Disabled {
                    theme.on_surface_variant().scale_alpha(DIM_ALPHA)
                } else {
                    theme.on_surface_variant()
                },
                border: iced::Border {
                    color: match status {
                        iced_widget::text_input::Status::Active
                        | iced_widget::text_input::Status::Hovered => theme.outline(),
                        iced_widget::text_input::Status::Focused { is_hovered: _ } => {
                            theme.primary()
                        }
                        iced_widget::text_input::Status::Disabled => {
                            theme.outline().scale_alpha(DIM_ALPHA)
                        }
                    },
                    width: if let iced_widget::text_input::Status::Focused { is_hovered: _ } =
                        status
                    {
                        3.0
                    } else {
                        2.0
                    },
                    radius: Radius::from(4.0),
                },
                icon: theme.on_surface_variant(),
                selection: theme.inverse_primary(),
                value: if status == iced_widget::text_input::Status::Disabled {
                    theme.on_surface().scale_alpha(DIM_ALPHA)
                } else {
                    theme.on_surface()
                },
            },
            Style::Filled => todo!(),
        }
    }

    fn view(self) -> Element<'a, Message, Theme, Renderer> {
        let field: Element<'a, Message, Theme, Renderer> = if let Some(label_text) = self.label_text
        {
            stack![
                iced_widget::column(vec![
                    space().height(Length::Fixed(6.0)).into(),
                    self.content.into()
                ]),
                iced_widget::row(vec![
                    space().width(Length::Fixed(16.0)).into(),
                    iced_widget::container(
                        iced_widget::text(label_text)
                            .color(self.theme.primary())
                            .size(12)
                    )
                    .style(move |_| {
                        iced_widget::container::Style {
                            background: Some(Background::Color(self.background.unwrap())),
                            ..Default::default()
                        }
                    })
                    .padding(Padding {
                        left: 4.0,
                        right: 4.0,
                        ..Default::default()
                    })
                    .into()
                ]),
            ]
            .into()
        } else {
            self.content.into()
        };

        if let Some(supporting_text) = self.supporting_text {
            iced_widget::column(vec![
                field,
                row![
                    space().width(Length::Fixed(16.0)),
                    iced_widget::text(supporting_text)
                        .size(12)
                        .color(self.theme.on_surface_variant()),
                ]
                .into(),
            ])
            .spacing(4)
            .into()
        } else {
            field
        }
    }

    #[must_use]
    pub fn new(
        placeholder: &str,
        value: &'a str,
        theme: Arc<dyn ColorScheme>,
        style: Style,
    ) -> Self {
        let theme_clone = theme.clone();
        Self {
            value,
            content: iced_widget::text_input(placeholder, value)
                .style(move |_, status| Self::style_internal(status, theme_clone.as_ref(), style))
                .padding(12),
            theme,
            style,
            background: None,
            label_text: None,
            supporting_text: None,
        }
    }

    #[must_use]
    pub fn secure(mut self, is_secure: bool) -> Self {
        self.content = self.content.secure(is_secure);
        self
    }

    #[must_use]
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.content = self.content.on_input(on_input);
        self
    }

    #[must_use]
    pub fn on_input_maybe(mut self, on_input: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.content = self.content.on_input_maybe(on_input);
        self
    }

    #[must_use]
    pub fn on_submit(mut self, message: Message) -> Self {
        self.content = self.content.on_submit(message);
        self
    }

    #[must_use]
    pub fn on_submit_maybe(mut self, message: Option<Message>) -> Self {
        self.content = self.content.on_submit_maybe(message);
        self
    }

    #[must_use]
    pub fn on_paste(mut self, on_paste: impl Fn(String) -> Message + 'a) -> Self {
        self.content = self.content.on_paste(on_paste);
        self
    }

    #[must_use]
    pub fn on_paste_maybe(mut self, on_paste: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.content = self.content.on_paste_maybe(on_paste);
        self
    }

    #[must_use]
    pub fn font(mut self, font: iced::Font) -> Self {
        self.content = self.content.font(font);
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: Icon<iced::Font>) -> Self {
        self.content = self.content.icon(icon);
        self
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.content = self.content.width(width);
        self
    }

    #[must_use]
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.content = self.content.padding(padding);
        self
    }

    #[must_use]
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.content = self.content.size(size);
        self
    }

    #[must_use]
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.content = self.content.line_height(line_height);
        self
    }

    #[must_use]
    pub fn align_x(mut self, alignment: impl Into<Horizontal>) -> Self {
        self.content = self.content.align_x(alignment);
        self
    }

    #[must_use]
    pub fn style(
        mut self,
        style: impl Fn(&Theme, iced_widget::text_input::Status) -> iced_widget::text_input::Style + 'a,
    ) -> Self {
        self.content = self.content.style(style);
        self
    }

    #[must_use]
    pub fn with_label_text(mut self, label_text: &'a str, background_color: Color) -> Self {
        self.label_text = Some(label_text);
        self.background = Some(background_color);
        let theme_clone = self.theme.clone();
        self.content = self.content.style(move |_, status| {
            let mut style = Self::style_internal(status, theme_clone.as_ref(), self.style);
            style.background = Background::Color(background_color);
            style
        });
        self
    }

    #[must_use]
    pub fn with_supporting_text(mut self, supporting_text: &'a str) -> Self {
        self.supporting_text = Some(supporting_text);
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for TextInput<'_, Message>
where
    Message: Clone,
{
    fn size(&self) -> Size<Length> {
        Widget::size(&self.content)
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content.layout(
            tree,
            renderer,
            limits,
            Some(&iced_widget::text_input::Value::new(self.value)),
        )
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content.draw(
            tree,
            renderer,
            theme,
            layout,
            cursor,
            Some(&iced_widget::text_input::Value::new(self.value)),
            viewport,
        );
    }
}
