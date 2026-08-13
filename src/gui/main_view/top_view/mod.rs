use std::sync::Arc;

use chilen_backend::music_lib::state::{Album, Artist, Genre, Playlist};
use iced::{Element, Length, Task};
use iced_m3::theme::ColorScheme;
use iced_widget::{center, container, stack, text};

use crate::gui::{Chilen, icons};

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

fn unwind_button(theme: &impl ColorScheme) -> Element<'_, Message> {
    container(
        iced_m3::widget::button(theme)
            .style(iced_m3::widget::button::Style::Tonal(
                iced_m3::theme::Accent::Tertiary,
            ))
            .label_maybe(None)
            .icon(&icons::ARROW_BACK)
            .icon_font(icons::filled())
            .on_press(Message::Unwind),
    )
    .align_top(Length::Fill)
    .align_left(Length::Fill)
    .into()
}

pub fn view(state: &Chilen, view: TopView) -> Element<'_, Message> {
    match view {
        TopView::Playlist(playlist) => stack![
            unwind_button(&state.theme),
            center(text(playlist.name.clone()))
        ],
        TopView::Album(album) => stack![
            unwind_button(&state.theme),
            center(text(album.title.clone()))
        ],
        TopView::Artist(artist) => stack![
            unwind_button(&state.theme),
            center(text(artist.name.clone()))
        ],
        TopView::Genre(genre) => stack![
            unwind_button(&state.theme),
            center(text(genre.name.clone()))
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
