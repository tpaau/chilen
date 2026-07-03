#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![feature(doc_cfg)]

mod daemon_thread;
pub mod handler;
mod music_lib;
pub mod playback;
#[cfg(test)]
mod tests;

use std::{
    env::{home_dir, temp_dir},
    fs::remove_file,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock, mpsc},
    thread::{self, JoinHandle},
};

use interprocess::local_socket::{Listener, ListenerOptions, Stream, traits::ListenerExt};
use log::{debug, error, info, trace, warn};

use chilen_ipc::{Command, DEFAULT_SOCKET_NAME, Event, Response};
use serde::{Deserialize, Serialize};

use crate::{
    daemon_thread::ThreadCommand,
    handler::{Request, send_request},
    music_lib::{
        CACHE_DIR,
        covers::LoadMode,
        state::{self, get_library},
    },
};

/// Defines the socket type to use when starting the daemon.
pub type SocketType = chilen_ipc::SocketType;

static EVENT_SENDERS: LazyLock<Arc<RwLock<Vec<mpsc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Vec::new())));

/// Damon thread to main daemon process communication.
static COMMAND_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<ThreadCommand>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

/// This property will always be set during daemon runtime, it is mostly safe to unwrap it in
/// functions launched by the daemon.
pub(crate) static CONFIG: LazyLock<Arc<RwLock<Option<Config>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Error {
    /// Socket creation failed.
    SocketError,
    /// The socket address is already in use.
    AddrInUse,
    /// Emitted when the daemon event channel is already initialized when starting the daemon.
    ///
    /// This likely means a second daemon was started in the same context.
    EventChannelInitialized,
    /// The event channel is not initialized, which likely means that the daemon isn't running.
    DaemonNotRunning,
    /// The provided music library path is not a directory or doesn't exist.
    NoLibrary,
    /// Cannot access the music library due to a permission issue.
    LibraryNotAccessible,
    /// The cache directory could not be initialized.
    ///
    /// This is usually a result of a permission issue.
    CacheDirError(String),
    /// The data directory could not be initialized.
    ///
    /// This is usually a result of a permission issue.
    DataDirError(String),
    /// Quit requests from external clients are not allowed.
    QuitDisabled,
    /// Raise requests are not allowed.
    RaiseDisabled,
    /// Toggling fullscreen mode by external clients is not allowed.
    SetFullscreenDisabled,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SocketError => write!(f, "Socket creation failed"),
            Self::AddrInUse => write!(f, "The socket address is already in use"),
            Self::EventChannelInitialized => {
                write!(f, "The event channel is already initialized")
            }
            Self::DaemonNotRunning => write!(f, "The daemon doesn't seem to be running"),
            Self::NoLibrary => {
                write!(
                    f,
                    "The provided music library path is not a directory or doesn't exist"
                )
            }
            Self::LibraryNotAccessible => write!(
                f,
                "Could not access the music library due to a permission issue"
            ),
            Self::CacheDirError(e) => write!(f, "Could not initialize the cache directory: {e}"),
            Self::DataDirError(e) => write!(f, "Could not initialize the data directory: {e}"),
            Self::QuitDisabled => write!(f, "Quit requests from external clients are not allowed"),
            Self::RaiseDisabled => write!(f, "Raise requests are not allowed"),
            Self::SetFullscreenDisabled => write!(
                f,
                "Toggling fullscreen mode by external clients is not allowed"
            ),
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

// TODO: Add a config option for reshuffling tracks on playlist repeat or something, will figure it out
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
    ///
    /// Players that have no ability to raise (eg. players CLI or TUI interfaces) should set this to
    /// false.
    pub can_raise: bool,
    // Whether clients can request the daemon to display the user interface in fullscreen mode.
    pub can_set_fullscreen: bool,
    /// Whether clients can request the daemon to quit.
    ///
    /// This only affects clients that connect to the daemon over a local socket, it does not
    /// affect the [`quit`] function.
    pub can_quit: bool,
    /// The basename of an installed .desktop file which complies with the Desktop entry
    /// specification, with the ".desktop" extension stripped.
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
            can_set_fullscreen: false,
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
    handler::cleanup();
    *COMMAND_SENDER.write().unwrap() = None;
    *EVENT_SENDERS.write().unwrap() = Vec::new();
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

/// Sends a command from a daemon thread to the main daemon process.
pub(crate) fn send_command(command: ThreadCommand) -> Result<(), String> {
    let conf_guard = crate::CONFIG.read().unwrap();
    let config = match conf_guard.as_ref() {
        Some(conf) => conf,
        None => return Err(Error::DaemonNotRunning.to_string()),
    };
    let sender_guard = COMMAND_SENDER.read().unwrap();
    let sender = match sender_guard.as_ref() {
        Some(sender) => sender,
        None => return Err(Error::DaemonNotRunning.to_string()),
    };
    if command == ThreadCommand::Quit {
        if !config.can_quit {
            return Err(Error::QuitDisabled.to_string());
        }
    } else if command == ThreadCommand::Raise && !config.can_raise {
        return Err(Error::RaiseDisabled.to_string());
    }
    if let Err(e) = sender.send(command) {
        error!("Could not send the thread command to the daemon: {e}");
        return Err(e.to_string());
    }
    Ok(())
}

// TEST: Has been a cause of many crashes
/// Send an event to the daemon thread.
pub(crate) fn send_event(event: Event) {
    let mut senders = EVENT_SENDERS.write().unwrap();
    let mut dead = Vec::new();
    for (i, sender) in senders.iter().enumerate() {
        if sender.send(event.clone()).is_err() {
            info!("Removing dead event sender at {i}");
            dead.push(i);
        }
    }
    for (i, ded) in dead.iter().enumerate() {
        senders.swap_remove(ded - i);
    }
}

/// Subscribe to important state changes.
pub(crate) fn subscribe_to_events() -> Result<mpsc::Receiver<Event>, chilen_ipc::Error> {
    let mut events = Vec::new();
    match get_library() {
        Ok(lib) => {
            events.push(Event::LibraryChanged(lib.into()));
        }
        Err(e) => {
            error!("Could not get the contents of the music library: {e}");
            return Err(e);
        }
    }
    match playback::get_initial_events() {
        Ok(playback_events) => {
            for event in playback_events {
                events.push(event);
            }
        }
        Err(e) => {
            error!("Could not get the initial events from the playback module: {e}");
            return Err(e);
        }
    };
    let guard = crate::CONFIG.read().unwrap();
    let conf = guard.as_ref().unwrap();
    events.push(Event::CanRaiseChanged(conf.can_raise));
    events.push(Event::CanQuitChanged(conf.can_quit));
    let (sender, receiver) = mpsc::channel();
    for event in events {
        sender.send(event).unwrap();
    }
    let mut guard = EVENT_SENDERS.write().unwrap();
    let senders: &mut Vec<mpsc::Sender<Event>> = guard.as_mut();
    senders.push(sender);
    Ok(receiver)
}

pub(crate) fn raise() -> Result<(), Error> {
    let guard = crate::CONFIG.read().unwrap();
    if guard.as_ref().unwrap().can_raise {
        match crate::send_command(ThreadCommand::Raise) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::RaiseDisabled),
        }
    } else {
        Err(Error::RaiseDisabled)
    }
}

