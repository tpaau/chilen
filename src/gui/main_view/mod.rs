mod albums;
mod artists;
mod genres;
mod home;
mod playlists;
pub mod top_view;
mod tracks;

use std::{ops::Not, path::PathBuf, sync::Arc};

use chilen_backend::{
    music_lib::{
        Album, Artist, Genre, LibraryItem, MusicLibrary, Playlist, Track, toggle_favorite,
    },
    playback::Queue,
};
use dirs::home_dir;
use iced::{Alignment, Border, Element, Length, Pixels, Task, padding};
use iced_core::text::{self, IntoFragment};
use iced_m3::{
    animation::motion::{self, fast_spatial},
    theme::{Accent, ColorScheme},
    widget::{
        button::{self, Content},
        hybrid_icon::Icon,
        navrail,
        shapes::polymorpher::{
            RoundedPolygon,
            shapes::{clover4, clover8, cookie6, cookie7, ghostish, sunny},
        },
        spacer,
    },
};
use iced_widget::{center, column, container, opaque, row, space, stack};
use log::{debug, error, info, trace};

use crate::{
    APP_NAME,
    gui::{
        self, Chilen, GOTH_CORNER_ROUNDING, SPACING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
        common_actions,
        dialog::{self, Dialog},
        font::bold_text,
        icons,
        main_view::{self, top_view::TopView},
        settings::Screen,
        widget::icon_shape::IconShape,
    },
};

pub(super) const VIEW_INNER_PADDING: f32 = crate::gui::SPACING_SMALL;

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

#[repr(usize)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum NavTab {
    #[default]
    Home,
    Playlists,
    Tracks,
    Albums,
    Artists,
    Genres,
}

impl NavTab {
    fn title(&self) -> String {
        match self {
            NavTab::Home => APP_NAME,
            NavTab::Playlists => "Playlists",
            NavTab::Tracks => "Tracks",
            NavTab::Albums => "Albums",
            NavTab::Artists => "Artists",
            NavTab::Genres => "Genres",
        }
        .to_string()
    }

    fn shape(&self) -> RoundedPolygon {
        match self {
            NavTab::Home => cookie6(),
            NavTab::Playlists => clover4(),
            NavTab::Tracks => ghostish(),
            NavTab::Albums => cookie7(),
            NavTab::Artists => clover8(),
            NavTab::Genres => sunny(),
        }
    }

    fn icon(&self) -> text::Fragment<'static> {
        match self {
            NavTab::Home => *icons::SNOWFLAKE,
            NavTab::Playlists => *icons::LIBRARY_MUSIC,
            NavTab::Tracks => *icons::MUSIC_NOTE,
            NavTab::Albums => *icons::ALBUM,
            NavTab::Artists => *icons::ARTIST,
            NavTab::Genres => *icons::GENRES,
        }
        .into_fragment()
    }
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

pub(crate) fn reload(state: &mut Chilen) {
    if let Some(lib) = &state.library {
        state.main_view.nav_stack.reload(lib);
        state.main_view.reload_visibilities(lib);
    } else {
        state.main_view.nav_stack = NavStack::default();
    }
}

pub struct State {
    pub nav_stack: NavStack,
    /// Holds which elements are visible and which are not.
    /// Sadly I couldn't use a range for this, it would've used much less memory but it lagged too
    /// much when I was scrolling erratically.
    pub visible: Option<Vec<bool>>,
}

