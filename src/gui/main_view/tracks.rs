use std::sync::Arc;

use chilen_backend::music_lib::state::{MusicLibrary, Track};
use iced::{Element, Length, widget::space};
use iced_widget::{column, sensor};

use crate::gui::{
    Chilen, main_view,
    widget::{
        self,
        list::{BUTTON_HEIGHT, BUTTON_SPACING},
    },
};

pub fn track_button<'a>(
    state: &'a Chilen,
    index: usize,
    track: &'a Arc<Track>,
) -> Element<'a, main_view::Message> {
    let content: Element<'a, main_view::Message> = if let Some(visible) = &state.main_view.visible
        && index < visible.len()
        && visible[index]
    {
        widget::list::track_button::track_button(state, track.clone())
            .on_press(main_view::Message::Noop)
            .into()
    } else {
        space().height(BUTTON_HEIGHT).width(Length::Fill).into()
    };
    sensor(content)
        .on_show(move |_| main_view::Message::ButtonPoppedIn(index))
        .on_hide(main_view::Message::ButtonPoppedOut(index))
        .into()
}

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, main_view::Message> {
    let content = column(
        lib.tracks
            .iter()
            .enumerate()
            .map(|(i, t)| track_button(state, i, t)),
    )
    .spacing(BUTTON_SPACING);

    iced_widget::scrollable(content)
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
