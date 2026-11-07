use iced::{Color, Element, Pixels, border::Radius};
use iced_core::text::IntoFragment;
use iced_m3::{
    style::StateLayer,
    theme::ColorScheme,
    widget::{
        advanced::menu_overlay::Placement,
        button::{self, ElevationStates, Outline},
        hybrid_icon::Icon,
    },
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
        let style_regular = button::StateStyle {
            container: Color::TRANSPARENT,
            label: theme.on_surface(),
            icon: theme.on_surface(),
            outline: Outline::default(),
        };
        let style_selected = button::StateStyle {
            container: theme.surface_container_high(),
            ..style_regular
        };
        let state_layer = StateLayer::new(theme.on_surface());

        let style = button::Style {
            regular: style_regular,
            unselected: style_regular,
            selected: style_selected,
            disabled: style_regular,
            disabled_unselected: style_regular,
            disabled_selected: style_selected,
            elevation: ElevationStates::new(theme.shadow()),
            state_layer,
            state_layer_unselected: state_layer,
            state_layer_selected: state_layer,
        };

        Self {
            surface: Box::new(surface),
            style,
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
        let mut size = button::Size::extra_small();
        size.width = size.height;
        size.corner_radius.shape_morph = false;
        size.corner_radius.square = Radius::from(f32::MAX);

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
