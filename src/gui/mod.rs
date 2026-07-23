mod font;
mod icons;
mod playlist_view;
#[cfg(test)]
mod tests;
mod widgets;

use std::sync::{Arc, LazyLock, RwLock};

use chilen_backend::music_lib::state::MusicLibrary;
use iced::{
    self, Border, Element, Font, Length, Padding, Subscription, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    stream,
    widget::{column, container, row},
    window::{self},
};
use iced_m3::{
    theme::{ColorScheme, Theme},
    widget::dialog,
};
use iced_widget::{button, space, stack};
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
    CloseDialog,
    PlaylistName(String),
    CreatePlaylist(String),
}

#[derive(Default, Debug, Clone)]
pub enum LoadingState {
    #[default]
    Loading,
    Failed(String),
    Loaded,
}

#[derive(Default)]
enum Dialog {
    #[default]
    None,
    CreatePlaylist(String),
}

struct Chilen {
    library: Option<Box<MusicLibrary>>,
    dialog: Dialog,
    loading_state: LoadingState,
    theme: Theme,
    settings: Settings,
}

impl Default for Chilen {
    fn default() -> Self {
        let settings = Settings::load();
        Self {
            library: None,
            dialog: Dialog::default(),
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
        stack![
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
            .style(|_| container::background(state.theme.surface_container())),
            match &state.dialog {
                Dialog::None => None,
                Dialog::CreatePlaylist(name) => dialog(
                    true,
                    space().width(Length::Fill).height(Length::Fill),
                    iced_m3::widget::text_input::<_, Message>(
                        &state.library.as_ref().unwrap().get_default_playlist_name(),
                        name,
                        &state.theme,
                    )
                    .with_label_text("Playlist name", state.theme.surface_container_high())
                    .on_input(Message::PlaylistName)
                    .on_submit(Message::CreatePlaylist(name.clone())),
                    state.theme.current()
                )
                .title("Create playlist")
                .font(font::font_bold())
                .push_button(space().width(Length::Fill))
                .push_button(
                    button("Cancel")
                        .style(|_, status| iced_m3::style::button(
                            status,
                            state.theme.current(),
                            iced_m3::style::Button::Outlined
                        ))
                        .padding(12)
                        .on_press(Message::CloseDialog)
                )
                .push_button(
                    button("Confirm")
                        .style(|_, status| iced_m3::style::button(
                            status,
                            state.theme.current(),
                            iced_m3::style::Button::Primary
                        ))
                        .padding(12)
                        .on_press(Message::CreatePlaylist(name.clone()))
                )
                .width(350)
                .into(),
            },
        ]
        .into()
    }

    fn update(state: &mut Chilen, message: Message) -> Task<Message> {
        match message {
            Message::CloseDialog => state.dialog = Dialog::None,
            Message::Playlist(msg) => match msg {
                playlist_view::Message::CreatePlaylist => {
                    state.dialog = Dialog::CreatePlaylist(String::new());
                }
                _ => return playlist_view::update(state, msg).map(Message::Playlist),
            },
            Message::Event(event) => match event {
                Event::Backend(event) => match event {
                    chilen_backend::Event::LibraryChanged(lib) => {
                        state.loading_state = LoadingState::Loaded;
                        state.library = Some(lib);
                    }
                    chilen_backend::Event::PlayerStateChanged(state) => todo!("Player events"),
                    chilen_backend::Event::LibraryLoadFailed(e) => {
                        state.loading_state = LoadingState::Failed(e)
                    }
                    chilen_backend::Event::Quit => return window::latest().and_then(window::close),
                    chilen_backend::Event::Raise => {
                        trace!("Raising!");
                        return window::latest().and_then(window::gain_focus);
                    }
                    chilen_backend::Event::SetFullscreen(fullscreen) => {
                        return window::latest().and_then(move |id| {
                            window::set_mode(
                                id,
                                match fullscreen {
                                    true => window::Mode::Fullscreen,
                                    false => window::Mode::Windowed,
                                },
                            )
                        });
                    }
                },
                Event::Window { event, id } => {
                    return match event {
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
                    };
                }
            },
            Message::PlaylistName(name) => state.dialog = Dialog::CreatePlaylist(name),
            Message::CreatePlaylist(name) => {
                state.dialog = Dialog::None;
                if let Err(e) = chilen_backend::music_lib::create_playlist(name, &None) {
                    error!("Couldn't create playlist: {e}")
                }
            }
        }
        Task::none()
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
