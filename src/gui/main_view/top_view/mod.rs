mod album;
mod artist;
mod genre;
mod playlist;

use std::sync::Arc;

use chilen_backend::music_lib::{Album, Artist, Genre, Playlist};
use iced::{Element, Length, Pixels, Task};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::{Accent, ColorScheme},
    widget::{
        advanced::menu_overlay,
        button::{self, Content},
        button_group::{self, standard::GroupItem},
        hybrid_icon::Icon,
        standard_button_group,
    },
};
use iced_widget::{container, responsive, scrollable};

use crate::gui::{
    Chilen, SPACING_SMALLER, event_loop,
    font::bold_text,
    icons,
    main_view::{self, top_view},
    menu_close,
};

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
    Navigate(TopView),
    Unwind,
    OpenMainMenu,
    CloseMainMenu,
    ToggleMenu(usize),
}

fn title<'a>(theme: &'a impl ColorScheme, content: String) -> Element<'a, event_loop::Message> {
    bold_text(content)
        .size(32.0)
        .color(theme.on_surface())
        .into()
}

#[derive(Default)]
pub struct State {
    pub main_menu_opened: bool,
}

fn horizontal_buttons<'a>(
    state: &'a Chilen,
    message_play: Option<event_loop::Message>,
    message_shuffle: Option<event_loop::Message>,
    menu_surface: impl Fn() -> Element<'a, event_loop::Message> + 'a,
) -> Element<'a, event_loop::Message> {
    let menu_surface = Arc::new(menu_surface);
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
            GroupItem::new(
                iced_m3::widget::button(
                    button::Style::tonal(&state.theme, Accent::Tertiary),
                    Content::Icon(Icon::Text {
                        text: icons::ARROW_BACK.into_fragment(),
                        font: Some(icons::filled()),
                    }),
                )
                .size(size)
                .on_press(event_loop::Message::MainView(
                    main_view::Message::TopView(top_view::Message::Unwind),
                )),
                true,
            ),
            GroupItem::new(
                iced_m3::widget::button(
                    button::Style::tonal(&state.theme, Accent::Secondary),
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
                message_play.is_some(),
            ),
            GroupItem::new(
                iced_m3::widget::button(
                    button::Style::filled(&state.theme, Accent::Primary),
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
                message_shuffle.is_some(),
            ),
            {
                GroupItem::new(
                    menu_overlay(
                        iced_m3::widget::button(
                            button::Style::outlined(&state.theme),
                            Content::Icon(Icon::Text {
                                text: icons::MORE_HORIZ.into_fragment(),
                                font: Some(icons::filled()),
                            }),
                        )
                        .size(size)
                        .selected(state.main_view.top_view.main_menu_opened)
                        .on_press(event_loop::Message::MainView(main_view::Message::TopView(
                            top_view::Message::OpenMainMenu,
                        ))),
                        {
                            let surface = Arc::clone(&menu_surface);
                            move || surface()
                        },
                        state.main_view.top_view.main_menu_opened,
                    )
                    .placement(menu_overlay::Placement::BottomRight {
                        padding: Pixels(SPACING_SMALLER),
                    })
                    .on_press(|p| {
                        menu_close(
                            p,
                            event_loop::Message::MainView(main_view::Message::TopView(
                                top_view::Message::CloseMainMenu,
                            )),
                        )
                    }),
                    false,
                )
            },
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

pub fn view<'a>(state: &'a Chilen, view: TopView) -> Element<'a, event_loop::Message> {
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

pub fn update(state: &mut Chilen, message: Message) -> Task<event_loop::Message> {
    state.main_view.top_view.main_menu_opened = false;
    match message {
        Message::Navigate(top_view) => state.main_view.nav_stack.navigate(top_view),
        Message::Unwind => {
            state.main_view.nav_stack.unwind();
        }
        Message::OpenMainMenu => state.main_view.top_view.main_menu_opened = true,
        Message::CloseMainMenu => state.main_view.top_view.main_menu_opened = false,
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
