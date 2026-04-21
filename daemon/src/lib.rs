mod daemon_thread;
pub mod data;
#[cfg(feature = "mpris")]
mod mpris;
pub mod playback;
#[cfg(test)]
mod tests;

use std::{
    env::home_dir,
    fs::remove_file,
    path::PathBuf,
    process::exit,
    sync::{
        Arc, LazyLock, RwLock,
        mpsc::{self, channel},
    },
    thread,
};

use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, NameType, Stream, traits::ListenerExt,
};
use log::{debug, error, info, trace, warn};

use mpipc::SocketType;
use mpipc::{
    ClientCommand, ConfigError, DEFAULT_SOCKET_NAME, DaemonError, DaemonEvent, DaemonResponse,
    exec_client_command, get_daemon_socket,
};
use serde::{Deserialize, Serialize};

use crate::data::{
    CACHE_DIR,
    music_lib::{self, LoadMode},
    set_data_dirs,
};

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<DaemonEvent>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Defines under which conditions should the daemon claim an occupied socket address.
///
/// This is only effective if filesystem sockets are used.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddrClaimMode {
    /// Do not claim the socket address under any circumstances.
    DoNotClaim,
    /// Only claim the socket address if there is no response to ping requests from the process
    /// that listens of the socket.
    #[default]
    ClaimIfUnresponsive,
    /// Force claim the socket address.
    ForceClaim,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Daemon config that can be passed to the `start` function.
pub struct Config {
    /// The directory containing daemon cache.
    pub cache_dir: PathBuf,
    /// The directory containing the program data, eg. the playlist file.
    pub data_dir: PathBuf,
    /// The directory containing the music library/audio files to load.
    pub music_dir: PathBuf,
    /// The name of the socket to listen on.
    pub socket_name: String,
    /// Defines under which conditions should the daemon claim an occupied socket address.
    pub addr_claim_mode: AddrClaimMode,
    /// Defines the type of socket the daemon should use.
    pub socket_type: SocketType,
    pub playback_config: playback::Config,
}

impl Config {
    /// Get the default daemon config.
    ///
    /// For a custom music player, you probably want to create your own config from
    /// scratch, or use the `Config::try_from_name(...)` constructor.
    ///
    /// # Examples
    /// ```
    /// # use daemon;
    /// let conf = daemon::Config::try_default().unwrap();
    /// ```
    pub fn try_default() -> Result<Self, ConfigError> {
        Self::try_from_name("music-player".to_string(), DEFAULT_SOCKET_NAME)
    }

