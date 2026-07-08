mod config;
mod playlist_view;
#[cfg(test)]
mod tests;
pub mod theme;
mod widgets;

use std::{
    collections::HashSet,
    sync::{Arc, LazyLock, RwLock},
};

use iced::{
    self, Border, Element, Font, Length, Padding, Subscription, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    stream,
    widget::{column, container, row},
    window::{self},
};
use log::{error, trace};

use crate::{
    gui::{
        config::{FontSize, Rounding, Spacing},
        theme::Theme,
    },
    music_lib::state::{MusicLibrary, Playlist},
    playback::state::PlayerState,
    settings::Settings,
};

#[derive(Debug, Clone)]
pub(super) enum Event {
    MusicLibraryChanged(Box<MusicLibrary>),
    PlayerStateChanged(PlayerState),
    LibraryLoadFailed(String),
    Quit,
    Raise,
    SetFullscreen(bool),
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
    rounding: Rounding,
    spacing: Spacing,
    font_size: FontSize,
}

impl Default for Chilen {
    fn default() -> Self {
        let settings = Settings::load();
        Self {
            playlists: HashSet::new(),
            loading_state: LoadingState::default(),
            theme: Theme::default(settings.dark_mode()),
            settings: Settings::load(),
            rounding: Rounding::default(),
            spacing: Spacing::default(),
            font_size: FontSize::default(),
        }
    }
}

pub static WINDOW_MODE: LazyLock<Arc<RwLock<Option<window::Mode>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

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
                .style(|_| container::background(state.theme.current().surface_container_low))
                .padding(Padding::new(state.spacing.small as f32))
                .width(Length::Fixed(350.0))
                .height(Length::Fill)
                .into(),
            container("Center view")
                .style(|_| {
                    container::Style::default()
                        .background(state.theme.current().background)
                        .border(Border::default().rounded(state.rounding.regular))
                })
                .padding(Padding::new(state.spacing.small as f32))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            // TODO: I should be able to resize this
            container("Currently playing")
                .style(|_| container::background(state.theme.current().surface_container_low))
                .padding(Padding::new(state.spacing.small as f32))
                .width(Length::Fixed(500.0))
                .height(Length::Fill)
                .into(),
        ])
        .into()]))
        .style(|_| container::background(state.theme.current().surface_container_low))
        .into()
    }

    fn update(state: &mut Chilen, message: Message) -> Task<Message> {
        match message {
            Message::Playlist(msg) => playlist_view::update(state, msg).map(Message::Playlist),
            Message::Event(event) => match event {
                Event::MusicLibraryChanged(lib) => playlist_view::update(
                    state,
                    playlist_view::Message::PlaylistsChanged(lib.playlists),
                )
                .map(Message::Playlist),
                Event::PlayerStateChanged(state) => todo!("Player events"),
                Event::LibraryLoadFailed(e) => {
                    state.loading_state = LoadingState::Failed(e);
                    Task::none()
                }
                Event::Quit => window::latest().and_then(window::close),
                Event::Raise => {
                    trace!("Raising!");
                    window::latest().and_then(window::gain_focus)
                }
                Event::SetFullscreen(fullscreen) => window::latest().and_then(move |id| {
                    window::set_mode(
                        id,
                        match fullscreen {
                            true => window::Mode::Fullscreen,
                            false => window::Mode::Windowed,
                        },
                    )
                }),
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

pub fn start() -> iced::Result {
    trace!("Launching GUI");
    iced::application(Chilen::default, Chilen::update, Chilen::view)
        .title("Chilen")
        .default_font(Font::with_name("Noto Sans Regular"))
        .subscription(|_| {
            Subscription::batch(vec![
                Chilen::subscription().map(Message::Event),
                window_subscription(),
            ])
        })
        .run()
}
