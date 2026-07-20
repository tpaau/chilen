use iced::{Background, Color, Element, border::Radius};
use iced_widget::{
    Renderer, Theme,
    core::{Length, Size, Widget},
};

use crate::theme::ColorScheme;

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
    fn style(
        status: iced_widget::text_input::Status,
        theme: &impl ColorScheme,
        style: Style,
    ) -> iced_widget::text_input::Style {
        match style {
            Style::Outlined => iced_widget::text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                placeholder: if status == iced_widget::text_input::Status::Disabled {
                    theme.on_surface_variant().scale_alpha(0.7)
                } else {
                    theme.on_surface_variant()
                },
                border: iced::Border {
                    color: match status {
                        iced_widget::text_input::Status::Active => theme.outline(),
                        iced_widget::text_input::Status::Hovered => theme.outline(),
                        iced_widget::text_input::Status::Focused { is_hovered: _ } => {
                            theme.primary()
                        }
                        iced_widget::text_input::Status::Disabled => {
                            theme.outline().scale_alpha(0.7)
                        }
                    },
                    width: if let iced_widget::text_input::Status::Focused { is_hovered: _ } =
                        status
                    {
                        3.0
                    } else {
                        2.0
                    },
                    radius: Radius::from(2.0),
                },
                icon: theme.on_surface_variant(),
                selection: theme.on_primary(),
                value: theme.primary(),
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
                .style(move |_, status| Self::style(status, theme, style)),
        }
    }

    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.content = self.content.on_input(on_input);
        self
    }

    pub fn on_submit(mut self, message: Message) -> Self {
        self.content = self.content.on_submit(message);
        self
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for TextInput<'_, Message>
where
    Message: 'a + Clone,
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
        style: &iced::advanced::renderer::Style,
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
