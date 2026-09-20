use iced::Element;
use iced_widget::space;

use crate::gui::{Chilen, settings::Message};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    space().into()
}
