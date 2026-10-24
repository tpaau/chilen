mod album;
mod artist;
mod genre;
mod playlist;

use std::sync::Arc;

use chilen_backend::music_lib::{Album, Artist, Genre, Playlist, Track};
use iced::{Element, Length, Task};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::{Accent, ColorScheme},
    widget::{
        button::{self, Content},
        button_group,
        hybrid_icon::Icon,
        standard_button_group,
    },
};
use iced_widget::{container, responsive, scrollable};
use log::error;

use crate::gui::{Chilen, common_actions, dialog, font::bold_text, icons};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopView {
    Playlist(Arc<Playlist>),
    Album(Arc<Album>),
    Artist(Arc<Artist>),
    Genre(Arc<Genre>),
}

impl From<Arc<Playlist>> for TopView {
    fn from(value: Arc<Playlist>) -> Self {
        Self::Playlist(value)
    }
}

impl From<Arc<Album>> for TopView {
    fn from(value: Arc<Album>) -> Self {
        Self::Album(value)
    }
}

impl From<Arc<Artist>> for TopView {
    fn from(value: Arc<Artist>) -> Self {
        Self::Artist(value)
    }
}

impl From<Arc<Genre>> for TopView {
    fn from(value: Arc<Genre>) -> Self {
        Self::Genre(value)
    }
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
        initial_position: Option<usize>,
    },
    PlayPlaylistNoShuffle {
        playlist: Arc<Playlist>,
        initial_position: Option<usize>,
    },
    ShufflePlaylist {
        playlist: Arc<Playlist>,
        initial_position: Option<usize>,
    },
    PlayAlbum {
        album: Arc<Album>,
        initial_position: Option<usize>,
    },
    PlayAlbumNoShuffle {
        album: Arc<Album>,
        initial_position: Option<usize>,
    },
    ShuffleAlbum {
        album: Arc<Album>,
        initial_position: Option<usize>,
    },
    PlayArtist {
        artist: Arc<Artist>,
        initial_position: Option<usize>,
    },
    PlayArtistNoShuffle {
        artist: Arc<Artist>,
        initial_position: Option<usize>,
    },
    ShuffleArtist {
        artist: Arc<Artist>,
        initial_position: Option<usize>,
    },
    PlayGenre {
        genre: Arc<Genre>,
        initial_position: Option<usize>,
    },
    PlayGenreNoShuffle {
        genre: Arc<Genre>,
        initial_position: Option<usize>,
    },
    ShuffleGenre {
        genre: Arc<Genre>,
        initial_position: Option<usize>,
    },
    AddTrackToQueue(Arc<Track>),
    AddAlbumToQueue(Arc<Album>),
    AddArtistToQueue(Arc<Artist>),
    AddTrackToPlaylist(Arc<Track>),
}

fn title<'a>(theme: &'a impl ColorScheme, content: String) -> Element<'a, Message> {
    bold_text(content)
        .size(32.0)
        .color(theme.on_surface())
        .into()
}

fn horizontal_buttons<'a>(
    theme: &'a impl ColorScheme,
    message_play: Option<Message>,
    message_shuffle: Option<Message>,
    message_options: Message,
) -> Element<'a, Message> {
    responsive(move |size| {
        let (size, spacing) = if size.width < 466.0 {
            (
                iced_m3::widget::button::Size::small(),
                button_group::standard::SPACING_SMALL,
            )
        } else {
            (
                iced_m3::widget::button::Size::medium(),
                button_group::standard::SPACING_MEDIUM,
            )
        };

        let buttons = vec![
            iced_m3::widget::button(
                button::Style::tonal(theme, Accent::Tertiary),
                Content::Icon(Icon::Text {
                    text: icons::ARROW_BACK.into_fragment(),
                    font: Some(icons::filled()),
                }),
            )
            .size(size)
            .on_press(Message::Unwind),
            iced_m3::widget::button(
                button::Style::tonal(theme, Accent::Secondary),
                Content::Full {
                    icon: Icon::Text {
                        text: icons::PLAY_ARROW.into_fragment(),
                        font: Some(icons::filled()),
                    },
                    label: "Play".into(),
                },
            )
            .size(size.width(Length::Fill))
            .on_press_maybe(message_play.clone()),
            iced_m3::widget::button(
                button::Style::filled(theme, Accent::Primary),
                Content::Full {
                    icon: Icon::Text {
                        text: icons::SHUFFLE.into_fragment(),
                        font: Some(icons::filled()),
                    },
                    label: "Shuffle".into(),
                },
            )
            .size(size.width(Length::Fill))
            .on_press_maybe(message_shuffle.clone()),
            iced_m3::widget::button(
                button::Style::outlined(theme),
                Content::Icon(Icon::Text {
                    text: icons::MORE_HORIZ.into_fragment(),
                    font: Some(icons::filled()),
                }),
            )
            .size(size)
            .on_press(message_options.clone()),
        ];

        standard_button_group(buttons)
            .spacing(spacing)
            .expanded_ratio(0.03) // So that the smaller buttons don't get crushed
            .into()
    })
    .into()
}

const MAX_COVER_SIZE: f32 = 512.0;
const MIN_COVER_SIZE: f32 = 192.0;

pub fn view<'a>(state: &'a Chilen, view: TopView) -> Element<'a, Message> {
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
            initial_position,
        } => {
            common_actions::play_playlist(state, playlist, initial_position);
        }
        Message::PlayPlaylistNoShuffle {
            playlist,
            initial_position,
        } => {
            common_actions::play_playlist_no_shuffle(state, playlist, initial_position);
        }
        Message::ShufflePlaylist {
            playlist,
            initial_position,
        } => {
            common_actions::shuffle_playlist(state, playlist, initial_position);
        }
        Message::PlayAlbum {
            album,
            initial_position,
        } => {
            common_actions::play_album(state, album, initial_position);
        }
        Message::PlayAlbumNoShuffle {
            album,
            initial_position,
        } => {
            common_actions::play_album_no_shuffle(state, album, initial_position);
        }
        Message::ShuffleAlbum {
            album,
            initial_position,
        } => {
            common_actions::shuffle_album(state, album, initial_position);
        }
        Message::PlayArtist {
            artist,
            initial_position,
        } => {
            common_actions::play_artist(state, artist, initial_position);
        }
        Message::PlayArtistNoShuffle {
            artist,
            initial_position,
        } => {
            common_actions::play_artist_no_shuffle(state, artist, initial_position);
        }
        Message::ShuffleArtist {
            artist,
            initial_position,
        } => {
            common_actions::shuffle_artist(state, artist, initial_position);
        }
        Message::PlayGenre {
            genre,
            initial_position,
        } => {
            common_actions::play_genre(state, genre, initial_position);
        }
        Message::PlayGenreNoShuffle {
            genre,
            initial_position,
        } => {
            common_actions::play_genre_no_shuffle(state, genre, initial_position);
        }
        Message::ShuffleGenre {
            genre,
            initial_position,
        } => {
            common_actions::shuffle_genre(state, genre, initial_position);
        }
        Message::AddTrackToQueue(track) => common_actions::append_tracks_to_queue(vec![track]),
        Message::AddAlbumToQueue(album) => {
            common_actions::append_tracks_to_queue(album.tracks.to_vec())
        }
        Message::AddArtistToQueue(artist) => {
            common_actions::append_tracks_to_queue(artist.tracks.to_vec())
        }
        Message::AddTrackToPlaylist(track) => dialog::add_track_to_playlist(state, track),
    }
    Task::none()
}
