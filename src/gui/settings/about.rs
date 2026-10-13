use std::sync::LazyLock;

use iced::{Alignment, Border, Element, Length, Pixels, padding};
use iced_core::text::{IntoFragment, LineHeight};
use iced_m3::{
    theme::ColorScheme,
    widget::{badge, button, card, hybrid_icon::Icon, spacer},
};
use iced_widget::{column, container, image, row, space, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_LARGE, SPACING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
    font::{self, bold_text},
    formatter::format_album_duration,
    icons::{self, icon_filled},
    settings::{CLOSE_BUTTON_SIZE, Message},
    widget::icon_shape::IconShape,
};

const BIO_TEXT: &str = "Hi, I'm tpaau - the dev behind Chilen!";
const IMAGE_SIZE: f32 = 96.0;
const VERSION: &str = env!("CARGO_PKG_VERSION");
const COMMIT: &str = env!("APP_GIT_COMMIT");
const REPOSITORY_URL: &str = "https://github.com/tpaau/chilen";
const WEBSITE_URL: &str = "https://tpaau.github.io/chilen/";
const CONTACT_URL: &str = "https://tpaau.dev/#contact";
static DEBUG_ENTRIES: LazyLock<Vec<Entry<'static>>> = LazyLock::new(|| {
    vec![
        Entry::new(*icons::COMMIT, "Commit hash", COMMIT),
        Entry::new(*icons::CODE, "Version", VERSION),
    ]
});

#[derive(Clone)]
struct Entry<'a> {
    icon: text::Fragment<'a>,
    label: text::Fragment<'a>,
    value: text::Fragment<'a>,
}

impl<'a> Entry<'a> {
    pub fn new(
        icon: impl IntoFragment<'a>,
        label: impl IntoFragment<'a>,
        value: impl IntoFragment<'a>,
    ) -> Self {
        Self {
            icon: icon.into_fragment(),
            label: label.into_fragment(),
            value: value.into_fragment(),
        }
    }
}

fn entry<'a>(entry: Entry<'a>, theme: &(impl ColorScheme + ?Sized)) -> Element<'a, Message> {
    container(
        row![
            icon_filled(entry.icon, icons::SIZE_REGULAR).color(theme.on_surface_variant()),
            text(entry.label)
                .color(theme.on_surface_variant())
                .size(font::SIZE_REGULAR),
            space().width(Length::Fill),
            text(entry.value)
                .size(font::SIZE_REGULAR)
                .color(theme.on_surface_variant())
        ]
        .spacing(SPACING_SMALL)
        .align_y(Alignment::Center),
    )
    .padding(padding::vertical(SPACING_SMALL))
    .into()
}

