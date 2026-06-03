# Chilen

Prototype local music player, very much a WIP right now.

There's currently no GUI, all interactions are either done through the provided CLI interface or via
MPRIS on Linux systems.

Chilen is split into two libraries and one binary package in an effort to allow developers for
creating custom music players, similar to [MPD](https://www.musicpd.org/). This was not the original
goal for Chilen, however, and it won't affect the usage of the app once it's finished.

Documentation on the submodules of Chilen can be found here:
- [`chilen_daemon`](https://tpaau.github.io/chilen/chilen_daemon/)
- [`chilen_ipc`](https://tpaau.github.io/chilen/chilen_ipc/)

## Table of contents
- [Features](#features)
- [Building from source](#building-from-source)
- [Usage](#usage)
    - [Running the daemon](#usage_daemon)
    - [Managing playlists](#usage_playback)
    - [Controlling playback](#usage_playback)
- [FAQ](#faq)
    - [Is this a rewrite/clone of MPD?](#faq_mpd)
- [Inspiration](#inspiration)


<a name="features"></a>
## Features
- Blazingly fast and memory safe 🚀🦀
- Modular design ideal for creating alternate frontends
- Support for a wide range of audio codecs


<a name="building-from-source"></a>
## Building from source
You will need to have Rust *nightly* installed on your system to compile this program.

On Linux, you will also need the development files for `alsa-lib`. They
usually can be installed as `alsa-lib-devel`:

For Fedora Silverblue, run this:
```bash
rpm-ostree install alsa-lib-devel
```


After you have all your dependencies installed, run the command below to compile the program:
```bash
cargo build
```

Then, you can run the player with `cargo`:
```bash
cargo run -- <PLAYER_ARGUMENTS>
```

You can find the compiled binary in `target/debug/chilen`.


<a name="usage"></a>
## Usage

> [!NOTE]
> Not all commands are listed here.
>
> Pass the `-h|--help` option to see all available commands.

Currently, the player only comes with a command-line interface. This is only to test if the daemon
works properly and will obviously not be the primary way of interacting with the player once it's
finished.

> [!TIP]
> You can pass the `-v|--verbosity` option to set the log level filter:
> ```bash
> cargo run -- -v trace <PLAYER_COMMAND>
> ```

<a name="usage_daemon"></a>
### Running the daemon
**Starting the daemon**
```bash
cargo run -- daemon start
```

**Stopping the daemon with a client command**
```bash
cargo run -- daemon stop
```

<a name="usage_playlists"></a>
### Managing playlists
Playlists can be managed with the `playlist` command.

**Creating a new empty playlist**
```bash
cargo run -- playlist new playlist-name ~/Music/track.mp3
```

**Deleting an existing playlist**
```bash
cargo run -- playlist delete playlist-name
```

**Listing existing playlists**
```bash
cargo run -- playlist list
```


<a name="usage_playback"></a>
### Controlling playback

> [!TIP]
> If you are on Linux, you can control audio playback with MPRIS.
>
> Desktop integrations for macOS and Windows are in the pipeline.

**Adding tracks to the queue by paths**
```bash
cargo run -- playback set-queue --tracks ~/Music/track.mp3
```

Tracks can also be appended to the queue:
```bash
cargo run -- playback append-to-queue ~/Music/track.mp3
```

You can also set a playlist as the queue:
```bash
cargo run -- playback set-queue --playlist playlist-name
```

**Pausing**
```bash
cargo run -- playback pause
```

**Playing**
```bash
cargo run -- playback play
```

**Playing a track at a specific index**

Tracks have their unique indices in the queue. You can jump to a track at a specific index.

```bash
cargo run -- playback play --index 6
```

**Skipping to the next track**
```bash
cargo run -- playback next
```

**Skipping to the previous track**
```bash
cargo run -- playback previous
```


<a name="faq"></a>
## FAQ

<a name="faq_mpd"></a>
### Is this a rewrite/clone of [MPD](https://www.musicpd.org/)?
No. I have never seen even one line of MPD code.

I tried to install MPD once but I ran into a dependency hell :P


<a name="inspiration"></a>
## Inspiration
- [Auxio](https://github.com/OxygenCobalt/Auxio): A music player that just works
