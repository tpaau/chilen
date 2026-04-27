# Music Player Daemon library

<small>Not to be confused with [MPD](https://www.musicpd.org/), which is an unrelated project.</small>

This library contains data types and functions used to start the music player daemon. It does not
include the utilities used to communicate with it. For that you will have to use `mpipc`.

The daemon listens over a namespaced socket by default, unless the host system does not support
namespaced sockets, or if the daemon was configured to use a filesystem socket.

# Examples

Starting the daemon with default config:
```no_run
# use daemon;
let config = daemon::Config::try_default().unwrap();
daemon::start(config).unwrap();
```

Starting the daemon with a custom config:
```no_run
# use std::{thread, time::Duration};
# use daemon::{AddrClaimMode, SocketType, playback};
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

let config = daemon::Config {
    cache_dir,
    data_dir,
    music_dir,
    socket_name: "TEST_PLAYER.socket".to_string(),
    addr_claim_mode: AddrClaimMode::default(),
    socket_type: SocketType::default(),
    playback_config: playback::Config {
        #[cfg(feature = "mpris")]
        identity: "Test Player".to_string(), // Human-readable name for the player
        #[cfg(feature = "mpris")]
        bus_name_suffix: "com.dev.test-player".to_string(),
        allow_rate_modification: true,
    },
};

// The daemon usually takes around 100ms to start listening
daemon::start(config).unwrap();
```