impl State {
    fn reload_visibilities(&mut self, lib: &MusicLibrary) {
        if let Some(visible) = self.visible.as_mut() {
            let len = match self.nav_stack.tab {
                NavTab::Home => 0,
                NavTab::Playlists => lib.playlists.len(),
                NavTab::Tracks => lib.tracks.len(),
                NavTab::Albums => lib.albums.len(),
                NavTab::Artists => lib.artists.len(),
                NavTab::Genres => lib.genres.len(),
            };
            if len != visible.len() {
                // FIX: May cause issues if a lot of stuff is needlessly loaded because of this
                visible.resize(len, true);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    SwitchTab(NavTab),
    ButtonPoppedIn(usize),
    ButtonPoppedOut(usize),
    TopView(top_view::Message),
    OpenSettings,
    PlayTracks { initial_position: usize },
    PlayTracksNoShuffle { initial_position: usize },
    ShuffleTracks { initial_position: Option<usize> },
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
    CreatePlaylist,
    ExportPlaylist(String),
    SaveExportedPlaylist(String, Option<rfd::FileHandle>),
    ImportPlaylist,
    OpenPlaylistImportDialog(Option<rfd::FileHandle>),
    OpenPlaylistRenameDialog { playlist: String, name: String },
    ConfirmPlaylistDeletion(Arc<Playlist>),
    PlayPlaylist(Arc<Playlist>),
    ShufflePlaylist(Arc<Playlist>),
    AddPlaylistToQueue(Arc<Playlist>),
    ToggleSideBar,
    ToggleFavorite(LibraryItem),
    PlayTrack(Arc<Track>),
}

pub fn view<'a>(state: &'a Chilen, compact: bool) -> Element<'a, main_view::Message> {
    let index = state.main_view.nav_stack.tab as usize;
    let items = vec![
        navrail::Item {
            icon_active: Icon::Text {
                text: icons::HOME.into_fragment(),
                font: Some(icons::filled()),
            },
            icon_inactive: Icon::Text {
                text: icons::HOME.into_fragment(),
                font: Some(icons::outlined()),
            },
            label: "Home".into(),
            badge: None,
            on_press: Message::SwitchTab(NavTab::Home),
        },
        navrail::Item {
            icon_active: Icon::Text {
                text: icons::LIBRARY_MUSIC.into_fragment(),
                font: Some(icons::filled()),
            },
            icon_inactive: Icon::Text {
                text: icons::LIBRARY_MUSIC.into_fragment(),
                font: Some(icons::outlined()),
            },
            label: "Playlists".into(),
            badge: None,
            on_press: Message::SwitchTab(NavTab::Playlists),
        },
        navrail::Item {
            icon_active: Icon::Text {
                text: icons::MUSIC_NOTE.into_fragment(),
                font: Some(icons::filled()),
            },
            icon_inactive: Icon::Text {
                text: icons::MUSIC_NOTE.into_fragment(),
                font: Some(icons::outlined()),
            },
            label: "Tracks".into(),
            badge: None,
            on_press: Message::SwitchTab(NavTab::Tracks),
        },
        navrail::Item {
            icon_active: Icon::Text {
                text: icons::ALBUM.into_fragment(),
                font: Some(icons::filled()),
            },
            icon_inactive: Icon::Text {
                text: icons::ALBUM.into_fragment(),
                font: Some(icons::outlined()),
            },
            label: "Albums".into(),
            badge: None,
            on_press: Message::SwitchTab(NavTab::Albums),
        },
        navrail::Item {
            icon_active: Icon::Text {
                text: icons::ARTIST.into_fragment(),
                font: Some(icons::filled()),
            },
            icon_inactive: Icon::Text {
                text: icons::ARTIST.into_fragment(),
                font: Some(icons::outlined()),
            },
            label: "Artists".into(),
            badge: None,
            on_press: Message::SwitchTab(NavTab::Artists),
        },
        navrail::Item {
            icon_active: Icon::Text {
                text: icons::GENRES.into_fragment(),
                font: Some(icons::filled()),
            },
            icon_inactive: Icon::Text {
                text: icons::GENRES.into_fragment(),
                font: Some(icons::outlined()),
            },
            label: "Genres".into(),
            badge: None,
            on_press: Message::SwitchTab(NavTab::Genres),
        },
    ];

    let mode = if state.persistent.sidebar_opened && !compact {
        navrail::Mode::Expanded {
            width: Pixels(navrail::CONTAINER_EXPANDED_MIN_WIDTH),
        }
    } else {
        navrail::Mode::Collapsed
    };
    let navrail = container(
        navrail(navrail::Style::new(&state.theme), items)
            .active(index)
            .mode(mode)
            .container_vertical_padding(navrail::ITEM_OFFSET)
            .on_menu_pressed_maybe(compact.not().then_some(|_| Message::ToggleSideBar)),
    )
    .style(|_| container::Style::default().background(state.theme.surface_container()));

    let main_content = {
        let content = match state.main_view.nav_stack.top() {
            View::Top(top) => top_view::view(state, top).map(Message::TopView),
            // FIX: This is a WORKAROUND.
            // The `Scrollable` widget doesn't correctly manage its state which makes multiple
            //  in the same state tree share a state.
            //
            // There is a PR in iced that resolves this: https://github.com/iced-rs/iced/pull/3347
            View::Tab(tab) => match tab {
                NavTab::Home => home::view(state),
                NavTab::Playlists => stack![playlists::view(state)].into(),
                NavTab::Tracks => stack![
                    space().width(Length::Fill).height(Length::Fill),
                    tracks::view(state)
                ]
                .into(),
                NavTab::Albums => stack![
                    space().width(Length::Fill).height(Length::Fill),
                    space().width(Length::Fill).height(Length::Fill),
                    albums::view(state)
                ]
                .into(),
                NavTab::Artists => stack![
                    space().width(Length::Fill).height(Length::Fill),
                    space().width(Length::Fill).height(Length::Fill),
                    space().width(Length::Fill).height(Length::Fill),
                    artists::view(state)
                ]
                .into(),
                NavTab::Genres => stack![
                    space().width(Length::Fill).height(Length::Fill),
                    space().width(Length::Fill).height(Length::Fill),
                    space().width(Length::Fill).height(Length::Fill),
                    space().width(Length::Fill).height(Length::Fill),
                    genres::view(state)
                ]
                .into(),
            },
        };
        center(content)
    };

    let title = if let View::Tab(tab) = state.main_view.nav_stack.top() {
        let icon_size = if tab == NavTab::Home { 42.0 } else { 34.0 };
        Some(
            row![
                IconShape::new(
                    tab.shape(),
                    state.theme.primary(),
                    tab.icon(),
                    state.theme.on_primary()
                )
                .size(60.0)
                .icon_size(icon_size)
                .transition_spring(fast_spatial(motion::Scheme::default())),
                bold_text(tab.title()).size(38.0),
                spacer(state.theme.outline_variant())
            ]
            .padding(padding::horizontal(SPACING_REGULAR).top(SPACING_REGULAR))
            .align_y(Alignment::Center)
            .spacing(SPACING_SMALL),
        )
    } else {
        None
    };

    let main_container = container(column![title, main_content].spacing(SPACING_SMALL))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(VIEW_INNER_PADDING)
        .style(|_| {
            container::Style::default()
                .background(state.theme.surface())
                .border(Border::default().rounded(GOTH_CORNER_ROUNDING))
        });

    // TODO: Use an actual toolbar widget here
    let toolbar = container(opaque(
        container(
            row![
                iced_m3::widget::button(
                    button::Style::filled(&state.theme, Accent::default()),
                    Content::Icon(Icon::Text {
                        text: icons::SEARCH.into_fragment(),
                        font: Some(icons::filled())
                    })
                )
                .on_press(Message::Noop),
                iced_m3::widget::button(
                    button::Style::outlined(&state.theme),
                    Content::Icon(Icon::Text {
                        text: icons::SETTINGS.into_fragment(),
                        font: Some(icons::filled())
                    })
                )
                .on_press(Message::OpenSettings),
            ]
            .align_y(Alignment::Center)
            .spacing(SPACING_REGULAR),
        )
        .style(|_| {
            container::Style::default()
                .background(state.theme.surface_container_high())
                .border(Border::default().rounded(f32::MAX))
                .shadow(iced_m3::style::shadow(
                    state.theme.shadow(),
                    iced_m3::style::Elevation::Level3,
                ))
        })
        .padding(SPACING_SMALLER),
    ))
    .width(Length::Fill)
    .align_bottom(Length::Fill)
    .align_x(Alignment::Center)
    .padding(VIEW_INNER_PADDING);

    let main = stack!(main_container, toolbar);

    container(row![navrail, main])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn init_visible(lib: &MusicLibrary, tab: &NavTab) -> Vec<bool> {
    match tab {
        NavTab::Home => Vec::new(),
        NavTab::Playlists => vec![false; lib.playlists.len()],
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
        Message::OpenSettings => state.settings_state.screen = Some(Screen::default()),
        Message::PlayTracks { initial_position } => {
            common_actions::play_all_tracks(state, initial_position)
        }
        Message::PlayTracksNoShuffle { initial_position } => {
            common_actions::play_all_tracks_no_shuffle(state, Some(initial_position));
        }
        Message::ShuffleTracks { initial_position } => {
            common_actions::shuffle_tracks(state, initial_position);
        }
        Message::PlayAlbum(album) => {
            common_actions::play_album_no_shuffle(state, album, None);
        }
        Message::ShuffleAlbum(album) => {
            common_actions::shuffle_album(state, album, None);
        }
        Message::PlayArtist(artist) => {
            common_actions::play_artist_no_shuffle(state, artist, None);
        }
        Message::ShuffleArtist(artist) => {
            common_actions::shuffle_artist(state, artist, None);
        }
        Message::PlayGenre(genre) => {
            common_actions::play_genre_no_shuffle(state, genre, None);
        }
        Message::ShuffleGenre(genre) => {
            common_actions::shuffle_genre(state, genre, None);
        }
        Message::AddTrackToQueue(track) => common_actions::append_tracks_to_queue(vec![track]),
        Message::AddAlbumToQueue(album) => {
            common_actions::append_tracks_to_queue(album.tracks.to_vec())
        }
        Message::AddArtistToQueue(artist) => {
            common_actions::append_tracks_to_queue(artist.tracks.to_vec())
        }
        Message::AddGenreToQueue(genre) => {
            common_actions::append_tracks_to_queue(genre.tracks.to_vec())
        }
        Message::AddTrackToPlaylist(track) => dialog::add_track_to_playlist(state, track),
        Message::CreatePlaylist => {
            state.dialog = Dialog::CreatePlaylist(String::new());
        }
        Message::ExportPlaylist(name) => {
            return Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_directory(home_dir().unwrap_or(PathBuf::from(".")))
                    .set_file_name(format!("{name}.m3u8"))
                    .save_file(),
                |maybe_handle| Message::SaveExportedPlaylist(name, maybe_handle),
            );
        }
        Message::SaveExportedPlaylist(name, file_handle) => match file_handle {
            Some(handle) => {
                if let Err(e) =
                    chilen_backend::music_lib::export_playlist_to_m3u8(&name, handle.path())
                {
                    let msg = format!(
                        "Couldn't export playlist {name} to {:?}: {e}",
                        handle.path()
                    );
                    error!("{msg}");
                    state.dialog = Dialog::Error(msg);
                }
            }
            None => debug!("User cancelled the export operation"),
        },
        Message::ImportPlaylist => {
            return Task::perform(
                rfd::AsyncFileDialog::new()
                    .add_filter("M3U8 Playlist File", &["m3u", "m3u8"])
                    .set_directory(home_dir().unwrap_or(PathBuf::from(".")))
                    .pick_file(),
                Message::OpenPlaylistImportDialog,
            );
        }
        Message::OpenPlaylistImportDialog(handle) => match handle {
            Some(handle) => {
                trace!("Showing import dialog for playlist {handle:?}");
                state.dialog = Dialog::ImportPlaylist(String::new(), handle);
            }
            None => info!("Didn't get the file handle, guessing the user cancelled the import"),
        },
        Message::OpenPlaylistRenameDialog { playlist, name } => {
            state.dialog = Dialog::RenamePlaylist { playlist, name }
        }
        Message::ConfirmPlaylistDeletion(playlist) => {
            if playlist.tracks.is_empty() {
                if let Err(e) = chilen_backend::music_lib::delete_playlists(vec![&playlist.name]) {
                    error!("Couldn't delete playlist: {e}");
                    state.dialog = Dialog::Error(format!("Couldn't delete playlist: {e}"))
                }
            } else {
                state.dialog = Dialog::DeletePlaylist(playlist)
            }
        }
        Message::PlayPlaylist(playlist) => {
            common_actions::play_playlist(state, playlist, None);
        }
        Message::ShufflePlaylist(playlist) => {
            common_actions::play_playlist(state, playlist, None);
        }
        Message::AddPlaylistToQueue(playlist) => {
            common_actions::append_tracks_to_queue(playlist.tracks.to_vec())
        }
        Message::ToggleSideBar => {
            state.persistent.sidebar_opened = !state.persistent.sidebar_opened;
            state.save_persistent();
        }
        Message::ToggleFavorite(favorite_item) => {
            let _ = toggle_favorite(favorite_item);
        }
        Message::PlayTrack(track) => {
            let _ = chilen_backend::playback::play_new_queue(
                Queue::Custom {
                    label: track
                        .title
                        .clone()
                        .map(|t| t.into_string())
                        .unwrap_or("Track".to_string()),
                    tracks: vec![track],
                },
                None,
            );
        }
    }
    Task::none()
}
