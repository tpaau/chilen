use iced::Element;
use iced_widget::column;

use crate::gui::{
    Chilen,
    event_loop::Message,
    settings::{Screen, TITLE_CONTENT_SPACE, title},
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    column![title(Screen::Library.label(), &state.theme),]
        .spacing(TITLE_CONTENT_SPACE)
        .into()
}
