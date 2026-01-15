mod daemon_thread;
mod cache;
pub mod track;

use std::{
    process::exit,
    sync::{
        mpsc, Arc, LazyLock, RwLock
    },
    thread,
};

use interprocess::local_socket::{ListenerOptions, Stream, traits::ListenerExt};
use log::{debug, error, info, trace, warn};

use mpipc::{ClientCommand, DaemonError, SOCKET_NAME, get_daemon_socket};
use smol::channel::unbounded;

use crate::cache::music_lib;

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<DaemonEvent>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

#[derive(Debug)]
/// Parsed CLI arguments for the daemon.
pub enum DaemonCommand {
    /// Start the daemon. This is not to be sent over the daemon socket.
    Start,
    /// Command for the daemon instance.
    Message {
        /// The command content meant to be sent over the daemon socket.
        command: ClientCommand,
    },
}

impl TryFrom<DaemonCommand> for ClientCommand {
    type Error = String;
    fn try_from(value: DaemonCommand) -> Result<Self, Self::Error> {
        match value {
            DaemonCommand::Start => Err(String::from(
                "Daemon start command cannot be converted to a client command",
            )),
            DaemonCommand::Message { command } => Ok(command),
        }
    }
}

#[derive(Debug)]
pub enum DaemonEvent {
    Shutdown,
    Restart,
}

fn handle_error(conn: std::io::Result<Stream>) -> Option<Stream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            warn!("Incoming connection failed: {e}");
            None
        }
    }
}

pub fn send_event(event: DaemonEvent) -> Result<(), String> {
    trace!("Sending an event to the daemon: {event:?}");
    match EVENT_SENDER.read().as_mut() {
        Ok(guard) => match guard.clone().unwrap().send(event) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub fn start() -> Result<(), DaemonError> {
    debug!("Starting daemon on '{SOCKET_NAME}'");

    let socket = match get_daemon_socket() {
        Ok(sock) => sock,
        Err(e) => {
            error!("Could not obtain a socket: {e}");
            return Err(DaemonError::SocketError {
                error: e.to_string(),
            });
        }
    };

    trace!("Creating a listener on '{SOCKET_NAME}'");
    let opts = ListenerOptions::new().name(socket.clone());

    let listener = match opts.create_sync() {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed creating a listener for the daemon socket: '{e}'");
            return Err(DaemonError::SocketError {
                error: e.to_string(),
            });
        }
    };

    thread::spawn(|| music_lib::load(music_lib::LoadMode::Initialize));

    thread::spawn(move || {
        info!("Listening for incomming connections");
        let mut senders = Vec::new();
        for conn in listener.incoming().filter_map(handle_error) {
            let (ttx, trx) = unbounded();
            senders.push(ttx);
            daemon_thread::spawn(conn, trx);
        }
    });

    let (event_sender, event_receiver) = mpsc::channel();
    let mut guard = EVENT_SENDER.write().unwrap();
    *guard = Some(event_sender);
    drop(guard);

    loop {
        match event_receiver.recv().unwrap() {
            DaemonEvent::Shutdown => {
                trace!("Received shutdown event");
                exit(0);
            }
            DaemonEvent::Restart => {}
        }
    }
}
