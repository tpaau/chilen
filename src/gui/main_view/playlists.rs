use std::ops::Not;

use chilen_backend::music_lib::{LibraryItem, is_favorite};
use chilen_widget::virtual_list::VirtualList;
use iced::{Element, Length, padding, widget::column};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::{Accent, ColorScheme},
    widget::{Action, fab_menu, hybrid_icon::Icon, shapes},
};
use iced_widget::{bottom_right, center, space, stack};

use crate::gui::{
    Chilen, SPACING_SMALL, icons,
    main_view::{
        Message, TOOLBAR_OFFSET,
        top_view::{self, TopView},
    },
    widget::{
        icon_shape::IconShape,
        list::{
            BUTTON_HEIGHT, BUTTON_SPACING,
            playlist_button::{self, PlaylistButton},
        },
    },
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let fab = {
        bottom_right(fab_menu(
            fab_menu::Style::new(&state.theme, Accent::default()),
            vec![
                iced_m3::widget::fab_menu::Item {
                    on_press: Action::Direct(Message::ImportPlaylist),
                    label: "Import playlist".into(),
                    icon: Icon::Text {
                        text: icons::UPLOAD_FILE.into_fragment(),
                        font: Some(icons::filled()),
                    },
                },
                iced_m3::widget::fab_menu::Item {
                    on_press: Action::Direct(Message::CreatePlaylist),
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
        && !lib.playlists.is_empty()
    {
        let highlighted_name = state.player_state.as_ref().and_then(|p| {
            if let chilen_backend::playback::QueueSource::Playlist { name } = &p.queue_source {
                Some(name)
            } else {
                None
            }
        });

        let content = VirtualList {
            model: lib.playlists.iter().enumerate(),
            delegate: Box::new(move |(index, playlist)| {
                let favorite_item: LibraryItem = playlist.clone().into();
                let is_favorite = is_favorite(&favorite_item).unwrap_or_default();

                PlaylistButton {
                    state,
                    playlist,
                    press: Message::TopView(top_view::Message::Navigate(TopView::Playlist(
                        playlist.clone(),
                    ))),
                    toggle_menu: Message::ToggleMenu(index),
                    messages: playlist_button::Messages {
                        play: playlist
                            .tracks
                            .is_empty()
                            .not()
                            .then_some(Message::PlayPlaylist(playlist.clone())),
                        shuffle: playlist
                            .tracks
                            .is_empty()
                            .not()
                            .then_some(Message::ShufflePlaylist(playlist.clone())),
                        add_to_queue: playlist
                            .tracks
                            .is_empty()
                            .not()
                            .then_some(Message::AddPlaylistToQueue(playlist.clone())),
                        export: playlist.tracks.is_empty().not().then_some(
                            Message::ExportPlaylist(playlist.name.clone().into_string()),
                        ),
                        toggle_favorite: Message::ToggleFavorite(favorite_item),
                        rename: Message::OpenPlaylistRenameDialog {
                            playlist: playlist.name.clone().into_string(),
                            name: playlist.name.clone().into_string(),
                        },
                        delete: Message::ConfirmPlaylistDeletion(playlist.clone()),
                    },
                    highlighted: highlighted_name
                        .map(|name| name.as_str() == playlist.name.as_ref())
                        .unwrap_or_default(),
                    is_favorite,
                    menu_opened: state.main_view.button_menu == Some(index),
                }
                .into()
            }),
            delegate_height: BUTTON_HEIGHT,
            visibilities: state.main_view.visible.as_deref().unwrap_or(&[]),
            list: Box::new(|content| column(content).spacing(BUTTON_SPACING).into()),
            on_show: Box::new(Message::ButtonPoppedIn),
            on_hide: Box::new(Message::ButtonPoppedOut),
        };

        stack![
            iced_widget::scrollable(column![content, space().height(*TOOLBAR_OFFSET)])
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_, status| iced_m3::style::scrollable(status, &state.theme)),
            fab
        ]
        .into()
    } else {
        stack![
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
            ),
            fab
        ]
        .into()
    }
}
