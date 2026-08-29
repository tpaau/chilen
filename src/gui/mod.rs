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
    playback::PlayerState,
};
use iced::{
    self, Element, Font, Subscription, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    stream,
    widget::{container, row},
    window::{self},
};
use iced_m3::theme::{ColorScheme, Theme};
use iced_widget::{responsive, stack};
use log::{error, trace};

use crate::{
    APP_NAME,
    gui::{dialog::Dialog, main_view::top_view},
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
    playback_view: playback_view::State,
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
            playlist_view: playlist_view::State::default(),
            playback_view: playback_view::State::default(),
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

const ROUNDING_SMALL: f32 = 14.0;
const ROUNDING_REGULAR: f32 = 14.0;
const ROUNDING_LARGE: f32 = 18.0;
const ROUNDING_LARGER: f32 = 20.0;

const PLAYBACK_DESIRED_WIDTH: f32 = 410.0;
const PLAYLIST_DESIRED_WIDTH: f32 = 350.0;
const PLAYLIST_MIN_WIDTH: f32 = 220.0;
const MAIN_MIN_WIDTH: f32 = 480.0;
const PLAYBACK_MIN_WIDTH: f32 = 320.0;
const TOTAL_MIN_WIDTH: f32 = PLAYLIST_MIN_WIDTH + MAIN_MIN_WIDTH + PLAYBACK_MIN_WIDTH;
// TODO: Should be calculated dynamically based on the window width
const MIN_HEIGHT: f32 = 1000.0;

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
        let base_content = responsive(|size| {
            let offset =
                size.width - PLAYLIST_DESIRED_WIDTH - PLAYBACK_DESIRED_WIDTH - MAIN_MIN_WIDTH;
            let ratio_base = PLAYLIST_DESIRED_WIDTH - PLAYLIST_MIN_WIDTH + PLAYBACK_DESIRED_WIDTH
                - PLAYBACK_MIN_WIDTH;
            let playlist_ratio = (PLAYLIST_DESIRED_WIDTH - PLAYLIST_MIN_WIDTH) / ratio_base;
            let playback_ratio = (PLAYBACK_DESIRED_WIDTH - PLAYBACK_MIN_WIDTH) / ratio_base;
            let playlist_width = if offset < 0.0 {
                PLAYLIST_DESIRED_WIDTH + offset * playlist_ratio
            } else {
                PLAYLIST_DESIRED_WIDTH
            };
            let playback_width = if offset < 0.0 {
                PLAYBACK_DESIRED_WIDTH + offset * playback_ratio
            } else {
                PLAYBACK_DESIRED_WIDTH
            };
            row![
                playlist_view::view(state, playlist_width).map(Message::PlaylistView),
                main_view::view(state).map(Message::MainView),
                playback_view::view(state, playback_width).map(Message::Playback),
            ]
            .into()
        });

        let base = container(base_content)
            .style(|_| container::background(state.theme.surface_container_low()));

        stack![base, dialog::view(state),].into()
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
                    chilen_backend::Event::LoadProgressChanged(_) => {}
                    chilen_backend::Event::Playback(event) => {
                        if let Some(player_state) = state.player_state.as_mut() {
                            player_state.handle_event(event);
                        } else {
                            match event {
                                chilen_backend::playback::Event::StateInitialized(player_state) => {
                                    trace!("Initializing player state representation in the GUI");
                                    state.player_state = Some(player_state);
                                }
                                _ => {
                                    error!(
                                        "Got a non-initializing event before the player state was initialized in the GUI!"
                                    );
                                }
                            }
                        }
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

pub fn start() -> iced::Result {
    trace!("Launching GUI");
    iced::application(Chilen::default, Chilen::update, Chilen::view)
        .font(icons::FILLED_ICONS_FONT_BYTES)
        .font(icons::OUTLINED_ICONS_FONT_BYTES)
        .font(font::BYTES_REGULAR)
        .font(font::BYTES_BOLD)
        .title(APP_NAME)
        .default_font(Font::with_name(font::NAME))
        .window(window::Settings {
            min_size: Some(iced::Size {
                width: TOTAL_MIN_WIDTH,
                height: MIN_HEIGHT,
            }),
            ..Default::default()
        })
        .subscription(|_| {
            Subscription::batch(vec![
                Chilen::subscription().map(Message::Event),
                window_subscription(),
            ])
        })
        .run()
}
