use iced::{Alignment, Border, Element, Font, Length};
use iced_widget::{center, column, space, text};

use crate::{
    style::shadow,
    theme::{Accent, ColorScheme},
    widget::drop_down_menu,
};

const TRIGGER_SIZE: f32 = 56.0;
const TRIGGER_ICON_SIZE: f32 = 20.0;
const BUTTON_TRIGGER_BETWEEN_SPACE: f32 = 8.0;
const BUTTON_SPACING: f32 = 4.0;

fn trigger_style(
    accent: Accent,
    opened: bool,
    theme: &dyn ColorScheme,
) -> iced_widget::container::Style {
    let (surface, icon) = match opened {
        true => match accent {
            Accent::Primary => (theme.primary(), theme.on_primary()),
            Accent::Secondary => (theme.secondary(), theme.on_secondary()),
            Accent::Tertiary => (theme.tertiary(), theme.on_tertiary()),
        },
        false => match accent {
            Accent::Primary => (theme.on_primary(), theme.primary()),
            Accent::Secondary => (theme.on_secondary(), theme.secondary()),
            Accent::Tertiary => (theme.on_tertiary(), theme.tertiary()),
        },
    };
    iced_widget::container::Style {
        text_color: Some(icon),
        background: Some(iced::Background::Color(surface)),
        border: Border::default().rounded(match opened {
            true => f32::MAX,
            false => 16.0,
        }),
        shadow: shadow(theme.shadow(), 0.3),
        ..Default::default()
    }
}

pub struct Entry<'a, Message> {
    pub message: Message,
    pub label: &'a str,
    pub icon: Option<&'a char>,
}

pub struct FABMenu<'a, Message> {
    label_font: Option<Font>,
    icon_font: Option<Font>,
    accent: Accent,
    entries: Vec<Entry<'a, Message>>,
    icon: &'a dyn Fn(bool) -> char,
    theme: &'a dyn ColorScheme,
}

impl<'a, Message> FABMenu<'a, Message> {
    pub fn new<I>(entries: I, icon: &'a dyn Fn(bool) -> char, theme: &'a dyn ColorScheme) -> Self
    where
        I: IntoIterator<Item = Entry<'a, Message>>,
    {
        Self {
            label_font: None,
            icon_font: None,
            accent: Accent::default(),
            entries: entries.into_iter().collect(),
            icon,
            theme,
        }
    }

    #[must_use]
    pub fn label_font(mut self, font: Font) -> Self {
        self.label_font = Some(font);
        self
    }

    #[must_use]
    pub fn label_font_maybe(mut self, maybe_font: Option<Font>) -> Self {
        self.label_font = maybe_font;
        self
    }

    #[must_use]
    pub fn icon_font(mut self, font: Font) -> Self {
        self.icon_font = Some(font);
        self
    }

    #[must_use]
    pub fn icon_font_maybe(mut self, maybe_font: Option<Font>) -> Self {
        self.icon_font = maybe_font;
        self
    }

    #[must_use]
    pub fn accent(mut self, accent: Accent) -> Self {
        self.accent = accent;
        self
    }

    #[must_use]
    pub fn entry(mut self, entry: Entry<'a, Message>) -> Self {
        self.entries.push(entry);
        self
    }

    #[must_use]
    pub fn maybe_entry(mut self, maybe_entry: Option<Entry<'a, Message>>) -> Self {
        if let Some(e) = maybe_entry {
            self.entries.push(e)
        }
        self
    }
}

impl<'a, Message: 'a + Clone> From<FABMenu<'a, Message>> for Element<'a, Message> {
    fn from(menu: FABMenu<'a, Message>) -> Self {
        let mut buttons: Vec<Element<'a, Message>> = menu
            .entries
            .into_iter()
            .map(|e| {
                crate::widget::button(menu.theme)
                    .on_press(e.message)
                    .size(crate::widget::button::Size::Medium) // TEST: Height should be 56px
                    .label(e.label)
                    .icon_maybe(e.icon)
                    .label_font_maybe(menu.label_font)
                    .icon_font_maybe(menu.icon_font)
                    .style(crate::widget::button::Style::Tonal(menu.accent))
                    .into()
            })
            .collect();
        buttons.push(
            space()
                .height(Length::Fixed(BUTTON_TRIGGER_BETWEEN_SPACE))
                .into(),
        );

        drop_down_menu(
            move |opened| {
                center(
                    text((menu.icon)(opened))
                        .size(TRIGGER_ICON_SIZE)
                        .font_maybe(menu.icon_font),
                )
                .width(Length::Fixed(TRIGGER_SIZE))
                .height(Length::Fixed(TRIGGER_SIZE))
                .style(move |_| trigger_style(menu.accent, opened, menu.theme))
                .into()
            },
            Some(
                column(buttons)
                    .spacing(BUTTON_SPACING)
                    .align_x(Alignment::End),
            ),
            drop_down_menu::Placement::TopLeft,
        )
        .menu_transparent(true)
        .into()
    }
}
