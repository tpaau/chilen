mod daemon_thread;
pub mod data;
mod playback;
#[cfg(test)]
mod tests;

use std::{
    env::home_dir,
    path::PathBuf,
    process::exit,
    sync::{
        Arc, LazyLock, RwLock,
        mpsc::{self, channel},
    },
    thread,
};

use interprocess::local_socket::{ListenerOptions, Stream, traits::ListenerExt};
use log::{debug, error, info, trace, warn};

use mpipc::{
    ClientCommand, DEFAULT_SOCKET_NAME, DaemonError, DaemonEvent, DataError, get_daemon_socket,
};
use serde::{Deserialize, Serialize};

use crate::data::{
    CACHE_DIR,
    music_lib::{self, LoadMode},
    set_data_dirs,
};

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<DaemonEvent>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Daemon config that can be passed to the `start` function.
pub struct Config {
    /// The directory containing daemon cache.
    cache_dir: PathBuf,
    /// The directory containing the program data, eg. the playlist file.
    data_dir: PathBuf,
    /// The directory containing the music library/audio files to load.
    music_dir: PathBuf,
    /// The name of the socket to listen on.
    socket_name: String,
}

impl Config {
    /// Get the default daemon config when running headless.
    ///
    /// For a custom music player, you probably want to create your own config from
    /// scratch, or use the `Config::try_from_name(...)` constructor.
    pub fn try_default() -> Result<Self, DataError> {
        Self::try_from_name(
            String::from("music-player"),
            String::from(DEFAULT_SOCKET_NAME),
        )
    }

    pub fn new(
        cache_dir: PathBuf,
        data_dir: PathBuf,
        music_dir: PathBuf,
        socket_name: String,
    ) -> Self {
        Self {
            cache_dir,
            data_dir,
            music_dir,
            socket_name,
        }
    }

    /// Create a new config from program and socket names.
    ///
    /// The generated paths will contain the name of the program, eg. `~/.cache/<PROGRAM_NAME>`,
    /// ``~/.local/share/<PROGRAM_NAME>`.
    ///
    /// Fails if the home directory is not available.
    pub fn try_from_name(name: String, socket_name: String) -> Result<Self, DataError> {
        let home_dir = match home_dir() {
            Some(home) => home,
            None => {
                return Err(DataError::HomeError);
            }
        };

        let mut cache_dir = home_dir.clone();
        cache_dir.push(".cache");
        cache_dir.push(&name);

        let mut data_dir = home_dir.clone();
        data_dir.push(".local/share");
        data_dir.push(name);

        let mut music_dir = home_dir.clone();
        music_dir.push("Music");

        Ok(Config {
            cache_dir,
            music_dir,
            data_dir,
            socket_name,
        })
    }
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

fn handle_error(conn: std::io::Result<Stream>) -> Option<Stream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            warn!("Incoming connection failed: {e}");
            None
        }
    }
}

pub(crate) fn send_event(event: DaemonEvent) -> Result<(), String> {
    match EVENT_SENDER.read().as_mut() {
        Ok(guard) => match guard.clone().unwrap().send(event) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Could not send the event to the daemon: {e}");
                Err(e.to_string())
            }
        },
        Err(e) => Err(e.to_string()),
    }
}

/// Start the daemon with the given config.
///
/// # Examples
/// ```no_run
/// # use daemon;
/// // You probably want to create a custom config from scratch
/// let config = daemon::Config::try_default().unwrap();
/// daemon::start(config).unwrap();
/// ```
pub fn start(config: Config) -> Result<(), DaemonError> {
    debug!("Starting daemon on '{}'", config.socket_name);

    if let Err(e) = set_data_dirs(config.clone()) {
        error!("Could not set the paths: {e}");
        return Err(DaemonError::DataError(e));
    }

    if config.socket_name == mpipc::DEFAULT_SOCKET_NAME {
        warn!(
            "Using the default IPC socket name. Please use a unique name outside of just testing."
        );
    }

    let socket = match get_daemon_socket(&config.socket_name) {
        Ok(sock) => sock,
        Err(e) => {
            error!("Could not obtain a socket: {e}");
            return Err(DaemonError::SocketError);
        }
    };

    trace!("Creating a listener on '{}'", config.socket_name);
    let opts = ListenerOptions::new().name(socket.clone());

    let listener = match opts.create_sync() {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed creating a listener for the daemon socket: '{e}'");
            return Err(DaemonError::ListenerError);
        }
    };

    thread::spawn(|| music_lib::load(LoadMode::Initialize));

    let senders = Arc::new(RwLock::new(Vec::new()));

    thread::spawn({
        let senders_clone = senders.clone();
        info!("Listening for incomming connections");
        move || {
            for (index, conn) in listener.incoming().filter_map(handle_error).enumerate() {
                let (ttx, trx) = channel();
                senders_clone.clone().write().unwrap().push(ttx);
                // Size for `u64` is 8 bytes, and for `usize` it's 4 bytes on 32-bit and 8
                // bytes on 64-bit, so the conversion can be done safely if I understand this
                // correctly :)
                daemon_thread::spawn(conn, trx, u64::try_from(index).unwrap());
            }
        }
    });

    let (event_sender, event_receiver) = mpsc::channel();
    let mut guard = EVENT_SENDER.write().unwrap();
    *guard = Some(event_sender);
    drop(guard);

    loop {
        let event = event_receiver.recv().unwrap();
        let mut guard = senders.write().unwrap();
        let mut dead = Vec::new();
        for (i, sender) in guard.iter().enumerate() {
            if sender.send(event.clone()).is_err() {
                debug!("Removing dead connection at {}", i);
                dead.push(i);
            }
        }
        for &ded in dead.iter().rev() {
            guard.remove(ded);
        }

        match event {
            DaemonEvent::Shutdown => {
                trace!("Received shutdown event");
                exit(0);
            }
            DaemonEvent::ConnectionClosed => {
                trace!("Connection with a client closed")
            }
            _ => {}
        }
    }
}
