use chilen_widget::theme_preview::ThemePreview;
use iced::{Alignment, Element, Length, Task};
use iced_m3::{
    theme::{
        ColorScheme,
        Mode::{self},
    },
    widget::{
        card::{self, MAX_CARD_BETWEEN_PADDING},
        switch,
    },
};
use iced_widget::{
    column, mouse_area, row,
    scrollable::{Direction, Scrollbar},
    space, text,
};

use crate::{
    gui::{
        Chilen, SPACING_REGULAR, SPACING_SMALL,
        font::{self, bold_text},
        themes::THEMES,
        widget::theme_mode_preview::ThemeModePreview,
    },
    settings,
};

#[derive(Debug, Clone)]
pub enum Message {
    SetTheme(usize),
    SetDarkMode(Mode),
    ToggleAutoTheme,
}

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let preview_size = 48.0;
    let themes = THEMES.iter().enumerate().map(|(i, t)| {
        let preview = match state.theme.mode {
            iced_m3::theme::Mode::Dark => ThemePreview {
                color_top: t.dark.primary,
                color_left: t.dark.primary_container,
                color_right: t.dark.surface,
                size: preview_size,
            },
            iced_m3::theme::Mode::Light => ThemePreview {
                color_top: t.light.primary,
                color_left: t.light.primary_container,
                color_right: t.light.surface,
                size: preview_size,
            },
            iced_m3::theme::Mode::Black => ThemePreview {
                color_top: t.dark.primary,
                color_left: t.dark.primary_container,
                color_right: t.dark.surface,
                size: preview_size,
            },
        };

        mouse_area(preview)
            .on_press(Message::SetTheme(i))
            .interaction(iced::mouse::Interaction::Pointer)
            .into()
    });

    let modes = row![
        ThemeModePreview {
            theme: &state.theme.dark,
            label: "Dark",
            selected: state.theme.mode == Mode::Dark,
            on_press: (!state.settings.theme_auto_mode).then_some(
                iced_m3::widget::OnPress::Direct(Message::SetDarkMode(Mode::Dark))
            )
        },
        ThemeModePreview {
            theme: &state.theme.light,
            label: "Light",
            selected: state.theme.mode == Mode::Light,
            on_press: (!state.settings.theme_auto_mode).then_some(
                iced_m3::widget::OnPress::Direct(Message::SetDarkMode(Mode::Light))
            )
        },
        ThemeModePreview {
            theme: &state.theme.black,
            label: "Black",
            selected: state.theme.mode == Mode::Black,
            on_press: (!state.settings.theme_auto_mode).then_some(
                iced_m3::widget::OnPress::Direct(Message::SetDarkMode(Mode::Black))
            )
        }
    ]
    .spacing(MAX_CARD_BETWEEN_PADDING);

    let auto_switch = mouse_area(
        row![
            column![
                text("Automatic theme")
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface()),
                text("Follows your system's light or dark mode preference.")
                    .size(font::SIZE_REGULAR)
                    .color(state.theme.on_surface_variant())
            ],
            space().width(Length::Fill),
            switch(&state.theme, state.settings.theme_auto_mode)
                .on_toggle(Message::ToggleAutoTheme)
        ]
        .align_y(Alignment::Center),
    )
    .interaction(iced::mouse::Interaction::Pointer)
    .on_press(Message::ToggleAutoTheme);

    let themes = iced_m3::widget::card(
        card::Style::elevated(&state.theme),
        column![
            column![
                bold_text("Palette")
                    .size(font::SIZE_LARGER)
                    .color(state.theme.on_surface()),
                text("Make Chilen yours!")
                    .size(font::SIZE_REGULAR)
                    .color(state.theme.on_surface_variant()),
            ],
            iced_widget::scrollable(row(themes).spacing(SPACING_REGULAR))
                .direction(Direction::Horizontal(Scrollbar::default()))
                .style(|_, status| iced_m3::style::scrollable(status, &state.theme)),
        ]
        .spacing(SPACING_REGULAR),
    )
    .width(Length::Fill);

    column![themes, auto_switch, modes]
        .spacing(SPACING_REGULAR)
        .into()
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::SetTheme(i) => {
            let theme = &THEMES[i];
            state.settings.theme_name = theme.name.to_string();
            state.theme = theme.clone().into_theme(state.theme.mode);
        }
        Message::SetDarkMode(mode) => {
            state.settings.theme_mode = mode;
            state.theme.mode = mode;
        }
        Message::ToggleAutoTheme => {
            state.settings.theme_auto_mode = !state.settings.theme_auto_mode
        }
    }
    settings::save(state);
    Task::none()
}
