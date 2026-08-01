## Use of generative AI
See the [AI Usage Policy](https://github.com/tpaau/chilen/blob/main/AI_POLICY.md).


## Modules

Project modules are managed in a
[Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).

The project has three main workspace members:
- The root is a binary crate with that is a shell built around the Chilen backend
    - [`chilen_backend`](https://tpaau.github.io/chilen/chilen_backend/) - Library crate that
        manages the music library, audio playback, etc. Formerly `chilen_daemon`.
    - [`iced_m3`](https://tpaau.github.io/chilen/iced_m3/) - Material Design 3 widget library for
        [iced](https://iced.rs/)


## Architecture

> [!NOTE]
> This file tree is not regularly updated and may be out of date.

```
src/
├── argparse.rs                # Command-line Argument parsing
├── gui                        # Graphical user interface made with `iced`
│   ├── font.rs                # Font related stuff
│   ├── icons.rs               # Material symbols stuff
│   ├── mod.rs
│   ├── playlist_view.rs
│   ├── tests.rs
│   ├── styles                 # Styling functions and utilities for widgets
│   │   ├── button.rs
│   │   ├── mod.rs
│   │   └── scrollable.rs
│   └── widgets                # Reusable widgets not to be included in `iced_m3`
│       ├── mod.rs
│       └── playlist_button.rs
├── main.rs
└── settings.rs                # User-configurable app settings

chilen_backend/src/
├── lib.rs                     # Backend initialization
├── music_lib                  # Music library management
│   ├── covers.rs              # Cover art management
│   ├── indexer.rs             # Library indexing
│   ├── mod.rs
│   ├── state.rs               # State management (eg. playlists)
│   └── tests.rs
├── playback                   # Audio playback
│   ├── mod.rs
│   ├── mpris.rs               # MPRIS integration
│   ├── state.rs               # State management (eg. queue, shuffle state)
│   └── tests.rs
└── tests.rs

iced_m3/src/
├── lib.rs
├── theme                      # Theme and palette stuff
│   ├── mod.rs
│   └── tests.rs
└── widget                     # Caterial widgets
    ├── drop_down_menu.rs      # Drop-down menu widget
    └── mod.rs
```

### Building
You will need to have Rust *nightly* installed on your system to compile this program.

On Linux, you will also need the development files for `alsa-lib`. They
usually can be installed as `alsa-lib-devel`:

For Fedora Silverblue, run this:
```bash
rpm-ostree install alsa-lib-devel
```

Consider installing [`just`](https://github.com/casey/just) for running some of
the development commands. This is entirely optional, though.

### Running
Start the app:
```bash
cargo run
```

You might also want to pass the `-v trace` flag for more debug info:
```bash
cargo run -- -v trace
```

> [!TIP]
> You can override the music, cache, and data directories from the CLI.

## Checks

> [!TIP]
> You can run the following checks all at once by running `just check`.

### Code style
Follow the standard Rust formatting. Check with:
```bash
cargo fmt --check --all
```

To auto-format, run:
```bash
cargo fmt --all
```

### Tests
To test the code, run:
```bash
cargo test --workspace
```

### Deny
Install `cargo-deny`:
```bash
cargo install --locked cargo-deny
```

Check dependencies:
```bash
cargo deny check
```
