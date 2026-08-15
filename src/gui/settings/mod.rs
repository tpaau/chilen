use iced::{Border, Element, Length, Task, color};
use iced_m3::{style::shadow, theme::ColorScheme};
use iced_widget::{center, container, opaque};

use crate::gui::{Chilen, ROUNDING_REGULAR, SPACING_SMALLER};

#[derive(Debug, Clone)]
pub enum Message {
    Close,
}

pub(super) fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::Close => state.dialog = super::dialog::Dialog::None,
    }
    Task::none()
}

pub(super) fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let content = iced_m3::widget::button(&state.theme)
        .label("close")
        .on_press(Message::Close);

    opaque(
        container(
            container(center(content))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(SPACING_SMALLER)
                .style(|_| {
                    container::Style::default()
                        .background(state.theme.surface())
                        .border(Border::default().rounded(ROUNDING_REGULAR))
                        .shadow(shadow(state.theme.shadow(), 0.7))
                }),
        )
        .style(|_| container::Style::default().background(color!(0x000000).scale_alpha(0.3)))
        .padding(64),
    )
}
