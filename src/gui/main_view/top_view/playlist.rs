use std::sync::Arc;

use chilen_backend::music_lib::Playlist;
use iced::{Alignment, Element, Length};
use iced_m3::{theme::ColorScheme, widget::spacer};
use iced_widget::{column, container, responsive, row, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALLER, font,
    formatter::format_album_duration,
    icons,
    main_view::top_view::{MAX_COVER_SIZE, MIN_COVER_SIZE, Message, horizontal_buttons, title},
    widget::{
        cover_image::CoverImage,
        list::{
            BUTTON_SPACING,
            track_button::{self, TrackButton},
        },
        text_spacer::text_spacer,
    },
};

pub(super) fn view<'a>(state: &'a Chilen, playlist: Arc<Playlist>) -> Element<'a, Message> {
    let playlist_cloned = playlist.clone();
    let display = responsive(move |size| {
        let has_tracks = !playlist_cloned.tracks.is_empty();
        let track_count_text = if !has_tracks {
            "No tracks".to_string()
        } else if playlist_cloned.tracks.len() == 1 {
            "1 track".to_string()
        } else {
            format!("{} tracks", playlist_cloned.tracks.len())
        };
        let cover_size = (size.width.min(size.height) / 3.0).clamp(MIN_COVER_SIZE, MAX_COVER_SIZE);

        let cover = CoverImage {
            image_path: playlist_cloned.cover.hires.clone(),
            icon: *icons::PLAYLIST_PLAY,
            icon_size: cover_size / 4.0,
            icon_color: state.theme.on_surface_variant(),
            container_color: state.theme.surface_container(),
            radius: ROUNDING_LARGE.into(),
            opacity: 1.0,
            width: cover_size.into(),
            height: cover_size.into(),
        };

        let item_data = column![
            text("Playlist")
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_surface_variant()),
            title(&state.theme, playlist_cloned.name.clone()),
            row![
                text(track_count_text)
                    .size(font::SIZE_LARGE)
                    .color(state.theme.on_surface()),
                has_tracks.then_some(text_spacer(
                    state.theme.on_surface_variant(),
                    font::SIZE_LARGE
                )),
                has_tracks.then_some(
                    text(format_album_duration(playlist_cloned.duration))
                        .size(font::SIZE_LARGE)
                        .color(state.theme.on_surface())
                )
            ]
            .align_y(Alignment::Center)
            .spacing(SPACING_SMALLER)
            .wrap(),
        ]
        .align_x(Alignment::Start);

        row![cover, item_data]
            .align_y(Alignment::Center)
            .spacing(SPACING_REGULAR)
            .wrap()
            .into()
    })
    .height(Length::Shrink)
    .width(Length::Shrink);

    let buttons = horizontal_buttons(
        &state.theme,
        (!playlist.tracks.is_empty()).then_some(Message::PlayPlaylistNoShuffle {
            playlist: playlist.clone(),
            initial_position: None,
        }),
        (!playlist.tracks.is_empty()).then_some(Message::ShufflePlaylist {
            playlist: playlist.clone(),
            initial_position: None,
        }),
        Message::Noop,
    );

    let highlighted_index = state.player_state.as_ref().and_then(|p| {
        if let chilen_backend::playback::QueueSource::Playlist { name: pl_name } = &p.queue_source
            && pl_name == &playlist.name
        {
            p.real_track_index(p.position)
        } else {
            None
        }
    });
    let playlist_cloned = playlist.clone();
    let track_buttons: Vec<_> = playlist
        .tracks
        .iter()
        .enumerate()
        .map(move |(index, track)| {
            TrackButton {
                state,
                track: track.clone(),
                messages: track_button::Messages {
                    play: Message::PlayPlaylistNoShuffle {
                        playlist: playlist_cloned.clone(),
                        initial_position: Some(index),
                    },
                    press: Message::PlayPlaylist {
                        playlist: playlist_cloned.clone(),
                        initial_position: Some(index),
                    },
                    shuffle: Some(Message::ShufflePlaylist {
                        playlist: playlist_cloned.clone(),
                        initial_position: Some(index),
                    }),
                    add_to_queue: Some(Message::AddTrackToQueue(track.clone())),
                    add_to_playlist: Message::AddTrackToPlaylist(track.clone()),
                    details: None,
                    remove: Some(Message::RemoveTrackFromPlaylist {
                        playlist: playlist_cloned.clone(),
                        index,
                    }),
                },
                info: track_button::Info::Artist,
                status: if highlighted_index
                    .map(|highlighted| highlighted == index)
                    .unwrap_or_default()
                {
                    track_button::Status::Playing
                } else {
                    track_button::Status::Idle
                },
            }
            .into()
        })
        .collect();

    let tracks_section: Element<'_, Message> = if track_buttons.is_empty() {
        container(
            text("No tracks yet!")
                .size(font::SIZE_REGULAR)
                .color(state.theme.on_surface_variant()),
        )
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
    } else {
        column(track_buttons).spacing(BUTTON_SPACING).into()
    };

    column![display, buttons, spacer(&state.theme), tracks_section]
        .width(Length::Fill)
        .spacing(SPACING_REGULAR)
        .into()
}
