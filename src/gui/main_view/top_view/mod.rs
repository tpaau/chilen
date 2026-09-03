mod album;
mod artist;
mod genre;
mod playlist;

use std::sync::Arc;

use chilen_backend::music_lib::{Album, Artist, Genre, Playlist, Track};
use iced::{Element, Length, Task, border::Radius};
use iced_m3::theme::ColorScheme;
use iced_widget::{Rule, container, responsive, row, rule, scrollable, text};
use log::error;

use crate::gui::{
    Chilen, SPACING_REGULAR, common_actions, dialog, font, icons, main_view::NavStack,
};

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
) -> Element<'a, Message> {
    responsive(move |size| {
        let button_size = if size.width < 466.0 {
            iced_m3::widget::button::Size::Small
        } else {
            iced_m3::widget::button::Size::Medium
        };
        row![
            iced_m3::widget::button(theme)
                .size(button_size)
                .style(iced_m3::widget::button::Style::Tonal(
                    iced_m3::theme::Accent::Tertiary,
                ))
                .label_maybe(None)
                .icon_font(icons::filled())
                .icon(&icons::ARROW_BACK)
                .on_press(Message::Unwind),
            iced_m3::widget::button(theme)
                .size(button_size.with_width(Length::Fill))
                .icon_font(icons::filled())
                .icon(&icons::PLAY_ARROW)
                .label("Play")
                .style(iced_m3::widget::button::Style::Tonal(
                    iced_m3::theme::Accent::Secondary
                ))
                .on_press(message_play.clone()),
            iced_m3::widget::button(theme)
                .size(button_size.with_width(Length::Fill))
                .icon_font(icons::filled())
                .icon(&icons::SHUFFLE)
                .label("Shuffle")
                .style(iced_m3::widget::button::Style::Filled(
                    iced_m3::theme::Accent::Primary
                ))
                .on_press(message_shuffle.clone()),
            iced_m3::widget::button(theme)
                .size(button_size)
                .icon_font(icons::filled())
                .style(iced_m3::widget::button::Style::Outlined)
                .icon(&icons::MORE_HORIZ)
                .label_maybe(None)
                .on_press(message_options.clone()),
        ]
        .spacing(SPACING_REGULAR)
        .into()
    })
    .into()
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
            initial_position,
        } => {
            common_actions::play_playlist(playlist, initial_position);
        }
        Message::PlayPlaylistNoShuffle {
            playlist,
            initial_position,
        } => {
            common_actions::play_playlist_no_shuffle(playlist, initial_position);
        }
        Message::ShufflePlaylist {
            playlist,
            initial_position,
        } => {
            common_actions::shuffle_playlist(playlist, initial_position);
        }
        Message::PlayAlbum {
            album,
            initial_position,
        } => {
            common_actions::play_album(album, initial_position);
        }
        Message::PlayAlbumNoShuffle {
            album,
            initial_position,
        } => {
            common_actions::play_album_no_shuffle(album, initial_position);
        }
        Message::ShuffleAlbum {
            album,
            initial_position,
        } => {
            common_actions::shuffle_album(album, initial_position);
        }
        Message::PlayArtist {
            artist,
            initial_position,
        } => {
            common_actions::play_artist(artist, initial_position);
        }
        Message::PlayArtistNoShuffle {
            artist,
            initial_position,
        } => {
            common_actions::play_artist_no_shuffle(artist, initial_position);
        }
        Message::ShuffleArtist {
            artist,
            initial_position,
        } => {
            common_actions::shuffle_artist(artist, initial_position);
        }
        Message::PlayGenre {
            genre,
            initial_position,
        } => {
            common_actions::play_genre(genre, initial_position);
        }
        Message::PlayGenreNoShuffle {
            genre,
            initial_position,
        } => {
            common_actions::play_genre_no_shuffle(genre, initial_position);
        }
        Message::ShuffleGenre {
            genre,
            initial_position,
        } => {
            common_actions::shuffle_genre(genre, initial_position);
        }
        Message::AddTrackToQueue(track) => common_actions::append_tracks_to_queue(vec![track]),
        Message::AddAlbumToQueue(album) => {
            common_actions::append_tracks_to_queue(album.tracks.clone())
        }
        Message::AddArtistToQueue(artist) => {
            common_actions::append_tracks_to_queue(artist.tracks.clone())
        }
        Message::AddTrackToPlaylist(track) => dialog::add_track_to_playlist(state, track),
    }
    Task::none()
}
