//! Test description of the module.
use std::{
    sync::{
        Arc, LazyLock, RwLock,
        mpsc::{self, Receiver},
    },
    thread,
};

use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize, ser::Error};

/// Something that happened on the side of the handler that the daemon should be aware of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Event {
    FullscreenChanged { is_fullscreen: bool },
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
            Ok(event) => todo!("Handle events"),
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
