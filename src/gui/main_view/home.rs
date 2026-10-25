use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_LARGE, SPACING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
    font::{self, bold_text, text},
    formatter::{
        format_album_count, format_album_track_count, format_artist_count, format_date_maybe,
        format_playlist_track_count, format_track_count, format_track_duration,
    },
    icons::{self, icon_filled},
    main_view::{Message, NavTab, top_view},
    widget::{
        cover_image::CoverImage, icon_shape::IconShape, text_spacer::text_spacer, wrappers::marquee,
    },
};
use chilen_backend::music_lib::LibraryItem;
use iced::{Alignment, Border, Element, Length, padding};
use iced_core::text::{self, IntoFragment};
use iced_m3::{
    theme::ColorScheme,
    widget::{
        OnPress, card,
        shapes::polymorpher::{
            RoundedPolygon,
            shapes::{clover8, cookie4, cookie12},
        },
        spacer,
    },
};
use iced_widget::{column, container, responsive, row, scrollable, space, stack};

// TODO: Should be a carousel!
fn card_list<'a>(
    items: impl IntoIterator<Item = Element<'a, Message>>,
    theme: &'a impl ColorScheme,
) -> Element<'a, Message> {
    scrollable(row(items).spacing(card::MAX_CARD_BETWEEN_PADDING))
        .style(|_, status| iced_m3::style::scrollable(status, theme))
        .horizontal()
        .into()
}

fn library_item_card<'a>(state: &'a Chilen, item: LibraryItem) -> Element<'a, Message> {
    let (title, cover, icon, info, kind, on_press) = match item {
        LibraryItem::Track(track) => (
            track
                .title
                .as_ref()
                .map(|b| b.as_ref())
                .unwrap_or("Track")
                .to_string(),
            track.cover.cover(),
            *icons::MUSIC_NOTE,
            {
                let mut info = Vec::new();

                if let Some(artists) = &track.artists {
                    info.push(artists.join(&state.settings.value_separator))
                }
                info.push(format_track_duration(track.duration));

                info
            },
            "Track".to_string(),
            Message::PlayTrack(track.clone()),
        ),
        LibraryItem::Playlist(playlist) => (
            playlist.name.clone().into_string(),
            playlist.cover.cover(),
            *icons::LIBRARY_MUSIC,
            vec![format_playlist_track_count(playlist.tracks.len())],
            "Playlist".to_string(),
            Message::TopView(top_view::Message::Navigate(playlist.clone().into())),
        ),
        LibraryItem::Album(album) => (
            album.title.clone().into_string(),
            album.cover.cover(),
            *icons::ALBUM,
            vec![
                format_album_track_count(album.tracks.len()),
                format_date_maybe(album.date),
            ],
            "Album".to_string(),
            Message::TopView(top_view::Message::Navigate(album.clone().into())),
        ),
        LibraryItem::Artist(artist) => (
            artist.name.clone().into_string(),
            artist.cover.cover(),
            *icons::ARTIST,
            vec![
                format_album_count(artist.albums.len()),
                format_track_count(artist.tracks.len()),
            ],
            "Artist".to_string(),
            Message::TopView(top_view::Message::Navigate(artist.clone().into())),
        ),
        LibraryItem::Genre(genre) => (
            genre.name.clone().into_string(),
            genre.cover.cover(),
            *icons::GENRES,
            vec![
                format_artist_count(genre.artists.len()),
                format_track_count(genre.tracks.len()),
            ],
            "Genre".to_string(),
            Message::TopView(top_view::Message::Navigate(genre.clone().into())),
        ),
    };

    let cover_size = 300.0;
    let rounding = ROUNDING_LARGE;

    let cover = CoverImage {
        image_path: cover.ok(),
        icon,
        icon_size: cover_size / 2.0,
        icon_color: state.theme.on_surface_variant(),
        container_color: state.theme.surface_container(),
        radius: rounding.into(),
        opacity: 1.0,
        width: cover_size.into(),
        height: cover_size.into(),
    };

    let chip = container(
        row![
            icon_filled(icon, icons::SIZE_SMALL).color(state.theme.on_tertiary_container()),
            text(kind)
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_tertiary_container())
        ]
        .spacing(SPACING_SMALLER),
    )
    .style(|_| {
        container::Style::default()
            .background(state.theme.tertiary_container())
            .border(Border::default().rounded(f32::MAX))
    })
    .padding(SPACING_SMALLER);

    let cover = stack![cover, container(chip).padding(SPACING_SMALLER)];

    let title = marquee(bold_text(title).size(32).color(state.theme.on_surface()));

    let info = {
        let mut info_text = Vec::with_capacity(info.len().saturating_mul(2).saturating_sub(1));
        let font_size = font::SIZE_REGULAR;
        let len = info.len();
        for (i, info) in info.into_iter().enumerate() {
            let text = iced_widget::text(info.clone())
                .size(font_size)
                .color(state.theme.on_surface_variant())
                .wrapping(text::Wrapping::None);

            info_text.push(text.into());

            if i != len - 1 {
                info_text.push(text_spacer(state.theme.on_surface_variant(), font_size));
            }
        }

        marquee(
            row(info_text)
                .spacing(SPACING_SMALLER)
                .align_y(Alignment::Center),
        )
    };

    let bottom = column![title, info]
        .padding(SPACING_LARGE)
        .spacing(SPACING_SMALL / 2.0);

    let style = card::Style {
        border: Border::default().rounded(rounding),
        ..card::Style::elevated(&state.theme)
    };
    card(style, column![cover, bottom])
        .interaction(card::Interaction::Press(OnPress::Direct(on_press)))
        .padding(0.0.into())
        .width(cover_size)
        .into()
}

