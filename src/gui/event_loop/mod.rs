pub mod library;
pub mod playback;

use std::thread;

use iced::{
    Subscription, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    keyboard::{self, key},
    stream, window,
};
use log::{error, trace};

use crate::gui::{
    Chilen, EVENT_SENDER, LoadingState, WINDOW_MODE,
    dialog::{self, Dialog},
    main_view::{self},
    persistent_state, playback_view, settings,
};

#[derive(Debug, Clone)]
pub enum Event {
    Backend(chilen_backend::Event),
    Window {
        event: window::Event,
        id: window::Id,
    },
    SystemThemeChanged(iced::theme::Mode),
}

// TODO: A bunch or `Arc<WhateverLibraryItem>` types are in here. I should rather pass IDs of those
// items in the library, that way we only have 8 bytes allocated on the stack per item.
#[derive(Clone)]
pub enum Message {
    Settings(settings::Message),
    Playback(playback::Message),
    Library(library::Message),
    PlaybackView(playback_view::Message),
    Dialog(dialog::Message),
    MainView(main_view::Message),
    Event(Event),
    /// Opens a URL in the browser
    OpenURL(String),
    EscapePressed,
}

fn backend_events_worker() -> impl Stream<Item = Event> {
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

fn backend_events() -> Subscription<Event> {
    Subscription::run(backend_events_worker)
}

fn theme_changes() -> Subscription<Message> {
    iced::system::theme_changes().map(|t| Message::Event(Event::SystemThemeChanged(t)))
}

fn window_events() -> Subscription<Message> {
    window::events().map(move |(id, event)| Message::Event(Event::Window { event, id }))
}

fn keyboard_shortcuts() -> Subscription<Message> {
    iced::event::listen_with(|event, _, _| {
        if matches!(
            event,
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                modified_key: _,
                physical_key: _,
                location: _,
                modifiers: _,
                text: _,
                repeat: _
            })
        ) {
            Some(Message::EscapePressed)
        } else {
            None
        }
    })
}

pub(super) fn subscription() -> Subscription<Message> {
    Subscription::batch(vec![
        backend_events().map(Message::Event),
        window_events(),
        theme_changes(),
        keyboard_shortcuts(),
    ])
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::Settings(message) => {
            return settings::update(state, message);
        }
        Message::Playback(message) => {
            return playback::update(state, message);
        }
        Message::Library(message) => return library::update(state, message),
        Message::PlaybackView(message) => {
            return playback_view::update(state, message);
        }
        Message::Dialog(message) => return dialog::update(state, message),
        Message::MainView(message) => {
            return main_view::update(state, message);
        }
        Message::Event(event) => match event {
            Event::Backend(event) => match event {
                chilen_backend::Event::LibraryChanged(lib) => {
                    state.loading_state = LoadingState::Loaded;
                    state.library = Some(lib);
                    persistent_state::library_loaded(state);
                    main_view::reload(state);
                }
                chilen_backend::Event::LibraryLoadFailed(_) => {
                    state.loading_state = LoadingState::Failed
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
                chilen_backend::Event::LoadProgressChanged(progress) => match progress {
                    chilen_backend::music_lib::Progress::Done => {
                        if let crate::gui::dialog::Dialog::Loading(_) = state.dialog {
                            state.dialog = crate::gui::dialog::Dialog::None;
                        }
                    }
                    chilen_backend::music_lib::Progress::Failed(error) => {
                        state.dialog = crate::gui::dialog::Dialog::Error(format!(
                            "Couldn't load the music library: {error}"
                        ))
                    }
                    _ => state.dialog = crate::gui::dialog::Dialog::Loading(Some(progress)),
                },
                chilen_backend::Event::Playback(event) => playback_view::handle_event(state, event),
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
            Event::SystemThemeChanged(mode) => {
                state.host_theme_preference = mode.into();
                if state.settings.theme_auto_mode {
                    state.theme.mode = mode.into();
                }
            }
        },
        Message::OpenURL(url) => {
            thread::spawn(move || {
                if let Err(e) = open::that(&url) {
                    error!("Failed to open URL \"{url}\": {e}");
                }
            });
        }
        Message::EscapePressed => {
            state.playback_view.button_menu = None;
            state.main_view.button_menu = None;
            state.main_view.top_view.main_menu_opened = false;
            state.main_view.button_menu = None;
            if !matches!(state.dialog, Dialog::None) {
                state.dialog = Dialog::None;
            } else {
                state.settings_state.screen = None;
            }
        }
    }
    Task::none()
}
