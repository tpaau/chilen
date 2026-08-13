use std::sync::Arc;

use chilen_backend::music_lib::state::{Album, Artist, Genre, Playlist};
use iced::{Alignment, Element, Length, Task, border::Radius};
use iced_m3::theme::ColorScheme;
use iced_widget::{center, column, container, responsive, row, rule, text};

use crate::gui::{Chilen, SPACING_REGULAR, font, icons, widget::cover_image::cover_image};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopView {
    Playlist(Arc<Playlist>),
    Album(Arc<Album>),
    Artist(Arc<Artist>),
    Genre(Arc<Genre>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Navigate(TopView),
    Unwind,
}

const SPACING: f32 = SPACING_REGULAR;

fn unwind_button(theme: &impl ColorScheme) -> Element<'_, Message> {
    iced_m3::widget::button(theme)
        .style(iced_m3::widget::button::Style::Tonal(
            iced_m3::theme::Accent::Tertiary,
        ))
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

const MAX_COVER_SIZE: f32 = 512.0;
const MIN_COVER_SIZE: f32 = 192.0;

pub fn view(state: &Chilen, view: TopView) -> Element<'_, Message> {
    match view {
        TopView::Playlist(playlist) => column![
            unwind_button(&state.theme),
            center(text(playlist.name.clone())),
        ],
        TopView::Album(album) => column![
            unwind_button(&state.theme),
            responsive(move |size| {
                let cover_size =
                    (size.width.min(size.height) / 3.0).clamp(MIN_COVER_SIZE, MAX_COVER_SIZE);
                row![
                    cover_image(
                        album.cover.hires.clone(),
                        &icons::ALBUM,
                        state.theme.on_surface_variant(),
                        state.theme.surface_container(),
                        12.0
                    )
                    .width(Length::Fixed(cover_size))
                    .height(Length::Fixed(cover_size)),
                    container(
                        column![
                            text("Album")
                                .size(font::SIZE_REGULAR)
                                .color(state.theme.on_surface_variant()),
                            title(&state.theme, album.title.clone())
                        ]
                        .align_x(Alignment::Start)
                    )
                ]
                .align_y(Alignment::Center)
                .spacing(SPACING)
                .into()
            })
            .height(Length::Shrink),
            row![
                iced_m3::widget::button(&state.theme)
                    .size(iced_m3::widget::button::Size::Medium)
                    .icon_font(icons::filled())
                    .icon(&icons::PLAY_ARROW)
                    .label("Play"),
                iced_m3::widget::button(&state.theme)
                    .size(iced_m3::widget::button::Size::Medium)
                    .icon_font(icons::filled())
                    .icon(&icons::SHUFFLE)
                    .label("Shuffle")
            ]
            .spacing(SPACING_REGULAR),
            rule::horizontal(1.0).style(|_| rule::Style {
                color: state.theme.outline_variant(),
                radius: Radius::default(),
                fill_mode: rule::FillMode::Full,
                snap: true
            }),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(SPACING),
        TopView::Artist(artist) => column![
            unwind_button(&state.theme),
            center(text(artist.name.clone())),
        ],
        TopView::Genre(genre) => column![
            unwind_button(&state.theme),
            center(text(genre.name.clone())),
        ],
    }
    .into()
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::Navigate(top_view) => state.main_view.nav_stack.navigate(top_view),
        Message::Unwind => {
            state.main_view.nav_stack.unwind();
        }
    }
    Task::none()
}
