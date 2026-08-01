use iced::{Element, Length};
use iced_widget::{column, text};

use crate::gui::{Chilen, Message, SPACING_SMALLER, widgets::track_button::track_button};

pub fn view(state: &Chilen) -> Element<'_, Message> {
    let lib = if let Some(lib) = &state.library {
        lib
    } else {
        return text("Loading...").into();
    };

    iced::widget::scrollable(
        column(
            lib.tracks
                .iter()
                .map(|t| track_button(state, t).into())
                .collect::<Vec<_>>(),
        )
        .spacing(SPACING_SMALLER),
    )
    .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}
