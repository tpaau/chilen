mod album;
mod artist;
mod genre;
mod playlist;

use std::sync::Arc;

use chilen_backend::music_lib::{Album, Artist, Genre, Playlist};
use iced::{Element, Length, Task, border::Radius};
use iced_m3::theme::ColorScheme;
use iced_widget::{Row, Rule, container, row, rule, scrollable, text};
use log::error;

use crate::gui::{Chilen, SPACING_REGULAR, font, icons, main_view::NavStack};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopView {
    Playlist(Arc<Playlist>),
    Album(Arc<Album>),
    Artist(Arc<Artist>),
    Genre(Arc<Genre>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Noop,
    Navigate(TopView),
    Unwind,
    RemoveTrackFromPlaylist {
        playlist: Arc<Playlist>,
        index: usize,
    },
    PlayPlaylist {
        playlist: Arc<Playlist>,
        initial_index: usize,
    },
    PlayPlaylistNoShuffle {
        playlist: Arc<Playlist>,
        initial_index: usize,
    },
    ShufflePlaylist {
        playlist: Arc<Playlist>,
        initial_index: usize,
    },
    PlayAlbum {
        album: Arc<Album>,
        initial_index: usize,
    },
    PlayAlbumNoShuffle {
        album: Arc<Album>,
        initial_index: usize,
    },
    ShuffleAlbum {
        album: Arc<Album>,
        initial_index: usize,
    },
    PlayArtist {
        artist: Arc<Artist>,
        initial_index: usize,
    },
    PlayArtistNoShuffle {
        artist: Arc<Artist>,
        initial_index: usize,
    },
    ShuffleArtist {
        artist: Arc<Artist>,
        initial_index: usize,
    },
    PlayGenre {
        genre: Arc<Genre>,
        initial_index: usize,
    },
    PlayGenreNoShuffle {
        genre: Arc<Genre>,
        initial_index: usize,
    },
    ShuffleGenre {
        genre: Arc<Genre>,
        initial_index: usize,
    },
}

fn title<'a>(theme: &'a impl ColorScheme, content: String) -> Element<'a, Message> {
    text(content)
        .size(32.0)
        .font(font::font_bold())
        .color(theme.on_surface())
        .into()
}

fn horizontal_buttons<'a>(
    theme: &'a impl ColorScheme,
    message_play: Message,
    message_shuffle: Message,
    message_options: Message,
) -> Row<'a, Message> {
    row![
        iced_m3::widget::button(theme)
            .size(iced_m3::widget::button::Size::Medium)
            .style(iced_m3::widget::button::Style::Tonal(
                iced_m3::theme::Accent::Tertiary,
            ))
            .label_maybe(None)
            .icon_font(icons::filled())
            .icon(&icons::ARROW_BACK)
            .on_press(Message::Unwind),
        iced_m3::widget::button(theme)
            .size(iced_m3::widget::button::Size::Medium.with_width(Length::Fill))
            .icon_font(icons::filled())
            .icon(&icons::PLAY_ARROW)
            .label("Play")
            .style(iced_m3::widget::button::Style::Tonal(
                iced_m3::theme::Accent::Secondary
            ))
            .on_press(message_play),
        iced_m3::widget::button(theme)
            .size(iced_m3::widget::button::Size::Medium.with_width(Length::Fill))
            .icon_font(icons::filled())
            .icon(&icons::SHUFFLE)
            .label("Shuffle")
            .style(iced_m3::widget::button::Style::Filled(
                iced_m3::theme::Accent::Primary
            ))
            .on_press(message_shuffle),
        iced_m3::widget::button(theme)
            .size(iced_m3::widget::button::Size::Medium)
            .icon_font(icons::filled())
            .style(iced_m3::widget::button::Style::Outlined)
            .icon(&icons::MORE_HORIZ)
            .label_maybe(None)
            .on_press(message_options),
    ]
    .spacing(SPACING_REGULAR)
}

fn spacer<'a>(theme: &'a impl ColorScheme) -> Rule<'a> {
    rule::horizontal(1.0).style(|_| rule::Style {
        color: theme.outline_variant(),
        radius: Radius::default(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    })
}

