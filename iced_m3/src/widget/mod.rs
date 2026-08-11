pub mod button;
pub mod dialog;
#[cfg(feature = "advanced")]
pub mod drop_down_menu;
pub mod fab_menu;
pub mod navbar;
pub mod text_input;
pub mod vertical_menu;

use iced::Element;
use iced_widget::container;

use crate::{
    theme::{ColorScheme, Palette},
    widget::{
        button::Button,
        dialog::Dialog,
        drop_down_menu::{DropDownMenu, Placement},
        fab_menu::FABMenu,
        navbar::Navbar,
        text_input::TextInput,
    },
};

#[must_use]
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

#[cfg(feature = "advanced")]
#[must_use]
pub fn drop_down_menu<'a, Message, Theme, Renderer>(
    trigger: impl Fn(bool) -> Element<'a, Message, Theme, Renderer> + 'a,
    menu: Option<impl Into<Element<'a, Message, Theme, Renderer>>>,
    placement: Placement,
) -> DropDownMenu<'a, Message, Theme, Renderer> {
    DropDownMenu::<'a, Message, Theme, Renderer>::new(trigger, menu, placement)
}

#[must_use]
pub fn menu<'a, Message>(
    sections: Vec<vertical_menu::Group<'a, Message>>,
    theme: &'a dyn ColorScheme,
) -> vertical_menu::Menu<'a, Message> {
    vertical_menu::Menu::new(sections, theme)
}

#[must_use]
pub fn text_input<'a, Message: Clone, Renderer>(
    placeholder: &str,
    value: &'a str,
    theme: &'a impl ColorScheme,
) -> TextInput<'a, Message> {
    TextInput::new(placeholder, value, theme)
}

#[must_use]
pub fn navbar<'a, Message, Theme, Renderer>(
    items: Vec<navbar::Item<'a, Message>>,
    theme: &'a impl ColorScheme,
) -> Navbar<'a, Message> {
    Navbar::new(items, theme)
}

#[must_use]
pub fn button<'a, Message>(theme: &'a dyn ColorScheme) -> Button<'a, Message> {
    Button::new(theme)
}

#[must_use]
pub fn fab_menu<'a, Message, I>(
    entries: I,
    icon: &'a dyn Fn(bool) -> char,
    theme: &'a dyn ColorScheme,
) -> FABMenu<'a, Message>
where
    I: IntoIterator<Item = fab_menu::Entry<'a, Message>>,
{
    FABMenu::new(entries, icon, theme)
}
