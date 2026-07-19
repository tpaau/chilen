pub mod dialog;
pub mod drop_down_menu;

use iced::Element;
use iced_widget::container;

use crate::{theme::Palette, widget::dialog::Dialog};

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
