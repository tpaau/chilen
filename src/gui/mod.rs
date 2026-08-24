mod dialog;
mod font;
mod formatter;
mod icons;
mod main_view;
mod playback_view;
mod playlist_view;
mod settings;
#[cfg(test)]
mod tests;
mod widget;

use std::sync::{Arc, LazyLock, RwLock};

use chilen_backend::{
    music_lib::{MusicLibrary, Playlist},
    playback::state::PlayerState,
};
use iced::{
    self, Element, Font, Length, Subscription, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    padding, stream,
    widget::{column, container, row},
    window::{self},
};
use iced_m3::theme::{ColorScheme, Theme};
use iced_widget::stack;
use log::{error, trace};

use crate::{
    APP_NAME,
    gui::{
        dialog::Dialog,
        font::{BYTES_BOLD, BYTES_REGULAR},
        icons::{FILLED_ICONS_FONT_BYTES, OUTLINED_ICONS_FONT_BYTES},
        main_view::top_view,
    },
    settings::Settings,
};

pub(super) use crate::gui::widget::list::THUMBNAIL_SIZE;

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
    CloseDialog,
    PlaylistNameEdited(String),
    CreatePlaylist(String),
    ImportPlaylist(Option<String>, rfd::FileHandle),
    RenamePlaylist { playlist: String, name: String },
    DeletePlaylist(Arc<Playlist>),
    MainView(main_view::Message),
    PlaylistView(playlist_view::Message),
    Settings(settings::Message),
    Playback(playback_view::Message),
}

#[derive(Default, Debug, Clone)]
pub enum LoadingState {
    #[default]
    Loading,
    Failed(String),
    Loaded,
}

struct Chilen {
    library: Option<Box<MusicLibrary>>,
    player_state: Option<PlayerState>,
    dialog: Dialog,
    loading_state: LoadingState,
    theme: Theme,
    settings: Settings,
    main_view: main_view::State,
    playlist_view: playlist_view::State,
}

impl Default for Chilen {
    fn default() -> Self {
        let settings = Settings::load();
        Self {
            library: None,
            player_state: None,
            dialog: Dialog::default(),
            loading_state: LoadingState::default(),
            theme: Theme::default(settings.theme_mode()),
            settings: Settings::load(),
            main_view: main_view::State {
                nav_stack: main_view::NavStack::default(),
                visible: None,
            },
            playlist_view: playlist_view::State { visible: None },
        }
    }
}

static WINDOW_MODE: LazyLock<Arc<RwLock<Option<window::Mode>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

const SPACING_SMALLER: f32 = 8.0;
const SPACING_SMALL: f32 = 12.0;
const SPACING_REGULAR: f32 = 16.0;
const SPACING_LARGE: f32 = 20.0;
const SPACING_LARGER: f32 = 24.0;

const ROUNDING_SMALLER: f32 = 12.0;
const ROUNDING_SMALL: f32 = 14.0;
const ROUNDING_REGULAR: f32 = 14.0;
const ROUNDING_LARGE: f32 = 18.0;
const ROUNDING_LARGER: f32 = 20.0;

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
            container(column![row![
                // TODO: I should be able to resize this
                container(playlist_view::view(state).map(Message::PlaylistView))
                    .padding(padding::horizontal(SPACING_SMALL).top(SPACING_SMALL))
                    .width(Length::Fixed(350.0))
                    .height(Length::Fill),
                main_view::view(state).map(Message::MainView),
                // TODO: I should be able to resize this
                playback_view::view(state).map(Message::Playback),
            ]])
            .style(|_| container::background(state.theme.surface_container())),
            dialog::view(state),
        ]
        .into()
    }

    fn update(state: &mut Chilen, message: Message) -> Task<Message> {
        match message {
            Message::CloseDialog => state.dialog = Dialog::None,
            Message::Event(event) => match event {
                Event::Backend(event) => match event {
                    chilen_backend::Event::LibraryChanged(lib) => {
                        state.loading_state = LoadingState::Loaded;
                        state.library = Some(lib);
                        top_view::reload(state);
                    }
                    chilen_backend::Event::PlayerStateChanged(player_state) => {
                        state.player_state = Some(player_state)
                    }
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
                    // TODO: Display the progress
                    chilen_backend::Event::LoadProgressChanged(progress) => {}
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
            Message::PlaylistNameEdited(name) => {
                state.dialog = match &state.dialog {
                    Dialog::CreatePlaylist(_) => Dialog::CreatePlaylist(name),
                    Dialog::ImportPlaylist(_, handle) => {
                        Dialog::ImportPlaylist(name, handle.clone())
                    }
                    Dialog::RenamePlaylist { playlist, name: _ } => Dialog::RenamePlaylist {
                        playlist: playlist.clone(),
                        name,
                    },
                    _ => unreachable!(),
                }
            }
            Message::CreatePlaylist(name) => {
                if let Err(e) = chilen_backend::music_lib::create_playlist(name, &None) {
                    error!("Couldn't create the playlist: {e}");
                    state.dialog = Dialog::Error(format!("Couldn't create the playlist: {e}"));
                } else {
                    state.dialog = Dialog::None;
                }
            }
            Message::ImportPlaylist(name, handle) => {
                if let Err(e) =
                    chilen_backend::music_lib::import_playlist_from_m3u8(name, &handle.into())
                {
                    error!("Could not import the playlist: {e}");
                    state.dialog = Dialog::Error(format!("Couldn't import the playlist: {e}"));
                } else {
                    state.dialog = Dialog::None;
                }
            }
            Message::RenamePlaylist { playlist, name } => {
                if let Err(e) = chilen_backend::music_lib::rename_playlist(&playlist, &name) {
                    error!("Could not rename the playlist: {e}");
                    state.dialog = Dialog::Error(format!("Couldn't rename the playlist: {e}"));
                } else {
                    state.dialog = Dialog::None;
                }
            }
            Message::DeletePlaylist(playlist) => {
                if let Err(e) =
                    chilen_backend::music_lib::delete_playlists(vec![playlist.name.clone()])
                {
                    error!("Couldn't delete playlist: {e}");
                    state.dialog = Dialog::Error(format!("Couldn't delete playlist: {e}"))
                } else {
                    state.dialog = Dialog::None;
                }
            }
            Message::MainView(msg) => {
                return main_view::update(state, msg).map(Message::MainView);
            }
            Message::PlaylistView(msg) => {
                return playlist_view::update(state, msg).map(Message::PlaylistView);
            }
            Message::Settings(msg) => return settings::update(state, msg).map(Message::Settings),
            Message::Playback(message) => {
                return playback_view::update(state, message).map(Message::Playback);
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
        iced::font::load(FILLED_ICONS_FONT_BYTES).discard(),
        iced::font::load(OUTLINED_ICONS_FONT_BYTES).discard(),
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
    .title(APP_NAME)
    .default_font(Font::with_name("Noto Sans"))
    .subscription(|_| {
        Subscription::batch(vec![
            Chilen::subscription().map(Message::Event),
            window_subscription(),
        ])
    })
    .run()
}