fn table<'a>(
    title: impl IntoFragment<'a>,
    entries: Vec<Entry<'a>>,
    theme: &(impl ColorScheme + ?Sized),
) -> Element<'a, Message> {
    let entries = entries
        .into_iter()
        .map(|e| entry(e, theme))
        .intersperse_with(|| {
            container(spacer(theme.outline_variant()))
                .padding(padding::horizontal(SPACING_SMALL / 2.0))
                .into()
        });

    column![
        bold_text(title)
            .color(theme.on_surface())
            .size(font::SIZE_LARGER),
        column(entries),
    ]
    .align_x(Alignment::Center)
    .spacing(SPACING_SMALL)
    .into()
}

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let accent = state.settings_state.random_accent;

    let logo_stuff = column![
        IconShape::new(
            state.settings_state.random_shape.clone(),
            accent.color(&state.theme),
            icons::SNOWFLAKE.into_fragment(),
            accent.on_color(&state.theme),
        )
        .size(260.0)
        .icon_size(180.0),
        column![
            badge(
                badge::Style {
                    container_color: accent.color_container(&state.theme),
                    label_color: accent.on_color_container(&state.theme),
                    container_border: Border::default().rounded(f32::MAX)
                },
                bold_text("Chilen")
                    .color(state.theme.on_surface())
                    .size(50)
                    .line_height(LineHeight::Absolute(Pixels(50.0)))
            )
            .label(format!("v{VERSION}")),
            text("Fully offline music player for your library").color(state.theme.on_surface())
        ]
        .align_x(Alignment::Center)
        .spacing(SPACING_SMALLER),
    ]
    .align_x(Alignment::Center)
    .spacing(SPACING_REGULAR);

    let buttons = row![
        button(
            button::Style::filled(&state.theme, accent),
            button::Content::Full {
                icon: Icon::Text {
                    text: icons::CODE.into_fragment(),
                    font: Some(icons::filled())
                },
                label: "Source Code".into()
            }
        )
        .size(button::Size::medium())
        .on_press(Message::OpenURL(REPOSITORY_URL.to_owned())),
        button(
            button::Style::filled(&state.theme, accent),
            button::Content::Full {
                icon: Icon::Text {
                    text: icons::LANGUAGE.into_fragment(),
                    font: Some(icons::filled())
                },
                label: "Website".into()
            }
        )
        .size(button::Size::medium())
        .on_press(Message::OpenURL(WEBSITE_URL.to_owned()))
    ]
    .spacing(SPACING_LARGE);

    let profile_card = card(
        card::Style {
            border: Border::default()
                .rounded(ROUNDING_LARGE)
                .color(state.theme.outline_variant())
                .width(1),
            ..card::Style::outlined(&state.theme)
        },
        row![
            image(state.settings_state.pfp_image_handle.clone())
                .border_radius(f32::MAX)
                .width(IMAGE_SIZE)
                .height(IMAGE_SIZE),
            column![
                bold_text("tpaau")
                    .color(state.theme.on_surface())
                    .size(24.0),
                text(BIO_TEXT)
                    .size(font::SIZE_REGULAR)
                    .color(state.theme.on_surface_variant())
            ]
            .spacing(SPACING_SMALLER),
            space().width(Length::Fill),
            button(
                button::Style::outlined(&state.theme),
                button::Content::Full {
                    icon: Icon::Text {
                        text: icons::MAIL.into_fragment(),
                        font: Some(icons::filled())
                    },
                    label: "Contact".into()
                }
            )
            .size(button::Size::medium())
            .on_press(Message::OpenURL(CONTACT_URL.to_owned()))
        ]
        .spacing(SPACING_SMALL)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill);

    let library_stats = table(
        "Library Statistics",
        vec![
            Entry::new(
                *icons::MUSIC_NOTE,
                "Tracks loaded",
                state
                    .library
                    .as_ref()
                    .map(|l| l.tracks.len().to_string())
                    .unwrap_or("Unavailable".to_string()),
            ),
            Entry::new(
                *icons::ALBUM,
                "Albums loaded",
                state
                    .library
                    .as_ref()
                    .map(|l| l.albums.len().to_string())
                    .unwrap_or("Unavailable".to_string()),
            ),
            Entry::new(
                *icons::ARTIST,
                "Artists loaded",
                state
                    .library
                    .as_ref()
                    .map(|l| l.artists.len().to_string())
                    .unwrap_or("Unavailable".to_string()),
            ),
            Entry::new(
                *icons::GENRES,
                "Genres loaded",
                state
                    .library
                    .as_ref()
                    .map(|l| l.genres.len().to_string())
                    .unwrap_or("Unavailable".to_string()),
            ),
            Entry::new(
                *icons::SCHEDULE,
                "Total duration",
                state
                    .library
                    .as_ref()
                    .map(|l| format_album_duration(l.total_duration))
                    .unwrap_or("Unavailable".to_string()),
            ),
        ],
        &state.theme,
    );
    let debug_info = table("Info", DEBUG_ENTRIES.to_vec(), &state.theme);

    let content = container(
        column![logo_stuff, buttons, profile_card, library_stats, debug_info]
            .spacing(2.0 * SPACING_LARGE)
            .align_x(Alignment::Center),
    )
    .center_x(Length::Fill);

    iced_widget::scrollable(column![content, space().height(CLOSE_BUTTON_SIZE.height)])
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
