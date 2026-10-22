use chilen_widget::virtual_list::VirtualList;
use iced::{Element, padding, widget::column};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::{Accent, ColorScheme},
    widget::{OnPress, fab_menu, hybrid_icon::Icon, shapes},
};
use iced_widget::{bottom_right, center, stack};

use crate::gui::{
    Chilen, SPACING_SMALL, icons,
    main_view::Message,
    widget::{
        icon_shape::IconShape,
        list::{BUTTON_HEIGHT, BUTTON_SPACING, playlist_button::playlist_button},
    },
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let fab = {
        bottom_right(fab_menu(
            fab_menu::Style::new(&state.theme, Accent::default()),
            vec![
                iced_m3::widget::fab_menu::Item {
                    on_press: OnPress::Direct(Message::ImportPlaylist),
                    label: "Import playlist".into(),
                    icon: Icon::Text {
                        text: icons::UPLOAD_FILE.into_fragment(),
                        font: Some(icons::filled()),
                    },
                },
                iced_m3::widget::fab_menu::Item {
                    on_press: OnPress::Direct(Message::CreatePlaylist),
                    label: "New playlist".into(),
                    icon: Icon::Text {
                        text: icons::PLAYLIST_ADD.into_fragment(),
                        font: Some(icons::filled()),
                    },
                },
            ],
            None,
        ))
        .padding(padding::bottom(SPACING_SMALL))
    };

    if let Some(lib) = &state.library
        && !lib.tracks.is_empty()
    {
        let highlighted_index = state.player_state.as_ref().and_then(|p| {
            if p.queue_source == chilen_backend::playback::QueueSource::AllTracks {
                p.real_track_index(p.position)
            } else {
                None
            }
        });

        let content = VirtualList {
            model: lib.playlists.iter().enumerate(),
            delegate: Box::new(move |(index, playlist)| {
                playlist_button(
                    state,
                    playlist,
                    index,
                    highlighted_index
                        .map(|highlighted_index| highlighted_index == index)
                        .unwrap_or_default(),
                    state.settings.vibrant_widgets,
                )
            }),
            delegate_height: BUTTON_HEIGHT,
            visibilities: state.main_view.visible.as_deref().unwrap_or(&[]),
            list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
            on_show: Box::new(Message::ButtonPoppedIn),
            on_hide: Box::new(Message::ButtonPoppedOut),
        };

        stack![
            iced_widget::scrollable(content)
                .style(|_, status| iced_m3::style::scrollable(status, &state.theme)),
            fab
        ]
        .into()
    } else {
        center(
            IconShape::new(
                shapes::polymorpher::shapes::sunny(),
                state.theme.surface_container_high(),
                icons::MUSIC_NOTE.into_fragment(),
                state.theme.on_surface_variant(),
            )
            .label(
                "Your playlists will show up here",
                state.theme.on_surface_variant(),
            ),
        )
        .into()
    }
}
