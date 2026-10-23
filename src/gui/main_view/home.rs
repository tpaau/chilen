use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_LARGE,
    font::{self, bold_text},
    icons::{ARROW_FORWARD_IOS, EQUALIZER, LIBRARY_MUSIC, MUSIC_NOTE, TRENDING_UP, icon_filled},
    main_view::Message,
    widget::icon_shape::IconShape,
};
use iced::{Alignment, Border, Element, Length, padding};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::ColorScheme,
    widget::{
        OnPress, card,
        shapes::polymorpher::{
            RoundedPolygon,
            shapes::{bun, cookie4, heart, sunny},
        },
    },
};
use iced_widget::{column, container, responsive, row, space};

fn card_item<'a>(
    theme: &(impl ColorScheme + ?Sized),
    shape: RoundedPolygon,
    icon: impl IntoFragment<'a>,
    title: impl IntoFragment<'a>,
    description: impl IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message> {
    let icon_shape = IconShape::new(shape, theme.primary(), icon, theme.on_primary())
        .size(48.0)
        .icon_size(24.0);
    let text = column![
        bold_text(title)
            .size(font::SIZE_LARGE)
            .color(theme.on_surface())
            .wrapping(iced_core::text::Wrapping::None),
        font::text(description)
            .color(theme.on_surface_variant())
            .wrapping(iced_core::text::Wrapping::None)
    ]
    .spacing(4);
    let content = row![
        icon_shape,
        text,
        space().width(Length::Fill),
        icon_filled(ARROW_FORWARD_IOS.into_fragment(), 20.0).color(theme.on_surface_variant())
    ]
    .spacing(18.0)
    .align_y(Alignment::Center);

    let style = card::Style {
        border: Border::default().rounded(ROUNDING_LARGE),
        ..card::Style::elevated(theme)
    };
    card(style, content)
        .interaction(card::Interaction::Press(OnPress::Direct(on_press)))
        .width(Length::Fill)
        .into()
}

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let cards = container(responsive(|size| {
        let favorites_card = card_item(
            &state.theme,
            heart(),
            *MUSIC_NOTE,
            "Favorite tracks",
            "No favorites!",
            Message::Noop,
        );

        let playlist_text = if let Some(lib) = &state.library
            && !lib.playlists.is_empty()
        {
            format!("Explore your {} playlists", lib.playlists.len())
        } else {
            "No playlists yet!".to_string()
        };
        let playlist_card = card_item(
            &state.theme,
            cookie4(),
            *LIBRARY_MUSIC,
            "Your playlists",
            playlist_text,
            Message::Noop,
        );

        let eq_card = card_item(
            &state.theme,
            sunny(),
            *EQUALIZER,
            "Equalizer",
            "Adjust how your music sounds",
            Message::Noop,
        );

        let statistics_card = card_item(
            &state.theme,
            bun(),
            *TRENDING_UP,
            "Statistics",
            "See your listening statistics",
            Message::Noop,
        );

        // Eyeballed it
        let compact = size.width < 780.0;

        match compact {
            true => column![favorites_card, playlist_card, statistics_card, eq_card]
                .spacing(card::MAX_CARD_BETWEEN_PADDING),
            false => column![
                row![favorites_card, playlist_card].spacing(card::MAX_CARD_BETWEEN_PADDING),
                row![statistics_card, eq_card].spacing(card::MAX_CARD_BETWEEN_PADDING)
            ]
            .spacing(card::MAX_CARD_BETWEEN_PADDING),
        }
        .into()
    }))
    .padding(padding::horizontal(SPACING_LARGE));

    cards.into()
}
