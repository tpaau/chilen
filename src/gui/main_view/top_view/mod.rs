use std::sync::Arc;

use chilen_backend::music_lib::state::{Album, Artist, Genre, Playlist};
use iced::{Element, Task};

use crate::gui::Chilen;

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

pub fn view(state: &Chilen) -> Element<'_, Message> {
    todo!()
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
