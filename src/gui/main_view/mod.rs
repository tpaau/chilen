mod albums;
mod artists;
mod genres;
pub mod top_view;
mod tracks;

use std::sync::Arc;

use chilen_backend::music_lib::{Album, Artist, Genre, MusicLibrary, Track};
use iced::{Alignment, Border, Element, Length, Task, padding};
use iced_m3::theme::ColorScheme;
use iced_widget::{center, column, container, row, space, stack};
use log::trace;

use crate::gui::{
    self, Chilen, ROUNDING_REGULAR, SPACING_REGULAR, SPACING_SMALL, common_actions, dialog, icons,
    main_view::{self, top_view::TopView},
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum View {
    Tab(NavTab),
    Top(TopView),
}

impl Default for View {
    fn default() -> Self {
        Self::Tab(NavTab::default())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum NavTab {
    #[default]
    Tracks,
    Albums,
    Artists,
    Genres,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NavStack {
    tab: NavTab,
    stack: Vec<TopView>,
}

impl NavStack {
    fn tab(&self) -> &NavTab {
        &self.tab
    }

    fn top(&self) -> View {
        if !self.stack.is_empty() {
            View::Top(self.stack[self.stack.len() - 1].clone())
        } else {
            View::Tab(self.tab)
        }
    }

    pub fn unwind(&mut self) -> Option<TopView> {
        self.stack.pop()
    }

    pub fn navigate(&mut self, top_view: TopView) {
        if self.top() != View::Top(top_view.clone()) {
            self.stack.push(top_view);
        }
    }

    pub fn set_top(&mut self, top_view: TopView) {
        if self.top() != View::Top(top_view.clone()) {
            self.stack = vec![top_view];
        }
    }

    pub fn reload(&mut self, lib: &MusicLibrary) {
        trace!("Reloading top view");
        self.stack = self
            .stack
            .iter()
            .filter_map(|view| match view {
                TopView::Playlist(playlist) => lib
                    .find_playlist_by_id(playlist.id)
                    .map(|p| TopView::Playlist(p.clone())),
                TopView::Album(album) => lib
                    .find_album(&album.title)
                    .map(|a| TopView::Album(a.clone())),
                TopView::Artist(artist) => lib
                    .find_artist(&artist.name)
                    .map(|a| TopView::Artist(a.clone())),
                TopView::Genre(genre) => lib
                    .find_genre(&genre.name)
                    .map(|g| TopView::Genre(g.clone())),
            })
            .collect();
    }

    fn switch_tab(&mut self, tab: NavTab) {
        self.stack = Vec::new();
        self.tab = tab;
    }
}

// TODO: Saving the navigation state on disk and then restoring it on startup would be a nice addition
pub struct State {
    pub nav_stack: NavStack,
    /// Holds which elements are visible and which are not.
    /// Sadly I couldn't use a range for this, it would've used much less memory but it lagged too
    /// much when I was scrolling erratically.
    pub visible: Option<Vec<bool>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Noop,
    SwitchTab(NavTab),
    ButtonPoppedIn(usize),
    ButtonPoppedOut(usize),
    TopView(top_view::Message),
    OpenSettings,
    PlayTracks { initial_position: usize },
    PlayTracksNoShuffle { initial_position: usize },
    ShuffleTracks { initial_position: usize },
    PlayAlbum(Arc<Album>),
    ShuffleAlbum(Arc<Album>),
    PlayArtist(Arc<Artist>),
    ShuffleArtist(Arc<Artist>),
    PlayGenre(Arc<Genre>),
    ShuffleGenre(Arc<Genre>),
    AddTrackToQueue(Arc<Track>),
    AddAlbumToQueue(Arc<Album>),
    AddArtistToQueue(Arc<Artist>),
    AddGenreToQueue(Arc<Genre>),
    AddTrackToPlaylist(Arc<Track>),
}

pub fn view<'a>(state: &'a Chilen) -> Element<'a, main_view::Message> {
    container(column![
        row![
            iced_m3::widget::button(&state.theme)
                .style(iced_m3::widget::button::Style::Filled(
                    iced_m3::theme::Accent::Primary
                ))
                .label_maybe(None)
                .icon(&icons::SEARCH)
                .on_press(Message::Noop),
            // TODO: Custom ordering
            {
                let index = match state.main_view.nav_stack.tab {
                    NavTab::Tracks => 0,
                    NavTab::Albums => 1,
                    NavTab::Artists => 2,
                    NavTab::Genres => 3,
                };
                iced_m3::widget::navbar::<_, iced::Theme, iced::Renderer>(
                    vec![
                        iced_m3::widget::navbar::Item {
                            icon: &icons::MUSIC_NOTE,
                            label: "Tracks",
                            message: Message::SwitchTab(NavTab::Tracks),
                        },
                        iced_m3::widget::navbar::Item {
                            icon: &icons::ALBUM,
                            label: "Albums",
                            message: Message::SwitchTab(NavTab::Albums),
                        },
                        iced_m3::widget::navbar::Item {
                            icon: &icons::ARTIST,
                            label: "Artists",
                            message: Message::SwitchTab(NavTab::Artists),
                        },
                        iced_m3::widget::navbar::Item {
                            icon: &icons::GENRES,
                            label: "Genres",
                            message: Message::SwitchTab(NavTab::Genres),
                        },
                    ],
                    &state.theme,
                )
                .focused_index(index)
                .icon_font_active(icons::filled())
                .icon_font_inactive(icons::outlined())
            },
            iced_m3::widget::button(&state.theme)
                .style(iced_m3::widget::button::Style::Outlined)
                .label_maybe(None)
                .icon(&icons::SETTINGS)
                .on_press(Message::OpenSettings),
        ]
        .align_y(Alignment::Center)
        .spacing(SPACING_REGULAR),
        {
            let content: Element<'_, Message> = if let Some(lib) = &state.library {
                let content = match state.main_view.nav_stack.top() {
                    View::Top(top) => top_view::view(state, top).map(Message::TopView),
                    // FIX: This is a WORKAROUND.
                    // The `Scrollable` widget doesn't correctly manage its state which makes multiple
                    // scrollables in the same state tree share a state.
                    //
                    // There is a PR in iced that resolves this: https://github.com/iced-rs/iced/pull/3347
                    View::Tab(tab) => match tab {
                        NavTab::Tracks => tracks::view(state, lib),
                        NavTab::Albums => stack![albums::view(state, lib)].into(),
                        NavTab::Artists => stack![
                            space().width(Length::Fill).height(Length::Fill),
                            artists::view(state, lib)
                        ]
                        .into(),
                        NavTab::Genres => stack![
                            space().width(Length::Fill).height(Length::Fill),
                            space().width(Length::Fill).height(Length::Fill),
                            genres::view(state, lib)
                        ]
                        .into(),
                    },
                };
                center(content).into()
            } else {
                space().into()
            };
            content
        }
    ])
    .style(|_| {
        container::Style::default()
            .background(state.theme.background())
            .border(Border::default().rounded(ROUNDING_REGULAR))
    })
    .padding(padding::horizontal(SPACING_SMALL))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn init_visible(lib: &MusicLibrary, tab: &NavTab) -> Vec<bool> {
    match tab {
        NavTab::Tracks => vec![false; lib.tracks.len()],
        NavTab::Albums => vec![false; lib.albums.len()],
        NavTab::Artists => vec![false; lib.artists.len()],
        NavTab::Genres => vec![false; lib.genres.len()],
    }
}

fn change_button(vec: &mut [bool], index: usize, visible: bool) {
    if index < vec.len() {
        vec[index] = visible;
    }
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    if state.main_view.visible.is_none()
        && let Some(lib) = &state.library
        && !matches!(message, Message::SwitchTab(_))
    {
        state.main_view.visible = Some(init_visible(lib, state.main_view.nav_stack.tab()));
    }
    match message {
        Message::SwitchTab(tab) => {
            let top = state.main_view.nav_stack.top();
            if top != gui::main_view::View::Tab(tab) {
                state.main_view.visible = state.library.as_ref().map(|lib| init_visible(lib, &tab));
                state.main_view.nav_stack.switch_tab(tab);
            }
        }
        Message::ButtonPoppedIn(index) => {
            change_button(state.main_view.visible.as_mut().unwrap(), index, true);
        }
        Message::ButtonPoppedOut(index) => {
            change_button(state.main_view.visible.as_mut().unwrap(), index, false);
        }
        Message::TopView(message) => return top_view::update(state, message).map(Message::TopView),
        Message::Noop => {}
        Message::OpenSettings => state.dialog = gui::dialog::Dialog::Settings,
        Message::PlayTracks { initial_position } => {
            common_actions::play_tracks(state, initial_position)
        }
        Message::PlayTracksNoShuffle { initial_position } => {
            common_actions::play_tracks_no_shuffle(state, Some(initial_position));
        }
        Message::ShuffleTracks { initial_position } => {
            common_actions::shuffle_tracks(state, Some(initial_position));
        }
        Message::PlayAlbum(album) => {
            common_actions::play_album_no_shuffle(album, None);
        }
        Message::ShuffleAlbum(album) => {
            common_actions::shuffle_album(album, None);
        }
        Message::PlayArtist(artist) => {
            common_actions::play_artist_no_shuffle(artist, None);
        }
        Message::ShuffleArtist(artist) => {
            common_actions::shuffle_artist(artist, None);
        }
        Message::PlayGenre(genre) => {
            common_actions::play_genre_no_shuffle(genre, None);
        }
        Message::ShuffleGenre(genre) => {
            common_actions::shuffle_genre(genre, None);
        }
        Message::AddTrackToQueue(track) => common_actions::append_tracks_to_queue(vec![track]),
        Message::AddAlbumToQueue(album) => {
            common_actions::append_tracks_to_queue(album.tracks.clone())
        }
        Message::AddArtistToQueue(artist) => {
            common_actions::append_tracks_to_queue(artist.tracks.clone())
        }
        Message::AddGenreToQueue(genre) => {
            common_actions::append_tracks_to_queue(genre.tracks.clone())
        }
        Message::AddTrackToPlaylist(track) => dialog::add_track_to_playlist(state, track),
    }
    Task::none()
}