fn title<'a>(
    theme: &(impl ColorScheme + ?Sized),
    shape: RoundedPolygon,
    icon: impl IntoFragment<'a>,
    title: impl IntoFragment<'a>,
) -> Element<'a, Message> {
    row![
        IconShape::new(shape, theme.secondary(), icon, theme.on_secondary())
            .size(46.0)
            .icon_size(28.0),
        bold_text(title).size(30),
        spacer(theme.outline_variant())
    ]
    .spacing(SPACING_REGULAR)
    .align_y(Alignment::Center)
    .into()
}

fn card_item<'a>(
    theme: &(impl ColorScheme + ?Sized),
    icon: impl IntoFragment<'a>,
    title: impl IntoFragment<'a>,
    description: impl IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message> {
    let icon_shape = IconShape::new(cookie4(), theme.primary(), icon, theme.on_primary())
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
        icon_filled(icons::ARROW_FORWARD_IOS.into_fragment(), 20.0)
            .color(theme.on_surface_variant())
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
    let cards = responsive(|size| {
        let shuffle_card = card_item(
            &state.theme,
            *icons::SHUFFLE,
            "Shuffle",
            "Shuffle your tracks",
            Message::ShuffleTracks {
                initial_position: None,
            },
        );

        // TODO: Make this one be a playlist creation button if no playlists are present in the library
        let playlist_text = if let Some(lib) = &state.library
            && !lib.playlists.is_empty()
        {
            format!("Explore your {} playlists", lib.playlists.len())
        } else {
            "No playlists yet!".to_string()
        };
        let playlist_card = card_item(
            &state.theme,
            *icons::LIBRARY_MUSIC,
            "Your playlists",
            playlist_text,
            Message::SwitchTab(NavTab::Playlists),
        );

        let eq_card = card_item(
            &state.theme,
            *icons::EQUALIZER,
            "Equalizer",
            "Adjust how your music sounds",
            Message::Noop,
        );

        let statistics_card = card_item(
            &state.theme,
            *icons::TRENDING_UP,
            "Statistics",
            "See your listening statistics",
            Message::Noop,
        );

        // Eyeballed it
        let compact = size.width < 780.0;

        match compact {
            true => column![shuffle_card, playlist_card, statistics_card, eq_card]
                .spacing(card::MAX_CARD_BETWEEN_PADDING),
            false => column![
                row![shuffle_card, playlist_card].spacing(card::MAX_CARD_BETWEEN_PADDING),
                row![statistics_card, eq_card].spacing(card::MAX_CARD_BETWEEN_PADDING)
            ]
            .spacing(card::MAX_CARD_BETWEEN_PADDING),
        }
        .into()
    })
    .height(Length::Shrink);

    let recent_title = title(&state.theme, cookie12(), *icons::RESTART_ALT, "Recent");

    let recent_cards = card_list(
        state
            .persistent
            .recent_items()
            .iter()
            .map(|item| library_item_card(state, item.clone().into())),
        &state.theme,
    );

    let recent = column![recent_title, recent_cards].spacing(SPACING_REGULAR);

    let favorites_title = title(&state.theme, clover8(), *icons::STAR, "Favorites");

    let favorite_cards = state
        .library
        .as_ref()
        .map(|lib| {
            lib.favorites
                .iter()
                .take(8)
                .map(|favorite| library_item_card(state, favorite.clone()))
        })
        .map(|i| card_list(i, &state.theme));

    let favorites = column![favorites_title, favorite_cards].spacing(SPACING_REGULAR);

    scrollable(
        column![column![space().height(16), cards], recent, favorites]
            .spacing(42)
            .padding(padding::horizontal(SPACING_REGULAR)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
    .into()
}