const MAX_COVER_SIZE: f32 = 512.0;
const MIN_COVER_SIZE: f32 = 192.0;

pub fn view(state: &Chilen, view: TopView) -> Element<'_, Message> {
    // FIX: Lists should be virtualized.
    // This is currently not viable as I would have to do the same workaround as in the `main_view`,
    // where I put the actual content in a `stack!` so that the layout is different and all
    // scrollables don't share the same state.
    let content = match view {
        TopView::Playlist(playlist) => playlist::view(state, playlist),
        TopView::Album(album) => album::view(state, album),
        TopView::Artist(artist) => artist::view(state, artist),
        TopView::Genre(genre) => genre::view(state, genre),
    };

    container(
        scrollable(content).style(|_, status| iced_m3::style::scrollable(status, &state.theme)),
    )
    .align_top(Length::Fill)
    .into()
}

pub(crate) fn reload(state: &mut Chilen) {
    if let Some(lib) = &state.library {
        state.main_view.nav_stack.reload(lib);
    } else {
        state.main_view.nav_stack = NavStack::default();
    }
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::Navigate(top_view) => state.main_view.nav_stack.navigate(top_view),
        Message::Unwind => {
            state.main_view.nav_stack.unwind();
        }
        Message::Noop => {
            // TODO: This is a placeholder
        }
        Message::RemoveTrackFromPlaylist { playlist, index } => {
            if let Err(e) = chilen_backend::music_lib::remove_tracks(&playlist.name, vec![index]) {
                let msg = format!("Couldn't remove track from playlist: {e}");
                error!("{msg}");
                state.dialog = crate::gui::dialog::Dialog::Error(msg);
            }
        }
        Message::PlayPlaylist {
            playlist,
            initial_index,
        } => {
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Playlist(playlist),
                initial_index,
            );
        }
        Message::PlayPlaylistNoShuffle {
            playlist,
            initial_index,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(
                chilen_backend::playback::ShuffleState::Off,
            );
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Playlist(playlist),
                initial_index,
            );
        }
        Message::ShufflePlaylist {
            playlist,
            initial_index,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(
                chilen_backend::playback::ShuffleState::On,
            );
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Playlist(playlist),
                initial_index,
            );
        }
        Message::PlayAlbum {
            album,
            initial_index,
        } => {
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Album(album),
                initial_index,
            );
        }
        Message::PlayAlbumNoShuffle {
            album,
            initial_index,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(
                chilen_backend::playback::ShuffleState::Off,
            );
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Album(album),
                initial_index,
            );
        }
        Message::ShuffleAlbum {
            album,
            initial_index,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(
                chilen_backend::playback::ShuffleState::On,
            );
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Album(album),
                initial_index,
            );
        }
        Message::PlayArtist {
            artist,
            initial_index,
        } => {
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Artist(artist),
                initial_index,
            );
        }
        Message::PlayArtistNoShuffle {
            artist,
            initial_index,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(
                chilen_backend::playback::ShuffleState::Off,
            );
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Artist(artist),
                initial_index,
            );
        }
        Message::ShuffleArtist {
            artist,
            initial_index,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(
                chilen_backend::playback::ShuffleState::On,
            );
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Artist(artist),
                initial_index,
            );
        }
        Message::PlayGenre {
            genre,
            initial_index,
        } => {
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Genre(genre),
                initial_index,
            );
        }
        Message::PlayGenreNoShuffle {
            genre,
            initial_index,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(
                chilen_backend::playback::ShuffleState::Off,
            );
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Genre(genre),
                initial_index,
            );
        }
        Message::ShuffleGenre {
            genre,
            initial_index,
        } => {
            let _ = chilen_backend::playback::set_shuffle_state(
                chilen_backend::playback::ShuffleState::On,
            );
            let _ = chilen_backend::playback::play_new_queue(
                chilen_backend::playback::Queue::Genre(genre),
                initial_index,
            );
        }
    }
    Task::none()
}
