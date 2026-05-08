#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![feature(doc_cfg)]
#![feature(mpmc_channel)]

mod daemon_thread;
mod music_lib;
pub mod playback;
#[cfg(test)]
mod tests;

use std::{
    env::{home_dir, temp_dir},
    fs::remove_file,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock, mpmc, mpsc},
    thread,
};

use interprocess::local_socket::{Listener, ListenerOptions, Stream, traits::ListenerExt};
use log::{debug, error, info, trace, warn};

use chilen_ipc::{Command, DEFAULT_SOCKET_NAME, Event, Response};
use serde::{Deserialize, Serialize};

use crate::{
    daemon_thread::ThreadCommand,
    music_lib::{CACHE_DIR, covers::LoadMode, state},
};

/// Defines the socket type to use when starting the daemon.
pub type SocketType = chilen_ipc::SocketType;

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpmc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static EVENT_RECEIVER: LazyLock<Arc<RwLock<Option<mpmc::Receiver<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static COMMAND_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<ThreadCommand>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

/// This property will always be set during daemon runtime, it is mostly safe to unwrap it in
/// functions launched by the daemon.
pub(crate) static CONFIG: LazyLock<Arc<RwLock<Option<Config>>>> =
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
    /// The event channel is not initialized, which likely means that there is no daemon running.
    DaemonNotRunning,
    NoLibrary,
    LibraryNotAccessible,
    CacheDirError(String),
    DataDirError(String),
    ConfigNotInitialized,
    /// Quit requests from external clients are not allowed
    QuitDisabled,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SocketError => write!(f, "Socket creation/connection failed"),
            Self::AddrInUse => write!(f, "The socket address is already in use"),
            Self::EventChannelInitialized => {
                write!(f, "The event channel for the daemon is already initialized")
            }
            Self::DaemonNotRunning => write!(f, "The daemon doesn't seem to be running"),
            Self::NoLibrary => {
                write!(f, "The provided music library directory does not exist")
            }
            Self::LibraryNotAccessible => write!(
                f,
                "Could not access the music library due to a permission issue"
            ),
            Self::CacheDirError(e) => write!(f, "Could not initialize the cache directory: {e}"),
            Self::DataDirError(e) => write!(f, "Could not initialize the data directory: {e}"),
            Self::ConfigNotInitialized => write!(f, "The daemon configuration is not set"),
            Self::QuitDisabled => write!(f, "Quit requests from external clients are not allowed"),
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

/// Error originating from the [`Config`] struct.
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
    /// Whether the player's user interface can be brought to the front using any appropriate
    /// mechanism available.
    pub can_raise: bool,
    /// Whether clients can request the daemon to quit.
    ///
    /// This only affects clients that connect to the daemon over a local socket, it does not
    /// affect the [`stop`] function.
    pub can_quit: bool,
    /// The basename of an installed .desktop file which complies with the Desktop entry
    /// specification, with the “.desktop” extension stripped.
    #[cfg(feature = "mpris")]
    pub desktop_entry: Option<String>,
    /// Configuration options specific to the [`playback`] module.
    pub playback_config: playback::Config,
}

