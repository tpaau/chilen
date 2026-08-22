mod albums;
mod artists;
mod genres;
pub mod top_view;
mod tracks;

use chilen_backend::music_lib::MusicLibrary;
use iced::{Alignment, Border, Element, Length, Task, padding};
use iced_m3::theme::ColorScheme;
use iced_widget::{center, column, container, row, sensor, space, stack, text};
use log::trace;

use crate::gui::{
    self, Chilen, ROUNDING_REGULAR, SPACING_REGULAR, SPACING_SMALL, icons,
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
                    .find_playlist(&playlist.name)
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
}

fn virtualize_entry<'a, E>(
    state: &'a Chilen,
    content: impl Fn() -> E + 'a,
    height: f32,
    index: usize,
) -> Element<'a, main_view::Message>
where
    E: Into<Element<'a, main_view::Message>>,
{
    let content: Element<'a, main_view::Message> = if let Some(visible) = &state.main_view.visible
        && index < visible.len()
        && visible[index]
    {
        (content)().into()
    } else {
        space().height(height).width(Length::Fill).into()
    };
    sensor(content)
        .on_show(move |_| main_view::Message::ButtonPoppedIn(index))
        .on_hide(main_view::Message::ButtonPoppedOut(index))
        .into()
}

pub fn view(state: &Chilen) -> Element<'_, main_view::Message> {
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
            if let Some(lib) = &state.library {
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
                center(content)
            } else {
                center(text("Loading..."))
            }
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
    }
    Task::none()
}
