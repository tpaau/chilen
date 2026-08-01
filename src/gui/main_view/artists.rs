use iced::Element;
use iced_widget::text;

use crate::gui::{Chilen, Message};

pub fn view(chilen: &Chilen) -> Element<'_, Message> {
    text("Artists").into()
}
