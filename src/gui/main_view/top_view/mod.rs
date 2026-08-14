mod album;
mod artist;

use std::sync::Arc;

use chilen_backend::music_lib::state::{Album, Artist, Genre, Playlist};
use iced::{Element, Length, Task, border::Radius};
use iced_m3::theme::ColorScheme;
use iced_widget::{Row, Rule, center, column, container, row, rule, scrollable, text};

use crate::gui::{Chilen, SPACING_REGULAR, font, icons};

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

fn horizontal_buttons<'a, Message: 'a + Clone>(
    theme: &'a impl ColorScheme,
    message_play: Message,
    message_shuffle: Message,
    message_options: Message,
) -> Row<'a, Message> {
    row![
        iced_m3::widget::button(theme)
            .size(iced_m3::widget::button::Size::Medium.with_width(Length::Fill))
            .icon_font(icons::filled())
            .icon(&icons::PLAY_ARROW)
            .label("Play")
            .style(iced_m3::widget::button::Style::Tonal(
                iced_m3::theme::Accent::Secondary
            ))
            .on_press(message_play),
        iced_m3::widget::button(theme)
            .size(iced_m3::widget::button::Size::Medium.with_width(Length::Fill))
            .icon_font(icons::filled())
            .icon(&icons::SHUFFLE)
            .label("Shuffle")
            .style(iced_m3::widget::button::Style::Filled(
                iced_m3::theme::Accent::Primary
            ))
            .on_press(message_shuffle),
        iced_m3::widget::button(theme)
            .size(iced_m3::widget::button::Size::Medium)
            .icon_font(icons::filled())
            .style(iced_m3::widget::button::Style::Outlined)
            .icon(&icons::MORE_HORIZ)
            .label_maybe(None)
            .on_press(message_options),
    ]
    .spacing(SPACING_REGULAR)
}

fn spacer<'a>(theme: &'a impl ColorScheme) -> Rule<'a> {
    rule::horizontal(1.0).style(|_| rule::Style {
        color: theme.outline_variant(),
        radius: Radius::default(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    })
}

const MAX_COVER_SIZE: f32 = 512.0;
const MIN_COVER_SIZE: f32 = 192.0;

pub fn view(state: &Chilen, view: TopView) -> Element<'_, Message> {
    let content = match view {
        TopView::Playlist(playlist) => column![center(text(playlist.name.clone()))].into(),
        TopView::Album(album) => album::view(state, album),
        TopView::Artist(artist) => artist::view(state, artist),
        TopView::Genre(genre) => column![center(text(genre.name.clone()))].into(),
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
