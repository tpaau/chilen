#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![feature(doc_cfg)]

mod daemon_thread;
mod music_lib;
pub mod playback;
#[cfg(test)]
mod tests;

use std::{
    env::home_dir,
    fs::remove_file,
    path::PathBuf,
    sync::{
        Arc, LazyLock, RwLock,
        mpsc::{self, channel},
    },
    thread,
};

use interprocess::local_socket::{Listener, ListenerOptions, Stream, traits::ListenerExt};
use log::{debug, error, info, trace, warn};

pub use mpipc;
use mpipc::{Command, DEFAULT_SOCKET_NAME, Event, Response, send_client_command};
use serde::{Deserialize, Serialize};

use crate::music_lib::CACHE_DIR;

/// Defines the socket type to use when starting the daemon.
pub type SocketType = mpipc::SocketType;

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Error {
    /// Could not obtain the daemon socket address.
    SocketError,
    /// The socket address is already in use.
    AddrInUse,
    /// Emitted when the daemon event channel is already initialized when starting the daemon.
    ///
    /// This likely means a second daemon was started in the same context.
    EventChannelInitialized,
    NoMusicLibrary,
    MusicLibraryNotAccessible,
    CacheDirError(String),
    DataDirError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SocketError => write!(f, "Socket cration/connection failed"),
            Self::AddrInUse => write!(f, "The socket address is already in use"),
            Self::EventChannelInitialized => {
                write!(f, "The event channel for the daemon is already initialized")
            }
            Self::NoMusicLibrary => {
                write!(f, "The provided music library directory does not exist")
            }
            Self::MusicLibraryNotAccessible => write!(
                f,
                "Could not access the music library due to a permission issue"
            ),
            Self::CacheDirError(e) => write!(f, "Could not initialize the cache directory: {e}"),
            Self::DataDirError(e) => write!(f, "Could not initialize the data directory: {e}"),
        }
    }
}

/// Defines under which conditions should the daemon claim an occupied socket address.
///
/// This is only effective if filesystem sockets are used.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddrClaimMode {
    /// Do not claim the socket address under any circumstances.
    DoNotClaim,
    /// Only claim the socket address if there is no response to ping commands from the process
    /// that listens on the socket.
    #[default]
    ClaimIfUnresponsive,
    /// Force claim the socket address.
    ForceClaim,
}

/// Error that can be thrown by the constructors of [Config].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigError {
    /// The provided bus name suffix for MPRIS was invalid.
    InvalidBusNameSuffix,
    /// Could not get the home directory path.
    HomeError,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBusNameSuffix => write!(f, "The bus name suffix provided was invalid"),
            Self::HomeError => write!(f, "Could not get the home directory path"),
        }
    }
}

/// Configuration options for the daemon.
///
/// Used with the [`start`] function.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    /// The directory containing daemon cache.
    pub cache_dir: PathBuf,
    /// The directory containing the program data, eg. the playlist file.
    pub data_dir: PathBuf,
    /// The directory containing the music library/audio files to load.
    pub music_dir: PathBuf,
    /// The name of the socket to listen on.
    ///
    /// This will either resolve to a namespaced socket with the same name, or, in case of a
    /// filesystem socket, a file with the same name in the temporary directory.
    pub socket_name: String,
    /// Defines under which conditions should the daemon claim an occupied socket address.
    pub addr_claim_mode: AddrClaimMode,
    /// Defines the type of socket the daemon should use.
    pub socket_type: SocketType,
    /// Configuration options specific to the [playback] module.
    pub playback_config: playback::Config,
}

impl Config {
    /// Get the default daemon config.
    ///
    /// For a custom music player, you probably want to create your own config from
    /// scratch, or use the [Config::try_from_name] constructor.
    ///
    /// # Examples
    /// ```
    /// # use daemon;
    /// let conf = daemon::Config::try_default().unwrap();
    /// ```
    pub fn try_default() -> Result<Self, ConfigError> {
        Self::try_from_name("music-player", DEFAULT_SOCKET_NAME)
    }

