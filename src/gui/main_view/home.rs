use iced::Element;

use crate::gui::{Chilen, main_view::Message};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    iced_widget::text("Hello!").into()
}
