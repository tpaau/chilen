use chilen_widget::theme_preview::ThemePreview;
use iced::{Element, Length, Task};
use iced_m3::{
    theme::{ColorScheme, Mode},
    widget::card,
};
use iced_widget::{column, mouse_area, row};

use crate::{
    gui::{
        Chilen, SPACING_REGULAR, SPACING_SMALL,
        font::{self, bold_text},
        themes::THEMES,
    },
    settings,
};

#[derive(Debug, Clone)]
pub enum Message {
    SetTheme(usize),
    SetDarkMode(Mode),
}

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let preview_size = 48.0;
    let themes = THEMES.iter().enumerate().map(|(i, t)| {
        let preview = match state.theme.mode {
            iced_m3::theme::Mode::Dark => ThemePreview {
                color_top: t.dark.primary,
                color_left: t.dark.primary_container,
                color_right: t.dark.surface_container,
                size: preview_size,
            },
            iced_m3::theme::Mode::Light => ThemePreview {
                color_top: t.light.primary,
                color_left: t.light.primary_container,
                color_right: t.light.surface_container,
                size: preview_size,
            },
        };

        mouse_area(preview)
            .on_press(Message::SetTheme(i))
            .interaction(iced::mouse::Interaction::Pointer)
            .into()
    });

    let themes = card(
        card::Style::elevated(&state.theme),
        column![
            bold_text("Palette")
                .size(font::SIZE_LARGER)
                .color(state.theme.on_surface()),
            row(themes).spacing(SPACING_REGULAR),
        ]
        .spacing(SPACING_SMALL),
    )
    .width(Length::Fill);

    column![themes].spacing(SPACING_REGULAR).into()
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::SetTheme(i) => {
            let theme = &THEMES[i];
            state.settings.theme_name = theme.name.to_string();
            state.theme.dark = theme.dark;
            state.theme.light = theme.light;
        }
        Message::SetDarkMode(mode) => {
            state.settings.theme_mode = mode;
            state.theme.mode = mode;
        }
    }
    settings::save(state);
    Task::none()
}
