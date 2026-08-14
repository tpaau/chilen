use std::{sync::Arc, time::Duration};

use chilen_backend::music_lib::state::{Album, Artist, Genre, Playlist};
use iced::{Alignment, Element, Length, Task, border::Radius};
use iced_m3::theme::ColorScheme;
use iced_widget::{center, column, container, responsive, row, rule, scrollable, space, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALL, SPACING_SMALLER, font, icons,
    widget::{
        self, artist_chip::artist_chip, cover_image::cover_image, list::BUTTON_SPACING,
        text_spacer::text_spacer,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopView {
    Playlist(Arc<Playlist>),
    Album(Arc<Album>),
    Artist(Arc<Artist>),
    Genre(Arc<Genre>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Noop,
    Navigate(TopView),
    Unwind,
}

fn unwind_button(theme: &impl ColorScheme) -> Element<'_, Message> {
    iced_m3::widget::button(theme)
        .size(iced_m3::widget::button::Size::Small.with_height(Length::Fill))
        .style(iced_m3::widget::button::Style::Tonal(
            iced_m3::theme::Accent::Tertiary,
        ))
        .corner_style(iced_m3::widget::button::CornerStyle::Square)
        .label_maybe(None)
        .icon(&icons::ARROW_BACK)
        .icon_font(icons::filled())
        .elevation(0.2)
        .on_press(Message::Unwind)
        .into()
}

fn title<'a>(theme: &'a impl ColorScheme, content: String) -> Element<'a, Message> {
    text(content)
        .size(32.0)
        .font(font::font_bold())
        .color(theme.on_surface())
        .into()
}

fn format_duration(d: Duration) -> String {
    let total_minutes = d.as_secs() / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if hours > 0 {
        if minutes == 0 {
            format!("{hours} hr")
        } else {
            format!("{hours} hr {minutes} min")
        }
    } else {
        format!("{minutes} min")
    }
}

const MAX_COVER_SIZE: f32 = 512.0;
const MIN_COVER_SIZE: f32 = 192.0;

pub fn view(state: &Chilen, view: TopView) -> Element<'_, Message> {
    let content = match view {
        TopView::Playlist(playlist) => column![center(text(playlist.name.clone())),],
        TopView::Album(album) => {
            let album_cloned = album.clone();
            let display = responsive(move |size| {
                let song_count_text = if album_cloned.tracks.len() == 1 {
                    "1 track".to_string()
                } else {
                    format!("{} tracks", album_cloned.tracks.len())
                };
                let cover_size =
                    (size.width.min(size.height) / 3.0).clamp(MIN_COVER_SIZE, MAX_COVER_SIZE);

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
                        text("Album")
                            .size(font::SIZE_REGULAR)
                            .color(state.theme.on_surface_variant()),
                        album_cloned.date.map(|_| text_spacer(
                            state.theme.on_surface_variant(),
                            font::SIZE_REGULAR
                        )),
                        album_cloned.date.map(|date| text(date.year)
                            .size(font::SIZE_REGULAR)
                            .color(state.theme.on_surface_variant()))
                    ]
                    .spacing(SPACING_SMALLER)
                    .align_y(Alignment::Center),
                    title(&state.theme, album_cloned.title.clone()),
                    row![
                        text(song_count_text)
                            .size(font::SIZE_LARGE)
                            .color(state.theme.on_surface()),
                        text_spacer(state.theme.on_surface_variant(), font::SIZE_LARGE),
                        text(format_duration(album_cloned.total_duration))
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

            let main_buttons = row![
                iced_m3::widget::button(&state.theme)
                    .size(iced_m3::widget::button::Size::Medium.with_width(Length::Fill))
                    .icon_font(icons::filled())
                    .icon(&icons::PLAY_ARROW)
                    .label("Play")
                    .style(iced_m3::widget::button::Style::Tonal(
                        iced_m3::theme::Accent::Secondary
                    ))
                    .on_press(Message::Noop),
                iced_m3::widget::button(&state.theme)
                    .size(iced_m3::widget::button::Size::Medium.with_width(Length::Fill))
                    .icon_font(icons::filled())
                    .icon(&icons::SHUFFLE)
                    .label("Shuffle")
                    .style(iced_m3::widget::button::Style::Filled(
                        iced_m3::theme::Accent::Primary
                    ))
                    .on_press(Message::Noop),
                iced_m3::widget::button(&state.theme)
                    .size(iced_m3::widget::button::Size::Medium)
                    .icon_font(icons::filled())
                    .style(iced_m3::widget::button::Style::Outlined)
                    .icon(&icons::MORE_HORIZ)
                    .label_maybe(None)
                    .on_press(Message::Noop),
            ]
            .spacing(SPACING_REGULAR);

            let track_buttons = album.tracks.iter().map(|t| {
                widget::list::track_button::track_button(state, t.clone())
                    .on_press(Message::Noop)
                    .into()
            });

            column![
                row![
                    container(unwind_button(&state.theme)).width(Length::Fixed(50.0)),
                    column![display, main_buttons].spacing(SPACING_REGULAR)
                ]
                .spacing(SPACING_REGULAR),
                rule::horizontal(1.0).style(|_| rule::Style {
                    color: state.theme.outline_variant(),
                    radius: Radius::default(),
                    fill_mode: rule::FillMode::Full,
                    snap: true
                }),
                column(track_buttons).spacing(BUTTON_SPACING)
            ]
            .width(Length::Fill)
            .spacing(SPACING_REGULAR)
        }
        TopView::Artist(artist) => column![center(text(artist.name.clone())),],
        TopView::Genre(genre) => column![center(text(genre.name.clone())),],
    };

    container(
        scrollable(content).style(|_, status| iced_m3::style::scrollable(status, &state.theme)),
    )
    .align_top(Length::Fill)
    .into()
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::Navigate(top_view) => state.main_view.nav_stack.navigate(top_view),
        Message::Unwind => {
            state.main_view.nav_stack.unwind();
        }
        Message::Noop => {
            // TODO: This is a placeholder
        }
    }
    Task::none()
}
