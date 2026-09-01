use chilen_backend::music_lib::MusicLibrary;
use iced::Element;
use iced_widget::column;

use crate::gui::{
    Chilen,
    main_view::{
        Message,
        top_view::{self, TopView},
    },
    widget::{
        self,
        list::{BUTTON_HEIGHT, BUTTON_SPACING},
        virtual_list::VirtualList,
    },
};

pub fn view<'a>(state: &'a Chilen, lib: &'a MusicLibrary) -> Element<'a, Message> {
    let highlighted_artist_name = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Artist { name } = &p.queue_source {
            Some(name)
        } else {
            None
        }
    });

    let content = VirtualList {
        model: lib.artists.iter(),
        delegate: Box::new(move |artist| {
            let highlighted = highlighted_artist_name
                .map(|name| *name == artist.name)
                .unwrap_or_default();
            widget::list::artist_button::artist_button(
                &state.theme,
                artist.clone(),
                Message::PlayArtist(artist.clone()),
                Message::ShuffleArtist(artist.clone()),
                highlighted,
            )
            .on_press_with(|| {
                Message::TopView(top_view::Message::Navigate(TopView::Artist(artist.clone())))
            })
            .into()
        }),
        delegate_height: BUTTON_HEIGHT,
        visibilities: state.main_view.visible.as_deref().unwrap_or(&[]),
        list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
        on_show: Box::new(Message::ButtonPoppedIn),
        on_hide: Box::new(Message::ButtonPoppedOut),
    };

    iced_widget::scrollable(content)
        .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
        .into()
}
