//! Module providing a way for the handler to send events to the daemon.
//!
//! The program launching the daemon using the [`start`](crate::start) function is its handler. The
//! handler can manage the daemon in ways other clients (programs that connect to the daemon over
//! the IPC socket) can't.
//!
//! This is because all clients can't respond to certain requests at the same time, as it would
//! result in a conflict. Let's say there are several clients all connected to the same daemon. If
//! the daemon was prompted to focus the main window of the app on the desktop, it wouldn't know
//! what to do.
//!
//! So it instead forwards that request to the handler, which can then manage it. There is always
//! only one handler, so no conflicts.
//!
//! A handler can still be a client, there is nothing preventing it from accessing the IPC. In fact,
//! this is the main way programs should communicate with the daemon, including the handler. The
//! exclusive connection is just there so that some commands that require one central entity can be
//! handled cleanly.
//!
//! If you are creating an app that connects to an external Chilen daemon as a frontend then you
//! won't be able to receive [requests](Request) from it.

pub(crate) mod state;

use std::{
    sync::{
        Arc, LazyLock, RwLock,
        mpsc::{self, Receiver},
    },
    thread,
};

use log::{error, trace, warn};
use serde::{Deserialize, Serialize};

/// Represents the fullscreen state of the handler window.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FullscreenState {
    /// The main window is currently occupying the entire screen.
    Fullscreen,
    /// The main window isn't filling the entire screen.
    Windowed,
    /// The handler doesn't have a GUI or there is no fullscreen mode (eg. in a TUI app).
    #[default]
    Unsupported,
}

/// Event sent from the handler to the daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Event {
    FullscreenChanged(FullscreenState),
}

/// Request sent from a client forwarded to the daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Request {
    /// Bring the music player’s user interface to the front using any appropriate mechanism
    /// available.
    Raise,
    /// Set whether the music player's user interface is displayed in full screen mode.
    SetFullscreen { fullscreen: bool },
}

static REQUEST_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Request>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

fn start_event_listener() {
    let (es, er) = mpsc::channel();
    *EVENT_SENDER.write().unwrap() = Some(es);

    loop {
        match er.recv() {
            Ok(event) => match event {
                Event::FullscreenChanged(state) => {
                    state::set_fullscreen(state);
                }
            },
            Err(_) => {
                trace!("The mpsc channel closed, assuming we're cleaning up");
                return;
            }
        }
    }
}

pub(crate) fn cleanup() {
    *REQUEST_SENDER.write().unwrap() = None;
    *EVENT_SENDER.write().unwrap() = None;
}

pub(crate) fn init() -> Receiver<Request> {
    let (rs, rr) = mpsc::channel();
    *REQUEST_SENDER.write().unwrap() = Some(rs);
    thread::spawn(start_event_listener);
    rr
}

// Send a request the the handler
pub(crate) fn send_request(request: Request) {
    let guard = REQUEST_SENDER.read().unwrap();
    if let Err(e) = guard.as_ref().unwrap().send(request) {
        warn!("Could not send a request to the handler (did the handler drop the receiver?): {e}");
    }
}

pub fn send_event(event: Event) -> Result<(), super::Error> {
    match EVENT_SENDER.read().unwrap().as_ref() {
        Some(sender) => {
            if let Err(e) = sender.send(event) {
                error!(
                    "Could not send the event to the daemon despite the EVENT_SENDER being initialized, this should never happen: {e}"
                )
            }
            let _ = sender.send(event);
            Ok(())
        }
        None => {
            error!("Could not send the event to the daemon as it's not running");
            Err(super::Error::DaemonNotRunning)
        }
    }
}
