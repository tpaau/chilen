pub mod dialog;
pub mod drop_down_menu;
pub mod text_input;

use iced::Element;
use iced_widget::container;

use crate::{
    theme::{ColorScheme, Palette},
    widget::{dialog::Dialog, text_input::TextInput},
};

pub fn dialog<'a, Message, Theme, Renderer>(
    is_open: bool,
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    palette: &'a Palette,
) -> Dialog<'a, Message, Theme, Renderer>
where
    Renderer: 'a + iced_widget::core::Renderer + iced_widget::core::text::Renderer,
    Theme: 'a + dialog::Catalog,
    Message: 'a + Clone,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
{
    Dialog::new(is_open, base, content, palette)
}

pub fn text_input<'a, Message: Clone, Renderer>(
    placeholder: &str,
    value: &'a str,
    theme: &'a impl ColorScheme,
    style: crate::widget::text_input::InputStyle,
) -> TextInput<'a, Message> {
    TextInput::new(placeholder, value, theme, style)
}
