mod albums;
mod artists;
mod favorites;
mod genres;
mod home;
mod playlists;
pub mod top_view;
mod tracks;

use std::{ops::Not, sync::LazyLock};

use chilen_backend::music_lib::MusicLibrary;
use iced::{Alignment, Border, Element, Length, Pixels, Task, padding};
use iced_core::text::{self, IntoFragment};
use iced_m3::{
    animation::motion::{self, fast_spatial},
    theme::ColorScheme,
    widget::{
        button::{self, Content, CornerRadius},
        hybrid_icon::Icon,
        navrail,
        shapes::polymorpher::{
            RoundedPolygon,
            shapes::{clover4, clover8, cookie6, cookie7, ghostish, sunny, very_sunny},
        },
        spacer,
    },
};
use iced_widget::{center, column, container, opaque, row, space, stack};
use log::trace;

use crate::{
    APP_NAME,
    gui::{
        self, Chilen, GOTH_CORNER_ROUNDING, SPACING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
        event_loop, font::bold_text, icons, main_view::top_view::TopView, settings,
        widget::icon_shape::IconShape,
    },
};

pub(super) const NO_FAVORITES_LABEL: &str = "No favorites yet!";
pub(super) const VIEW_INNER_PADDING: f32 = crate::gui::SPACING_SMALL;
const TOOLBAR_OUTER_PADDING: f32 = SPACING_SMALL;
static TOOLBAR_BUTTON_SIZE: LazyLock<button::Size> = LazyLock::new(button::Size::small);
const TOOLBAR_INNER_PADDING: f32 = SPACING_SMALLER;
static TOOLBAR_OFFSET: LazyLock<f32> = LazyLock::new(|| {
    let height = TOOLBAR_BUTTON_SIZE.height;
    if let Length::Fixed(height) = height {
        height + 2.0 * TOOLBAR_INNER_PADDING + 2.0 * TOOLBAR_OUTER_PADDING
    } else {
        unreachable!("The button height should always be fixed!")
    }
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum View {
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
    Favorites,
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
            NavTab::Favorites => "Favorites",
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
            NavTab::Favorites => very_sunny(),
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
            NavTab::Favorites => *icons::STAR,
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

    pub(super) fn top(&self) -> View {
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

    pub(super) fn switch_tab(&mut self, tab: NavTab) {
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

#[derive(Default)]
pub struct State {
    pub nav_stack: NavStack,
    /// Holds which elements are visible and which are not.
    /// Sadly I couldn't use a range for this, it would've used much less memory but it lagged too
    /// much when I was scrolling erratically.
    pub visible: Option<Vec<bool>>,
    pub button_menu: Option<usize>, // The index of the button which has a menu opened
    pub top_view: top_view::State,
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
                NavTab::Favorites => lib.favorites.len(),
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
    SwitchTab(NavTab),
    ButtonPoppedIn(usize),
    ButtonPoppedOut(usize),
    TopView(top_view::Message),
    ToggleSideBar,
    ToggleMenu(usize),
}

pub fn view<'a>(state: &'a Chilen, compact: bool) -> Element<'a, event_loop::Message> {
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
            on_press: event_loop::Message::MainView(Message::SwitchTab(NavTab::Home)),
        },
        navrail::Item {
            icon_active: Icon::Text {
                text: icons::STAR.into_fragment(),
                font: Some(icons::filled()),
            },
            icon_inactive: Icon::Text {
                text: icons::STAR.into_fragment(),
                font: Some(icons::outlined()),
            },
            label: "Favorites".into(),
            badge: None,
            on_press: event_loop::Message::MainView(Message::SwitchTab(NavTab::Favorites)),
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
            on_press: event_loop::Message::MainView(Message::SwitchTab(NavTab::Playlists)),
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
            on_press: event_loop::Message::MainView(Message::SwitchTab(NavTab::Tracks)),
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
            on_press: event_loop::Message::MainView(Message::SwitchTab(NavTab::Albums)),
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
            on_press: event_loop::Message::MainView(Message::SwitchTab(NavTab::Artists)),
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
            on_press: event_loop::Message::MainView(Message::SwitchTab(NavTab::Genres)),
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
            .on_menu_pressed_maybe(
                compact
                    .not()
                    .then_some(|_| event_loop::Message::MainView(Message::ToggleSideBar)),
            ),
    )
    .style(|_| container::Style::default().background(state.theme.surface_container()));

    let main_content = {
        let content = match state.main_view.nav_stack.top() {
            View::Top(top) => top_view::view(state, top),
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
                NavTab::Favorites => favorites::view(state),
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
                // TODO: Track search
                // iced_m3::widget::button(
                //     button::Style::filled(&state.theme, Accent::default()),
                //     Content::Icon(Icon::Text {
                //         text: icons::SEARCH.into_fragment(),
                //         font: Some(icons::filled())
                //     })
                // )
                // .size(*TOOLBAR_BUTTON_SIZE)
                // .on_press(Message::Noop),
                iced_m3::widget::button(
                    button::Style::outlined(&state.theme),
                    Content::Full {
                        icon: Icon::Text {
                            text: icons::SETTINGS.into_fragment(),
                            font: Some(icons::filled())
                        },
                        label: "Settings".into()
                    }
                )
                .size(
                    (*TOOLBAR_BUTTON_SIZE).corner_radius(CornerRadius::medium().shape_morph(false))
                )
                .on_press(event_loop::Message::Settings(settings::Message::Open)),
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
        .padding(TOOLBAR_INNER_PADDING),
    ))
    .width(Length::Fill)
    .align_bottom(Length::Fill)
    .align_x(Alignment::Center)
    .padding(VIEW_INNER_PADDING + TOOLBAR_OUTER_PADDING);

    let main = stack!(main_container, toolbar);

    container(row![navrail, main])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub(super) fn init_visible(lib: &MusicLibrary, tab: &NavTab) -> Vec<bool> {
    match tab {
        NavTab::Home => Vec::new(),
        NavTab::Playlists => vec![false; lib.playlists.len()],
        NavTab::Tracks => vec![false; lib.tracks.len()],
        NavTab::Albums => vec![false; lib.albums.len()],
        NavTab::Artists => vec![false; lib.artists.len()],
        NavTab::Genres => vec![false; lib.genres.len()],
        NavTab::Favorites => vec![false; lib.favorites.len()],
    }
}

fn change_button(vec: &mut [bool], index: usize, visible: bool) {
    if index < vec.len() {
        vec[index] = visible;
    }
}

pub fn update(state: &mut Chilen, message: Message) -> Task<event_loop::Message> {
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
        Message::TopView(message) => return top_view::update(state, message),
        Message::ToggleSideBar => {
            state.persistent.sidebar_opened = !state.persistent.sidebar_opened;
            state.save_persistent();
        }
        Message::ToggleMenu(index) => {
            if state.main_view.button_menu == Some(index) {
                state.main_view.button_menu = None;
            } else {
                state.main_view.button_menu = Some(index);
            }
        }
    }
    Task::none()
}