pub(crate) fn set_fullscreen(fullscreen: bool) -> Result<(), Error> {
    let guard = crate::CONFIG.read().unwrap();
    if guard.as_ref().unwrap().can_set_fullscreen {
        match crate::send_command(ThreadCommand::SetFullscreen(fullscreen)) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::SetFullscreenDisabled),
        }
    } else {
        Err(Error::SetFullscreenDisabled)
    }
}

/// Set whether clients can send raise requests to the daemon.
///
/// Will fail with [`Error::DaemonNotRunning`] if the daemon isn't running.
pub fn set_can_raise(can_raise: bool) -> Result<(), Error> {
    let mut conf_guard = CONFIG.write().unwrap();
    let conf = match conf_guard.as_mut() {
        Some(conf) => conf,
        None => return Err(Error::DaemonNotRunning),
    };
    if conf.can_raise != can_raise {
        conf.can_raise = can_raise;
        send_event(Event::CanRaiseChanged(conf.can_raise));
    }
    Ok(())
}

pub fn set_can_set_fullscreen(can_set_fullscreen: bool) -> Result<(), Error> {
    let mut conf_guard = CONFIG.write().unwrap();
    let conf = match conf_guard.as_mut() {
        Some(conf) => conf,
        None => return Err(Error::DaemonNotRunning),
    };
    if conf.can_set_fullscreen != can_set_fullscreen {
        conf.can_set_fullscreen = can_set_fullscreen;
        send_event(Event::CanGoFullscreenChanged(conf.can_raise));
    }
    Ok(())
}