    /// Create a new config from program and socket names.
    ///
    /// The generated paths will contain the name of the program, eg. `~/.cache/<PROGRAM_NAME>`,
    /// `~/.local/share/<PROGRAM_NAME>`.
    ///
    /// Fails if the path to the home directory cannot be obtained.
    ///
    /// # Examples
    /// ```
    /// # use daemon;
    /// let conf = daemon::Config::try_from_name("my-player", "MY_PLAYER");
    /// ```
    pub fn try_from_name(name: &str, socket_name: &str) -> Result<Self, ConfigError> {
        let home_dir = match home_dir() {
            Some(home) => home,
            None => {
                return Err(ConfigError::HomeError);
            }
        };

        let mut cache_dir = home_dir.clone();
        cache_dir.push(".cache");
        cache_dir.push(name);

        let mut data_dir = home_dir.clone();
        data_dir.push(".local/share");
        data_dir.push(name);

        let mut music_dir = home_dir.clone();
        music_dir.push("Music");

        #[cfg(feature = "mpris")]
        let bus_name_suffix = String::from("com.dev.") + name;
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

fn handle_error(conn: std::io::Result<Stream>) -> Option<Stream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            warn!("Incoming connection failed: {e}");
            None
        }
    }
}

// TODO: I should also create a `stop` function for stopping the daemon without connecting to it
/// Create an IPC socket listener for the daemon with the specified address.
///
/// Depending on the configuration, this can either return a namespaced socket or a filesystem one.
///
/// A filesystem socket will be returned if namespaced sockets are not supported, and the
/// [SocketType] passed is [SocketType::NamespacedOrFilesystem], or if the [SocketType] value
/// passed is [SocketType::FilesystemOnly].
///
/// The [`AddrClaimMode`] value defines under which conditions should an occupied socket address be
/// claimed. This is only effective for filesystem sockets.
fn get_listener(
    socket_name: &str,
    socket_type: &SocketType,
    claim_mode: &AddrClaimMode,
) -> Result<Listener, Error> {
    let socket = match mpipc::get_socket(socket_name, socket_type) {
        Ok(sock) => sock,
        Err(e) => {
            error!("Could not obtain the socket: {e}");
            return Err(Error::SocketError);
        }
    };

    let opts = ListenerOptions::new().name(socket.clone());
    if socket.is_namespaced() {
        trace!(
            "Creating a listener on \"{}\" (namespaced socket)",
            socket_name
        );
    } else {
        trace!(
            "Creating a listener on \"{}\" (filesystem socket)",
            socket_name
        );
    }

    match opts.create_sync() {
        Ok(listener) => Ok(listener),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse
                && socket.is_path()
                && !socket.is_namespaced()
            {
                warn!("The socket address is already in use");

                match claim_mode {
                    AddrClaimMode::DoNotClaim => {
                        info!(
                            "The daemon is configured not to reclaim the socket address, aborting"
                        );
                        Err(Error::AddrInUse)
                    }
                    AddrClaimMode::ClaimIfUnresponsive => {
                        info!("Attempting to claim the socket address");
                        match send_client_command(Command::Ping, socket_name, socket_type) {
                            Ok(response) => {
                                if response == Response::Pong {
                                    error!(
                                        "The other deamon responded to the pong command, aborting"
                                    );
                                    return Err(Error::AddrInUse);
                                } else {
                                    error!(
                                        "Got an unexpected response from the deamon: {response:?}"
                                    );
                                    return Err(Error::AddrInUse);
                                }
                            }
                            Err(e) => {
                                if e == mpipc::Error::ConnectionError {
                                    info!(
                                        "The other daemon is either dead or unresponsive, claiming the address"
                                    );
                                } else {
                                    error!(
                                        "Got an unexpected error while sending a ping command: {e}"
                                    );
                                    return Err(Error::AddrInUse);
                                }
                            }
                        }

                        if let Err(e) = remove_file(mpipc::get_fs_socket_path(socket_name)) {
                            error!("Could not remove the old socket: {e}");
                            return Err(Error::SocketError);
                        }
                        let opts = ListenerOptions::new().name(socket.clone());
                        match opts.create_sync() {
                            Ok(listener) => {
                                info!("Succesfully claimed the address");
                                Ok(listener)
                            }
                            Err(e) => {
                                error!("Could not create a listener: {e}");
                                Err(Error::SocketError)
                            }
                        }
                    }
                    AddrClaimMode::ForceClaim => {
                        info!("Force claiming the socket address");
                        if let Err(e) = remove_file(mpipc::get_fs_socket_path(socket_name)) {
                            error!("Could not remove the old socket: {e}");
                            return Err(Error::SocketError);
                        }
                        let opts = ListenerOptions::new().name(socket.clone());
                        match opts.create_sync() {
                            Ok(listener) => {
                                info!("Succesfully claimed the address");
                                Ok(listener)
                            }
                            Err(e) => {
                                error!(
                                    "Could not create a listener despite claiming the socket: {e}"
                                );
                                Err(Error::SocketError)
                            }
                        }
                    }
                }
            } else {
                error!("Failed creating a listener for the daemon socket: {e}");
                Err(Error::SocketError)
            }
        }
    }
}

