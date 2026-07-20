use iced::{Background, Color, Element, Padding, Pixels, alignment::Horizontal, border::Radius};
use iced_widget::{
    Renderer, Theme,
    core::{Length, Size, Widget},
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
        theme: &impl ColorScheme,
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
        self.content.into()
    }

    pub fn new(
        placeholder: &str,
        value: &'a str,
        theme: &'a impl ColorScheme,
        style: Style,
    ) -> Self {
        Self {
            value,
            content: iced_widget::text_input(placeholder, value)
                .style(move |_, status| Self::style_internal(status, theme, style))
                .padding(12),
        }
    }

    pub fn secure(mut self, is_secure: bool) -> Self {
        self.content = self.content.secure(is_secure);
        self
    }

    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.content = self.content.on_input(on_input);
        self
    }

    pub fn on_input_maybe(mut self, on_input: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.content = self.content.on_input_maybe(on_input);
        self
    }

    pub fn on_submit(mut self, message: Message) -> Self {
        self.content = self.content.on_submit(message);
        self
    }

    pub fn on_submit_maybe(mut self, message: Option<Message>) -> Self {
        self.content = self.content.on_submit_maybe(message);
        self
    }

    pub fn on_paste(mut self, on_paste: impl Fn(String) -> Message + 'a) -> Self {
        self.content = self.content.on_paste(on_paste);
        self
    }

    pub fn on_paste_maybe(mut self, on_paste: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.content = self.content.on_paste_maybe(on_paste);
        self
    }

    pub fn font(mut self, font: iced::Font) -> Self {
        self.content = self.content.font(font);
        self
    }

    pub fn icon(mut self, icon: Icon<iced::Font>) -> Self {
        self.content = self.content.icon(icon);
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.content = self.content.width(width);
        self
    }

    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.content = self.content.padding(padding);
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.content = self.content.size(size);
        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.content = self.content.line_height(line_height);
        self
    }

    pub fn align_x(mut self, alignment: impl Into<Horizontal>) -> Self {
        self.content = self.content.align_x(alignment);
        self
    }

    pub fn style(
        mut self,
        style: impl Fn(&Theme, iced_widget::text_input::Status) -> iced_widget::text_input::Style + 'a,
    ) -> Self {
        self.content = self.content.style(style);
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
