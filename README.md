# music-player

Prototype local music player, very much a WIP right now.


## Table of contents
- [Features](#features)
- [Project structure](#project-structure)
- [Building from source](#building-from-source)
- [Usage](#usage)
    - [Running the daemon](#usage_daemon)
    - [Managing playlists](#usage_playback)
    - [Controlling playback](#usage_playback)
- [FAQ](#faq)
    - [Is this a rewrite/clone of [MPD](https://www.musicpd.org/)?](#faq_mpd)
- [Inspiration](#inspiration)


<a name="features"></a>
## Features

- Blazingly fast and memory safe 🚀🦀
- Modular design ideal for creating alternate frontents

<a name="project-structure"></a>
## Project structure

The project has three main workspace members:
- The root is a binary package with a CLI and will soon also have a GUI
    - `daemon` is the core of the music player, responsible for managing cache, the music library, and playback
    - `mpipc` is a library that defines common types and functions used to interact with the daemon

Those modules are managed in a
[Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).

```
├── daemon // The daemon library
│   └── src
│       ├── daemon_thread.rs   // Client connection handling
│       ├── data
│       │   ├── cache          // Covers and cache management
│       │   │   ├── covers.rs  // Cover caching
│       │   │   ├── indexer.rs // Music library indexing
│       │   │   └── mod.rs
│       │   ├── mod.rs
│       │   └── music_lib.rs   // Music library state managemnt (playlists, etc.)
│       ├── lib.rs
│       ├── playback.rs        // Audio playback and Mpris integrations
│       └── tests.rs
├── mpipc // Library used to interact with the daemon
│   └── src
│       └── lib.rs
└── src
    ├── argparse.rs // Command-line argument parsing
    ├── cli.rs      // Command-line argument execution
    ├── gui         // TBD
    │   └── mod.rs
    ├── main.rs
    └── tests.rs
```

<a name="building-from-source"></a>
## Building from source
You will need to have Rust properly configured on your system to compile this program.

For compiling the player from source on Linux, you will also need to have the development files for
`alsa-libs` installed.

#### Fedora Silverblue
```bash
rpm-ostree install alsa-libs-devel
```

After you have all your dependencies installed, run the command below to compile the program:
```bash
cargo build --release
```

Then, you can run the player with `cargo`:
```bash
cargo run --release -- <PLAYER_ARGUMENTS>
```

You can find the compiled binary in `target/release/music-player`.


<a name="usage"></a>
## Usage

Currently, the player only comes with a command-line interface. This is only to test if the deamon
works properly and will obviously not be the primary way of interacting with the player once it's
finished.

> [!NOTE]
> Not all commands are listed here.

> [!TIP]
> You can pass the `-v|--verbosity` option to set the log level filter:
> ```bash
> cargo run --release -- -v trace <PLAYER_COMMAND>
> ```

<a name="usage_daemon"></a>
#### Running the daemon
**Starting the daemon**
```bash
cargo run --release -- daemon start
```

**Stopping the daemon with a client command**
```bash
cargo run --release -- daemon stop
```

<a name="usage_playlists"></a>
#### Managing playlists
Playlists can be managed with the `playlist` command.

**Creating a new empty playlist**
```bash
cargo run --release -- playlist new playlist-name ~/Music/track.mp3
```

**Deleting an existing playlist**
```bash
cargo run --release -- playlist delete playlist-name
```

**Listing existing playlists**
```bash
cargo run --release -- playlist list
```


<a name="usage_playback"></a>
#### Controlling playback

**Adding tracks to the queue by paths**
```bash
cargo run --release -- playback set-queue --tracks ~/Music/track.mp3
```

Tracks can also be appended to the queue:
```bash
cargo run --release -- playback append-to-queue ~/Music/track.mp3
```

You can also set the queue to be a playlist:
```bash
cargo run --release -- playback set-queue --playlist playlist-name
```

**Pausing**
```bash
cargo run --release -- playback pause
```

**Playing**
```bash
cargo run --release -- playback play
```

**Playing a track at a specific index**

Tracks have their unique indices in the queue. You can jump to a track at a specific index.

```bash
cargo run --release -- playback play --index 6
```

**Skipping to the next track**
```bash
cargo run --release -- playback next
```

**Skipping to the previous track**
```bash
cargo run --release -- playback previous
```


<a name="faq"></a>
## FAQ

<a name="faq_mpd"></a>
#### Is this a rewrite/clone of [MPD](https://www.musicpd.org/)?
No. I have never seen even one line of MPD code.

I tried to install MPD once but I ran into a dependency hell :P


<a name="inspiration"></a>
## Inspiration
- [Auxio](https://github.com/OxygenCobalt/Auxio): A music player that just works
