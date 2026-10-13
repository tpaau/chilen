use iced::Element;
use iced_widget::column;

use crate::gui::{
    Chilen,
    settings::{Message, Screen, TITLE_CONTENT_SPACE, title},
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    column![title(Screen::Playback.label(), &state.theme),]
        .spacing(TITLE_CONTENT_SPACE)
        .into()
}
