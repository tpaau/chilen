## Use of generative AI
See the [AI Usage Policy](https://github.com/tpaau/chilen/blob/main/AI_POLICY.md).


## Modules

Project modules are managed in a
[Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).

The project has three main workspace members:
- The root is a binary crate with that is a shell built around the Chilen backend
    - [`chilen_backend`](https://tpaau.github.io/chilen/chilen_backend/) - Library crate that
        manages the music library, audio playback, etc. Formerly `chilen_daemon`.


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
