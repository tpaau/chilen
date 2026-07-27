mod dialog;
mod font;
mod icons;
mod playlist_view;
#[cfg(test)]
mod tests;
mod widgets;

use std::{
    env::home_dir,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
};

use chilen_backend::music_lib::state::{MusicLibrary, Playlist};
use iced::{
    self, Border, Element, Font, Length, Padding, Subscription, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    stream,
    widget::{column, container, row},
    window::{self},
};
use iced_m3::theme::{ColorScheme, Theme};
use iced_widget::stack;
use log::{error, info, trace};

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
    CloseDialog,
    OpenPlaylist(Arc<Playlist>),
    PlaylistNameEdited(String),
    OpenPlaylistCreationDialog,
    CreatePlaylist(String),
    OpenPlaylistImportFilePicker,
    OpenPlaylistImportDialog(Option<rfd::FileHandle>),
    ImportPlaylist(Option<String>, rfd::FileHandle),
    OpenPlaylistRenameDialog {
        playlist: Arc<Playlist>,
        name: String,
    },
    RenamePlaylist {
        playlist: Arc<Playlist>,
        name: String,
    },
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
    ImportPlaylist(String, rfd::FileHandle),
    Error(String),
    RenamePlaylist {
        playlist: Arc<Playlist>,
        name: String,
    },
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
                container(playlist_view::view(state))
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
            Message::OpenPlaylistImportDialog(handle) => {
                if let Some(handle) = handle {
                    trace!("Showing import dialog for playlist {handle:?}");
                    state.dialog = Dialog::ImportPlaylist(String::new(), handle);
                } else {
                    info!("Didn't get the file handle!");
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
            Message::OpenPlaylist(pl) => todo!(),
            Message::OpenPlaylistCreationDialog => {
                state.dialog = Dialog::CreatePlaylist(String::new());
            }
            Message::OpenPlaylistImportFilePicker => {
                return Task::perform(
                    rfd::AsyncFileDialog::new()
                        .add_filter("M3U8 Playlist File", &["m3u", "m3u8"])
                        .set_directory(home_dir().unwrap_or(PathBuf::from(".")))
                        .pick_file(),
                    Message::OpenPlaylistImportDialog,
                );
            }
            Message::OpenPlaylistRenameDialog { playlist, name } => {
                state.dialog = Dialog::RenamePlaylist { playlist, name }
            }
            Message::RenamePlaylist { playlist, name } => todo!(),
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
