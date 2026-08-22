use std::sync::Arc;

use chilen_backend::music_lib::Album;
use iced::{Alignment, Element, Length};
use iced_m3::theme::ColorScheme;
use iced_widget::{column, responsive, row, space, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALL, SPACING_SMALLER, font, icons,
    main_view::top_view::{
        MAX_COVER_SIZE, MIN_COVER_SIZE, Message, TopView, format_duration, horizontal_buttons,
        spacer, title,
    },
    widget::{
        self, artist_chip::artist_chip, cover_image::cover_image, list::BUTTON_SPACING,
        text_spacer::text_spacer,
    },
};

pub(super) fn view<'a>(state: &'a Chilen, album: Arc<Album>) -> Element<'a, Message> {
    let album_cloned = album.clone();

    let display = responsive(move |size| {
        let track_count_text = if album_cloned.tracks.len() == 1 {
            "1 track".to_string()
        } else {
            format!("{} tracks", album_cloned.tracks.len())
        };
        let cover_size = (size.width.min(size.height) / 3.0).clamp(MIN_COVER_SIZE, MAX_COVER_SIZE);

        let artist_chips = state.library.as_ref().map(|lib| {
            let mut artist_chips: Vec<Element<'_, Message>> =
                Vec::with_capacity(2 * album_cloned.artists.len() - 1);
            for (i, artist) in album_cloned.artists.iter().enumerate() {
                if let Some(artist) = lib.find_artist(artist) {
                    artist_chips.push(
                        artist_chip(&state.theme, artist.clone())
                            .on_press(Message::Navigate(TopView::Artist(artist.clone())))
                            .into(),
                    );
                    if let Some(len) = album_cloned.artists.len().checked_sub(1)
                        && i < len
                    {
                        artist_chips.push(text_spacer(
                            state.theme.on_surface_variant(),
                            font::SIZE_LARGE,
                        ));
                    }
                }
            }
            artist_chips
        });

        let cover = cover_image(
            album_cloned.cover.hires.clone(),
            &icons::ALBUM,
            cover_size / 4.0,
            state.theme.on_surface_variant(),
            state.theme.surface_container(),
            ROUNDING_LARGE,
        )
        .width(Length::Fixed(cover_size))
        .height(Length::Fixed(cover_size));

        let item_data = column![
            row![
                text(if album_cloned.tracks.len() == 1 {
                    "Single"
                } else {
                    "Album"
                })
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_surface_variant()),
                album_cloned
                    .date
                    .map(|_| text_spacer(state.theme.on_surface_variant(), font::SIZE_REGULAR)),
                album_cloned.date.map(|date| text(date.year)
                    .size(font::SIZE_REGULAR)
                    .color(state.theme.on_surface_variant()))
            ]
            .spacing(SPACING_SMALLER)
            .align_y(Alignment::Center),
            title(&state.theme, album_cloned.title.clone()),
            row![
                text(track_count_text)
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface()),
                text_spacer(state.theme.on_surface_variant(), font::SIZE_LARGE),
                text(format_duration(album_cloned.duration))
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface()),
            ]
            .align_y(Alignment::Center)
            .spacing(SPACING_SMALLER),
            space().height(Length::Fixed(SPACING_SMALL)),
            artist_chips.map(|c| row(c)
                .align_y(Alignment::Center)
                .spacing(SPACING_SMALLER)
                .wrap())
        ]
        .align_x(Alignment::Start);

        row![cover, item_data]
            .align_y(Alignment::Center)
            .spacing(SPACING_REGULAR)
            .into()
    })
    .height(Length::Shrink)
    .width(Length::Shrink);

    let buttons = horizontal_buttons(&state.theme, Message::Noop, Message::Noop, Message::Noop);

    let track_buttons = album.tracks.iter().map(|t| {
        widget::list::track_button::track_button(state, t.clone())
            .on_press(Message::Noop)
            .into()
    });

    column![
        display,
        buttons,
        spacer(&state.theme),
        column(track_buttons).spacing(BUTTON_SPACING)
    ]
    .width(Length::Fill)
    .spacing(SPACING_REGULAR)
    .into()
}
