use iced::{Element, Pixels};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::{Accent, ColorScheme},
    widget::{advanced::menu_overlay::Placement, button, hybrid_icon::Icon},
};

use crate::gui::{SPACING_SMALLER, icons, menu_close};

pub struct MenuButton<'a, Message, Theme, Renderer>
where
    Message: Clone,
{
    surface: Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'a>,
    style: button::Style,
    menu_opened: bool,
    toggle_menu: Message,
}

impl<'a, Message, Theme, Renderer> MenuButton<'a, Message, Theme, Renderer>
where
    Message: Clone,
{
    #[must_use]
    pub fn new(
        surface: impl Fn() -> Element<'a, Message, Theme, Renderer> + 'a,
        theme: &(impl ColorScheme + ?Sized),
        toggle_menu: Message,
    ) -> Self {
        Self {
            surface: Box::new(surface),
            style: button::Style::filled(theme, Accent::Primary),
            menu_opened: false,
            toggle_menu,
        }
    }

    #[must_use]
    pub fn opened(mut self, opened: bool) -> Self {
        self.menu_opened = opened;
        self
    }
}

impl<'a, Message, Theme, Renderer> From<MenuButton<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer:
        'a + iced::advanced::text::Renderer<Font = iced::Font> + iced::advanced::svg::Renderer,
{
    fn from(value: MenuButton<'a, Message, Theme, Renderer>) -> Self {
        let size = button::Size::extra_small();
        size.width(size.height);

        let button = iced_m3::widget::button(
            value.style,
            iced_m3::widget::button::Content::Icon(Icon::Text {
                text: icons::MORE_HORIZ.into_fragment(),
                font: Some(icons::filled()),
            }),
        )
        .size(size)
        .selected(value.menu_opened)
        .on_press(value.toggle_menu.clone());

        iced_m3::widget::advanced::menu_overlay(button, value.surface, value.menu_opened)
            .placement(Placement::BottomRight {
                padding: Pixels(SPACING_SMALLER),
            })
            .on_press(move |properties| menu_close(properties, value.toggle_menu.clone()))
            .into()
    }
}
