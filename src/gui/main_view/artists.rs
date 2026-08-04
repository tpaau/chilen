use iced::Element;
use iced_widget::text;

use crate::gui::{Chilen, Message};

pub fn view(state: &Chilen) -> Element<'_, Message> {
    text("Artists").into()
}
