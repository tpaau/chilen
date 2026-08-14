use std::sync::Arc;

use chilen_backend::music_lib::state::{Artist, MusicLibrary};
use iced::{Element, Length, widget::space};
use iced_widget::{column, sensor};

use crate::gui::{
    Chilen,
    main_view::{
        self,
        top_view::{self, TopView},
    },
    widget::{
        self,
        list::{BUTTON_HEIGHT, BUTTON_SPACING},
    },
};

pub fn artist_button<'a>(
    state: &'a Chilen,
    index: usize,
    artist: &'a Arc<Artist>,
) -> Element<'a, main_view::Message> {
    let content: Element<'a, main_view::Message> = if let Some(visible) = &state.main_view.visible
        && index < visible.len()
        && visible[index]
    {
        widget::list::artist_button::artist_button(&state.theme, artist.clone())
            .on_press_with(|| {
                main_view::Message::TopView(top_view::Message::Navigate(TopView::Artist(
                    artist.clone(),
                )))
            })
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
        lib.artists
            .iter()
            .enumerate()
            .map(|(i, a)| artist_button(state, i, a)),
    )
    .spacing(BUTTON_SPACING);

    iced_widget::scrollable(content)
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
