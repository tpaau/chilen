use iced::Element;
use iced_widget::text;

use crate::gui::{Chilen, playback_view::Message};

pub(super) fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    text("I am lyrics").into()
}