    /// Create a new config from program and socket names.
    ///
    /// The generated paths will contain the name of the program, eg. `~/.cache/<PROGRAM_NAME>`,
    /// ``~/.local/share/<PROGRAM_NAME>`.
    ///
    /// Fails if the home directory is not available.
    ///
    /// # Examples
    /// ```
    /// # use daemon;
    /// let conf = daemon::Config::try_from_name("my-player".to_string(), "MY_PLAYER");
    /// ```
    pub fn try_from_name(name: String, socket_name: &str) -> Result<Self, ConfigError> {
        let home_dir = match home_dir() {
            Some(home) => home,
            None => {
                return Err(ConfigError::HomeError);
            }
        };

        let mut cache_dir = home_dir.clone();
        cache_dir.push(".cache");
        cache_dir.push(&name);

        let mut data_dir = home_dir.clone();
        data_dir.push(".local/share");
        data_dir.push(&name);

        let mut music_dir = home_dir.clone();
        music_dir.push("Music");

        #[cfg(feature = "mpris")]
        let bus_name_suffix = String::from("com.dev.") + &name;
        #[cfg(feature = "mpris")]
        if !bus_name_suffix.is_ascii() {
            return Err(ConfigError::InvalidBusNameSuffix);
        }

        Ok(Config {
            cache_dir,
            music_dir,
            data_dir,
            socket_name: socket_name.to_string(),
            addr_claim_mode: AddrClaimMode::default(),
            socket_type: SocketType::default(),
            playback_config: playback::Config {
                #[cfg(feature = "mpris")]
                identity: name.to_string(),
                #[cfg(feature = "mpris")]
                bus_name_suffix,
                allow_rate_modification: false,
            },
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

    let socket = match get_daemon_socket(&config.socket_name, &config.socket_type) {
        Ok(sock) => sock,
        Err(e) => {
            error!("Could not obtain the socket: {e}");
            return Err(DaemonError::SocketError);
        }
    };

    let opts = ListenerOptions::new().name(socket.clone());
    if socket.is_namespaced() {
        trace!(
            "Creating a listener on \"{}\" (namespaced socket)",
            config.socket_name
        );
    } else {
        trace!(
            "Creating a listener on \"{}\" (filesystem socket)",
            config.socket_name
        );
    }

    let listener = match opts.create_sync() {
        Ok(listener) => listener,
        Err(e) => {
            if (!GenericNamespaced::is_supported()
                || config.socket_type == SocketType::FilesystemOnly)
                && e.kind() == std::io::ErrorKind::AddrInUse
            {
                warn!("The socket address is already in use");

                match config.addr_claim_mode {
                    AddrClaimMode::DoNotClaim => {
                        warn!("The daemon is configured to not reclaim the socket address");
                        return Err(DaemonError::AddrInUse);
                    }
                    AddrClaimMode::ClaimIfUnresponsive => {
                        info!("Attempting to claim the socket address if it appears to be unused");
                        match exec_client_command(
                            ClientCommand::Ping,
                            &config.socket_name,
                            &config.socket_type,
                        ) {
                            Ok(response) => {
                                if response == DaemonResponse::Pong {
                                    error!(
                                        "The other deamon responded to the pong command, not claiming the address"
                                    );
                                    return Err(DaemonError::AddrInUse);
                                } else {
                                    error!(
                                        "Got an unexpected response from the deamon: {response:?}"
                                    );
                                    return Err(DaemonError::UnexpectedResponse);
                                }
                            }
                            Err(e) => {
                                if e == DaemonError::ConnectionError {
                                    info!(
                                        "The other daemon is either dead or unresponsive, claiming the address"
                                    );
                                } else {
                                    error!(
                                        "Got an unexpected error while trying to send a ping command: {e}"
                                    );
                                    return Err(DaemonError::UnexpectedResponse);
                                }
                            }
                        }

                        if let Err(e) = remove_file(mpipc::get_fs_socket_path(&config.socket_name))
                        {
                            error!("Could not remove the old socket: {e}");
                            return Err(DaemonError::SocketError);
                        }
                        let opts = ListenerOptions::new().name(socket.clone());
                        match opts.create_sync() {
                            Ok(listener) => {
                                info!("Succesfully claimed the address");
                                listener
                            }
                            Err(e) => {
                                error!(
                                    "Could not create a listener despite claiming the socket: {e}"
                                );
                                return Err(DaemonError::SocketError);
                            }
                        }
                    }
                    AddrClaimMode::ForceClaim => {
                        info!("Force claiming the socket address");
                        if let Err(e) = remove_file(mpipc::get_fs_socket_path(&config.socket_name))
                        {
                            error!("Could not remove the old socket: {e}");
                            return Err(DaemonError::SocketError);
                        }
                        let opts = ListenerOptions::new().name(socket.clone());
                        match opts.create_sync() {
                            Ok(listener) => {
                                info!("Succesfully claimed the address");
                                listener
                            }
                            Err(e) => {
                                error!(
                                    "Could not create a listener despite claiming the socket: {e}"
                                );
                                return Err(DaemonError::SocketError);
                            }
                        }
                    }
                }
            } else {
                error!("Failed creating a listener for the daemon socket: '{e}'");
                return Err(DaemonError::SocketError);
            }
        }
    };

    thread::spawn(move || {
        let _ = music_lib::load(LoadMode::Initialize);
        playback::init(config.playback_config);
    });

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
