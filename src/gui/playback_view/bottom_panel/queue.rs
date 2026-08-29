use iced::Element;
use iced_m3::theme::ColorScheme;
use iced_widget::{container, text};

use crate::gui::{Chilen, playback_view::Message};

pub(super) fn view<'a>(state: &'a Chilen, additional_padding: f32) -> Element<'a, Message> {
    container(text("I am queue").color(state.theme.on_surface()))
        .padding(additional_padding)
        .into()
}