impl Config {
    /// Get the default daemon config.
    ///
    /// For a custom music player, you probably want to create your own config from
    /// scratch, or use the [`Config::try_from_name`] constructor.
    ///
    /// # Examples
    /// ```
    /// # use chilen_daemon;
    /// let conf = chilen_daemon::Config::try_default().unwrap();
    /// ```
    pub fn try_default() -> Result<Self, ConfigError> {
        Self::try_from_name("my-player", DEFAULT_SOCKET_NAME)
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
    /// # use chilen_daemon;
    /// let conf = chilen_daemon::Config::try_from_name("my-player", "MY_PLAYER");
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
            can_raise: false,
            can_quit: true,
            #[cfg(feature = "mpris")]
            desktop_entry: None,
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

/// Clean up resources on shutdown.
fn cleanup() {
    trace!("Cleaning up...");
    music_lib::cleanup(); // The MPRIS server must go first because it unwraps `CONFIG`
    state::cleanup();
    playback::cleanup();
    playback::state::cleanup();
    *EVENT_SENDER.write().unwrap() = None;
    *EVENT_RECEIVER.write().unwrap() = None;
    *COMMAND_SENDER.write().unwrap() = None;
    *CONFIG.write().unwrap() = None;
    trace!("Done cleaning up");
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

/// Returns a filesystem path for the given socket name.
fn get_fs_socket_path(socket_name: &str) -> PathBuf {
    let mut temp_dir = temp_dir();
    temp_dir.push(socket_name);
    temp_dir
}

/// Create an IPC socket listener for the daemon with the specified address.
///
/// Depending on the configuration, this can either return a namespaced socket or a filesystem one.
///
/// A filesystem socket will be returned if namespaced sockets are not supported, and the
/// `socket_type` value passed is [`SocketType::NamespacedOrFilesystem`], or if the `socket_type`
/// value passed is [`SocketType::FilesystemOnly`].
///
/// The [`AddrClaimMode`] value defines under which conditions should an occupied socket address be
/// claimed. This is only effective for filesystem sockets.
fn get_listener(
    socket_name: &str,
    socket_type: &SocketType,
    claim_mode: &AddrClaimMode,
) -> Result<Listener, Error> {
    let socket = match chilen_ipc::get_socket(socket_name, socket_type) {
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
                        match chilen_ipc::send_command(Command::Ping, socket_name, socket_type) {
                            Ok(response) => {
                                if response == Response::Pong {
                                    error!(
                                        "The other daemon responded to the pong command, aborting"
                                    );
                                    return Err(Error::AddrInUse);
                                } else {
                                    error!(
                                        "Got an unexpected response from the daemon: {response:?}"
                                    );
                                    return Err(Error::AddrInUse);
                                }
                            }
                            Err(e) => {
                                if e == chilen_ipc::Error::ConnectionError {
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

                        if let Err(e) = remove_file(get_fs_socket_path(socket_name)) {
                            error!("Could not remove the old socket: {e}");
                            return Err(Error::SocketError);
                        }
                        let opts = ListenerOptions::new().name(socket.clone());
                        match opts.create_sync() {
                            Ok(listener) => {
                                info!("Successfully claimed the address");
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
                        if let Err(e) = remove_file(get_fs_socket_path(socket_name)) {
                            error!("Could not remove the old socket: {e}");
                            return Err(Error::SocketError);
                        }
                        let opts = ListenerOptions::new().name(socket.clone());
                        match opts.create_sync() {
                            Ok(listener) => {
                                info!("Successfully claimed the address");
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

pub(crate) fn send_command(command: ThreadCommand) -> Result<(), String> {
    if command == ThreadCommand::Quit {
        let guard = crate::CONFIG.read().unwrap();
        if !guard.as_ref().unwrap().can_quit {
            return Err(Error::QuitDisabled.to_string());
        }
    }
    match &**COMMAND_SENDER.read().as_ref().unwrap() {
        Some(sender) => match sender.send(command) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Could not send the thread command to the daemon: {e}");
                Err(e.to_string())
            }
        },
        None => Err(String::from(
            "Could ont obtain the command channel, this is expected during testing",
        )),
    }
}

/// Send an event to the daemon thread.
///
/// Will always return an error when the daemon isn't initialized, for example during testing.
pub(crate) fn send_event(event: Event) -> Result<(), String> {
    match &**EVENT_SENDER.read().as_ref().unwrap() {
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
    }
}

pub(crate) fn subscribe_to_events() -> mpmc::Receiver<Event> {
    EVENT_RECEIVER
        .read()
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone()
}

/// Set whether clients can send raise requests to the daemon.
///
/// Will fail with [`Error::ConfigNotInitialized`] if the daemon isn't running.
pub fn set_can_raise(can_raise: bool) -> Result<(), Error> {
    let mut conf_guard = CONFIG.write().unwrap();
    let conf = match conf_guard.as_mut() {
        Some(conf) => conf,
        None => return Err(Error::ConfigNotInitialized),
    };
    if conf.can_raise != can_raise {
        conf.can_raise = can_raise;
        let _ = send_event(Event::CanRaiseChanged(conf.can_raise));
    }
    Ok(())
}

/// Set whether the daemon should accept quit requests from clients.
///
/// This does not affect the [`stop`] function.
///
/// Will fail with [`Error::ConfigNotInitialized`] if the daemon isn't running.
pub fn set_can_quit(can_quit: bool) -> Result<(), Error> {
    let mut conf_guard = CONFIG.write().unwrap();
    let conf = match conf_guard.as_mut() {
        Some(conf) => conf,
        None => return Err(Error::ConfigNotInitialized),
    };
    if conf.can_quit != can_quit {
        conf.can_quit = can_quit;
        let _ = send_event(Event::CanQuitChanged(conf.can_quit));
    }
    Ok(())
}

/// Stop a running daemon instance.
///
/// This has the same effect as sending a [`Command::Shutdown`] to the daemon, but it bypasses
/// the requirement to connect to it over a local socket.
///
/// If a daemon is not running, [`Error::DaemonNotRunning`] will be returned.
pub fn quit() -> Result<(), Error> {
    if send_command(ThreadCommand::Quit).is_err() {
        error!("The daemon doesn't seem to be running");
        Err(Error::DaemonNotRunning)
    } else {
        Ok(())
    }
}

/// Same as [`stop`] with an additional check to ensure an external client cannot stop the daemon
/// if config options disallow it.
#[cfg(feature = "mpris")]
pub(crate) fn client_quit() -> Result<(), Error> {
    let guard = crate::CONFIG.read().unwrap();
    if guard.as_ref().unwrap().can_quit {
        quit().unwrap();
        Ok(())
    } else {
        Err(Error::QuitDisabled)
    }
}

// TODO: Add tests to make sure daemon can start and stop properly, and that functions that require
// the daemon to be running work if it is and fail if it's not.
/// Start the daemon with the given config.
///
/// The daemon usually starts listening for commands around 100ms after this function is
/// called on a low-end system, but some commands sent too early might fail if the music library
/// isn't loaded yet, the playback module is not initialized, or if the MPRIS server hasn't started
/// yet (if the MPRIS feature is enabled).
///
/// The initialization of the music library is by far the most time-consuming process ran when the
/// daemon starts, and the time it takes vastly depends on the read speeds of the hard drive of
/// the host machine and the size of the music library.
///
/// **Note:** This function will block. Launch it in a separate [`thread`] if you want to run the
/// daemon in the background.
///
/// # Examples
/// ```no_run
/// # use chilen_daemon;
/// let config = chilen_daemon::Config::try_default().unwrap();
/// chilen_daemon::start(config).unwrap();
/// ```
pub fn start(config: Config) -> Result<(), Error> {
    debug!("Starting daemon on \"{}\"", config.socket_name);

    *CONFIG.write().unwrap() = Some(config.clone());

    if let Err(e) = music_lib::set_dirs(config.clone()) {
        error!("Could not set the paths: {e}");
        return Err(e);
    }

    if config.socket_name == chilen_ipc::DEFAULT_SOCKET_NAME {
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
        let _ = music_lib::state::load(LoadMode::Load);
        playback::init(config.playback_config);
    });

    thread::spawn({
        move || {
            for (index, conn) in listener.incoming().filter_map(handle_error).enumerate() {
                // Size for `u64` is 8 bytes, and for `usize` it's 4 bytes on 32-bit and 8
                // bytes on 64-bit, so the conversion can be done safely if I understand this
                // correctly :)
                daemon_thread::spawn(conn, u64::try_from(index).unwrap());
            }
        }
    });

    let mut sender_guard = EVENT_SENDER.write().unwrap();
    let mut receiver_guard = EVENT_RECEIVER.write().unwrap();
    if sender_guard.as_ref().is_some() || receiver_guard.as_ref().is_some() {
        error!("The event channel is already initialized!");
        return Err(Error::EventChannelInitialized);
    }
    let (sender, receiver) = mpmc::channel();
    *sender_guard = Some(sender);
    *receiver_guard = Some(receiver);
    drop(sender_guard);
    drop(receiver_guard);

    let mut sender_guard = COMMAND_SENDER.write().unwrap();
    if sender_guard.as_ref().is_some() {
        error!("The command channel is already initialized!");
        return Err(Error::EventChannelInitialized);
    }
    let (sender, receiver) = mpsc::channel();
    *sender_guard = Some(sender);
    drop(sender_guard);

    // This loop will come in handy as new commands are added
    // loop {
    let cmd = receiver.recv().unwrap();
    match cmd {
        ThreadCommand::Quit => {
            trace!("Received quit command");
            let _ = send_event(Event::Quit);
            cleanup();
            info!("Stopped.");
            Ok(())
        }
    }
    // }
}