/// Set whether the daemon should accept quit requests from clients.
///
/// This does not affect the [`quit`] function.
///
/// Will fail with [`Error::DaemonNotRunning`] if the daemon isn't running.
pub fn set_can_quit(can_quit: bool) -> Result<(), Error> {
    let mut conf_guard = CONFIG.write().unwrap();
    let conf = match conf_guard.as_mut() {
        Some(conf) => conf,
        None => return Err(Error::DaemonNotRunning),
    };
    if conf.can_quit != can_quit {
        conf.can_quit = can_quit;
        send_event(Event::CanQuitChanged(conf.can_quit));
    }
    Ok(())
}

/// Stop a running daemon instance.
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

/// Same as [`stop`] with an additional checks to ensure an external client cannot stop the daemon
/// if the configuration disallows it.
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

/// Start the daemon with the given config.
///
/// The [`Receiver`](mpsc::Receiver) returned by this functions can be used to listen for requests
/// that are not broadcast to all clients connected via the IPC socket. This is to ensure requests
/// such as [`Request::Raise`] do not cause conflicts when there are multiple clients attempting to
/// handle them all at once.
///
/// The daemon usually starts listening for commands around 100ms after this function is
/// called, but some commands sent too early might fail if the music library isn't loaded yet, the
/// playback module is not initialized, or if the MPRIS server isn't running (if the MPRIS feature
/// is enabled).
///
/// The initialization of the music library is by far the most time-consuming process during
/// startup, and the time it takes vastly depends on the read speeds of the hard drive on the host
/// machine and the size of the music library.
///
/// **Note:** This function launches the daemon in a separate thread, it doesn't block.
///
/// # Examples
/// ```no_run
/// # use chilen_daemon;
/// let config = chilen_daemon::Config::try_default().unwrap();
/// let (_, handle) = chilen_daemon::start(config);
/// match handle.join().unwrap() {
///     Ok(_) => println!("Daemon exited"),
///     Err(e) => {
///         panic!("Daemon failed: {e}");
///     }
/// }
/// ```
pub fn start(config: Config) -> (mpsc::Receiver<Request>, JoinHandle<Result<(), Error>>) {
    let receiver = handler::init();
    (receiver, thread::spawn(|| start_blocking(config)))
}

// TEST: Add tests to make sure daemon can start and stop properly
fn start_blocking(config: Config) -> Result<(), Error> {
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

    let mut sender_guard = COMMAND_SENDER.write().unwrap();
    if sender_guard.as_ref().is_some() {
        error!("The command channel is already initialized!");
        return Err(Error::EventChannelInitialized);
    }
    let (sender, receiver) = mpsc::channel();
    *sender_guard = Some(sender);
    drop(sender_guard);

    loop {
        let cmd = receiver.recv().unwrap();
        match cmd {
            ThreadCommand::Raise => {
                trace!("Received a raise request");
                handler::send_request(Request::Raise);
            }
            ThreadCommand::SetFullscreen(fullscreen) => {
                trace!("Received request to set fullscreen to: {fullscreen}");
                handler::send_request(Request::SetFullscreen { fullscreen });
            }
            ThreadCommand::Quit => {
                trace!("Received quit command");
                send_event(Event::Quit);
                cleanup();
                info!("Stopped.");
                return Ok(());
            }
        }
    }
}
