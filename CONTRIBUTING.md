## Building

### Prerequisites
You will need Rust nightly to build the project.

Additionally, if you are on Linux, the `alsa-lib-devel` package is required.

Consider installing [`just`](https://github.com/casey/just) for running some of
the development commands. This is entirely optional, though.

## Project structure

Project modules are managed in a
[Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).

The project has three main workspace members:
- The root is a binary package with a CLI and will soon also have a GUI
    - [`chilen_daemon`](https://tpaau.github.io/chilen/chilen_daemon/) - Managing the daemon
        - [`m3u8`](https://tpaau.github.io/chilen/m3u8/) - M3U8 playlist file parsing and serialization
    - [`chilen_ipc`](https://tpaau.github.io/chilen/chilen_ipc/) - Chilen daemon inter-process communication

> [!NOTE]
> This file tree is not regularly updated and may be out of date.

```
src/                     # Binary package root
├── argparse.rs          # Command-line argument parsing
├── cli.rs               # Command-line argument execution
├── gui                  # GUI module
│   └── mod.rs
└── main.rs

chilen_daemon/src        # Daemon library root
├── daemon_thread.rs     # Client command handling
├── lib.rs               # Main daemon thread
├── music_lib
│   ├── covers.rs        # Cover cache management
│   ├── indexer.rs       # Track indexing
│   ├── mod.rs           # Music library management
│   ├── state.rs         # Playlist management
│   └── tests.rs
├── playback
│   ├── mod.rs           # Audio playback
│   ├── mpris.rs         # MPRIS integration
│   ├── state.rs         # Playback state management
│   └── tests.rs
└── tests.rs

chilen_daemon/m3u8/src
├── lib.rs               # Common data structures and M3U8 playlist file serialization
├── parser
│   ├── mod.rs           # M3U8 playlist file parsing
│   └── tests.rs
└── tests.rs

chilen_ipc/src/          # IPC library root
├── library.rs           # Types for music library management
├── lib.rs               # Most common data types and functions
└── playback.rs          # Types for playback management
```

### Building
Clone the repository and navigate to its root:
```bash
git clone https://github.com/tpaau/chilen
cd chilen
```

Build the project:
```bash
cargo build
```

Or use the `release` preset:
```bash
cargo build --release
```

### Running
Start the daemon:
```bash
cargo run -- daemon start
```

> [!TIP]
> You might also want to pass the `-v trace` flag for more debug info.

Then run some client commands:
```bash
cargo run -- playlist list -d
```

## Checks

> [!TIP]
> You can run the following checks at once by running `just check`.

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
To test your code, run:
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

## Use of generative AI
See the [Chilen AI Usage Policy](https://github.com/tpaau/chilen/blob/main/AI_POLICY.md).