/// Send an event to the daemon thread.
///
/// Will always return an error when the daemon isn't initialized, for example during testing.
pub(crate) fn send_event(event: Event) -> Result<(), String> {
    match EVENT_SENDER.read().as_mut() {
        Ok(guard) => match guard.clone() {
            Some(guard) => match guard.send(event) {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("Could not send the event to the daemon: {e}");
                    Err(e.to_string())
                }
            },
            None => Err(String::from(
                "Could not obtain the event channel, this is expected during testing",
            )),
        },
        Err(e) => Err(e.to_string()),
    }
}

/// Start the daemon with the given config.
///
/// The `daemon` usually starts listening for commands around 100ms after this function is
/// called on a low-end system, but some commands sent too early might fail if the music library
/// isn't loaded yet, the playback module is not initialized, or if the MPRIS server hasn't started
/// yet (if the MPRIS feature is enabled).
///
/// The initialization of the music library is by far the most time-consuming process ran when the
/// `daemon` starts, and the time it takes vastly depends on the read speeds of the hard drive of
/// the host machine and the size of the music library.
///
/// **Note:** This function will block. Launch it in a separate [thread](std::thread) if you want
/// to run the daemon in the background.
///
/// # Examples
/// ```no_run
/// # use daemon;
/// let config = daemon::Config::try_default().unwrap();
/// daemon::start(config).unwrap();
/// ```
pub fn start(config: Config) -> Result<(), Error> {
    debug!("Starting daemon on \"{}\"", config.socket_name);

    if let Err(e) = music_lib::set_dirs(config.clone()) {
        error!("Could not set the paths: {e}");
        return Err(e);
    }

    if config.socket_name == mpipc::DEFAULT_SOCKET_NAME {
        warn!(
            "Using the default IPC socket name. Please use a unique name outside of just testing"
        );
    }

    let listener = get_listener(
        &config.socket_name,
        &config.socket_type,
        &config.addr_claim_mode,
    )?;

    thread::spawn(move || {
        let _ = music_lib::load(true);
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

    let mut guard = EVENT_SENDER.write().unwrap();
    // I could check if the sender is still connected here
    if guard.as_ref().is_some() {
        error!("Could not start the daemon because the event channel is already initialized");
        info!("(Did you try to start a second daemon in the same context?)");
        return Err(Error::EventChannelInitialized);
    }
    let (event_sender, event_receiver) = mpsc::channel();
    *guard = Some(event_sender);
    drop(guard);

    loop {
        // Launching a different daemon overwrites the [EVENT_SENDER] variable, causing the
        // previous sender to be dropped and the `recv()` method to return an [Err] type.
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
            Event::Shutdown => {
                trace!("Received shutdown event");
                std::process::exit(0);
            }
            Event::ConnectionClosed => {
                trace!("Connection with a client closed")
            }
            _ => {}
        }
    }
}
