mod font;
mod icons;
mod playlist_view;
pub mod styles;
#[cfg(test)]
mod tests;
mod widgets;

use std::{
    collections::HashSet,
    sync::{Arc, LazyLock, RwLock},
};

use chilen_backend::music_lib::state::Playlist;
use iced::{
    self, Border, Element, Font, Length, Padding, Subscription, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    stream,
    widget::{column, container, row},
    window::{self},
};
use iced_m3::theme::{ColorScheme, Theme};
use log::{error, trace};

use crate::{
    gui::{
        font::{BYTES_BOLD, BYTES_REGULAR},
        icons::ICONS_FONT_BYTES,
    },
    settings::Settings,
};

#[derive(Debug, Clone)]
pub(super) enum Event {
    Backend(chilen_backend::Event),
    Window {
        event: window::Event,
        id: window::Id,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),
    Playlist(playlist_view::Message),
}

#[derive(Default, Debug, Clone)]
pub enum LoadingState {
    #[default]
    Loading,
    Failed(String),
    Loaded,
}

struct Chilen {
    playlists: HashSet<Arc<Playlist>>,
    loading_state: LoadingState,
    theme: Theme,
    settings: Settings,
}

impl Default for Chilen {
    fn default() -> Self {
        let settings = Settings::load();
        Self {
            playlists: HashSet::new(),
            loading_state: LoadingState::default(),
            theme: Theme::default(settings.dark_mode()),
            settings: Settings::load(),
        }
    }
}

static WINDOW_MODE: LazyLock<Arc<RwLock<Option<window::Mode>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

const SPACING_SMALLER: u32 = 8;
const SPACING_SMALL: u32 = 12;
const SPACING_REGULAR: u32 = 16;
const SPACING_LARGE: u32 = 20;
const SPACING_LARGER: u32 = 24;

const ROUNDING_SMALLER: u32 = 12;
const ROUNDING_SMALL: u32 = 14;
const ROUNDING_REGULAR: u32 = 14;
const ROUNDING_LARGE: u32 = 18;
const ROUNDING_LARGER: u32 = 20;

const DIM_TEXT_ALPHA: f32 = 0.7;

pub fn event_sender_initialized() -> bool {
    EVENT_SENDER.read().unwrap().is_some()
}

pub fn send_event(event: Event) {
    match EVENT_SENDER.write().unwrap().as_mut() {
        Some(sender) => {
            if let Err(e) = sender.try_send(event) {
                error!("Could not send the event: {e}");
            }
        }
        None => {
            error!("The sender is not initialized!")
        }
    }
}

fn window_subscription() -> Subscription<Message> {
    window::events().map(|(id, event)| Message::Event(Event::Window { event, id }))
}

impl Chilen {
    fn view(state: &Chilen) -> Element<'_, Message> {
        container(column([row([
            // TODO: I should be able to resize this
            container(playlist_view::view(state).map(Message::Playlist))
                .padding(Padding::new(SPACING_SMALL as f32))
                .width(Length::Fixed(350.0))
                .height(Length::Fill)
                .into(),
            container("Main view")
                .style(|_| {
                    container::Style::default()
                        .background(state.theme.background())
                        .border(Border::default().rounded(ROUNDING_REGULAR))
                })
                .padding(Padding::new(SPACING_SMALL as f32))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            // TODO: I should be able to resize this
            container("Currently playing")
                .padding(Padding::new(SPACING_SMALL as f32))
                .width(Length::Fixed(500.0))
                .height(Length::Fill)
                .into(),
        ])
        .into()]))
        .style(|_| container::background(state.theme.surface_container()))
        .into()
    }

    fn update(state: &mut Chilen, message: Message) -> Task<Message> {
        match message {
            Message::Playlist(msg) => playlist_view::update(state, msg).map(Message::Playlist),
            Message::Event(event) => match event {
                Event::Backend(event) => match event {
                    chilen_backend::Event::LibraryChanged(lib) => playlist_view::update(
                        state,
                        playlist_view::Message::PlaylistsChanged(lib.playlists),
                    )
                    .map(Message::Playlist),
                    chilen_backend::Event::PlayerStateChanged(state) => todo!("Player events"),
                    chilen_backend::Event::LibraryLoadFailed(e) => {
                        state.loading_state = LoadingState::Failed(e);
                        Task::none()
                    }
                    chilen_backend::Event::Quit => window::latest().and_then(window::close),
                    chilen_backend::Event::Raise => {
                        trace!("Raising!");
                        window::latest().and_then(window::gain_focus)
                    }
                    chilen_backend::Event::SetFullscreen(fullscreen) => {
                        window::latest().and_then(move |id| {
                            window::set_mode(
                                id,
                                match fullscreen {
                                    true => window::Mode::Fullscreen,
                                    false => window::Mode::Windowed,
                                },
                            )
                        })
                    }
                },
                Event::Window { event, id } => match event {
                    window::Event::Opened {
                        position: _,
                        size: _,
                    } => window::mode(id).then(|mode| {
                        *WINDOW_MODE.write().unwrap() = Some(mode);
                        Task::none()
                    }),
                    window::Event::Resized(_) => window::mode(id).then(|mode| {
                        *WINDOW_MODE.write().unwrap() = Some(mode);
                        Task::none()
                    }),
                    _ => Task::none(),
                },
            },
        }
    }

    fn worker() -> impl Stream<Item = Event> {
        stream::channel(128, async |mut out| {
            let (sender, mut receiver) = mpsc::channel(128);
            *EVENT_SENDER.write().unwrap() = Some(sender);

            loop {
                let input = receiver.select_next_some().await;
                if let Err(e) = out.send(input).await {
                    error!("Could not send the event, aborting: {e}");
                    break;
                }
            }
        })
    }

    fn subscription() -> Subscription<Event> {
        Subscription::run(Self::worker)
    }
}

fn load_fonts() -> Task<Message> {
    trace!("Loading fonts...");
    Task::batch([
        iced::font::load(ICONS_FONT_BYTES).discard(),
        iced::font::load(BYTES_REGULAR).discard(),
        iced::font::load(BYTES_BOLD).discard(),
    ])
}

pub fn start() -> iced::Result {
    trace!("Launching GUI");
    iced::application(
        || (Chilen::default(), load_fonts()),
        Chilen::update,
        Chilen::view,
    )
    .title("Chilen")
    .default_font(Font::with_name("Noto Sans"))
    .subscription(|_| {
        Subscription::batch(vec![
            Chilen::subscription().map(Message::Event),
            window_subscription(),
        ])
    })
    .run()
}
