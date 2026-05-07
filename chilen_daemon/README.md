# Chilen Daemon library

> ### Warning
> The project is under active development, so please be wary of any bugs you may find, and expect
> breaking API changes in the upcoming updates!

This library contains data types and functions used to start and control the Chilen daemon.

Unlike [MPD](https://www.musicpd.org/), Chilen daemon can be bundled with your program's binary
package to ensure a seamless installation process for your program and a cohesive experience for the
end user.

The daemon uses a local socket to connect to its clients. The type of socket used depends on your
platform, and the startup configuration of the daemon. Under the hood, the [`interprocess`] and
[`rmp_serde`] crates are used for handling clients connections and [`chilen_ipc`] for providing
common data types.

This crate does not provide direct access to the daemon over the local socket. For such
functionality, please refer to the `chilen_ipc` crate.

Certain interfaces with the daemon are *only* available through the use of this library, and are
not available to regular clients connecting over the IPC socket. The program controlling the daemon
can, for instance:
- Stop the daemon without having to connect to it over the local socket
- Modify some of the properties set at launch

# Examples

Starting the daemon with default config:
```no_run
# use chilen_daemon;
let config = chilen_daemon::Config::try_default().unwrap();
chilen_daemon::start(config).unwrap();
```

Starting the daemon with a custom config:
```no_run
# use std::{thread, time::Duration};
# use chilen_daemon::{AddrClaimMode, SocketType, playback};
let home = std::env::home_dir().unwrap();
let name = "test-player";

let mut cache_dir = home.clone();
cache_dir.push("cache");
cache_dir.push(name);

let mut data_dir = home.clone();
data_dir.push(".local/share");
data_dir.push(name);

let mut music_dir = home.clone();
music_dir.push("Music");

let config = chilen_daemon::Config {
    cache_dir,
    data_dir,
    music_dir,
    socket_name: "TEST_PLAYER.socket".to_string(),
    addr_claim_mode: AddrClaimMode::default(),
    socket_type: SocketType::default(),
    can_raise: false,
    playback_config: playback::Config {
        #[cfg(feature = "mpris")]
        identity: "Test Player".to_string(), // Human-readable name for the player
        #[cfg(feature = "mpris")]
        bus_name_suffix: "com.dev.test-player".to_string(),
        allow_rate_modification: true,
    },
};

// The daemon usually takes around 100ms to start listening
chilen_daemon::start(config).unwrap();
```
