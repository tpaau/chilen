# Chilen

Prototype local music player, very much a WIP right now.

Documentation on Chilen submodules:
- [`chilen_backend`](https://tpaau.github.io/chilen/chilen_backend/) - The thing that manages the music library and audio playback, can be reused with different frontends
- [`iced_m3`](https://tpaau.github.io/chilen/iced_m3/) - Material Design 3 widget library for [iced](https://iced.rs/)

Other projects related to Chilen:
- [`lrc_rs`](https://github.com/tpaau/lrc_rs) - Robust crate for working with synced lyrics content in the LRC format with support for the A2 extension
- [`m3u8_rs`](https://github.com/tpaau/m3u8_rs) - Rust crate for M3U8 playlist files

## Table of contents
- [Features](#features)
- [Building from source](#building-from-source)
- [Inspiration](#inspiration)
- [Contributing](#contributing)


<a name="features"></a>
## Features
- Fully offline and blazingly fast 🚀🦀
- Proper desktop integration
- Support for a wide range of audio codecs
- Modular design


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

<a name="inspiration"></a>
## Inspiration
- [Auxio](https://github.com/OxygenCobalt/Auxio): A music player that just works

<a name="contributing"></a>
## Contributing
See [CONTRIBUTING](https://github.com/tpaau/chilen/blob/main/CONTRIBUTING.md).
