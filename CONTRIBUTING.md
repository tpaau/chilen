## Building

### Prerequisites
You will need Rust and `cargo` to build the project.

Additionally, if you are on Linux, you will need the `alsa-libs-devel` library
installed.

Consider installing [`just`](https://github.com/casey/just) for running some of
the development commands.

## Project structure
```
qml/        # QML files for the GUI
src/
├── cache/  # Music library, demon settings, covers cache
└── daemon/ # Daemon connection handling
mpipc/      # The IPC library
```

### Compiling
Clone the repository and navigate to its root:
```bash
git clone https://codeberg.org/tpaau/music-player
cd music-player
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
Installing `cargo-deny`:
```bash
cargo install --locked cargo-deny
```

Check dependencies:
```bash
cargo deny check
```

## Use of generative AI
Submitting code, documentation, and other text generated with the assistance of
LLMs (Large Language Models) is allowed, as long as it follows the project
guidelines. Content clearly generated with an LLM, with little to no human
intervention, that is of low quality or does not follow the guidelines will be
rejected.

Submitting other media created with generative AI is strictly prohibited. This
includes, but is not limited to:
- Images
- Audio
- Video
