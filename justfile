doc:
	cargo doc --no-deps -p chilen_daemon -p chilen_ipc -p m3u8 -p lrc_rs --all-features

open-doc:
	cargo doc --no-deps -p chilen_daemon -p chilen_ipc -p m3u8 -p lrc_rs --all-features --open

test:
	cargo test --workspace
	cargo test --workspace --no-default-features

check:
	cargo fmt --check --all
	just test
	just doc
	cargo deny check

fmt:
	cargo fmt --all

loc:
	cloc src/ chilen_ipc/src/ chilen_daemon/src/ chilen_daemon/m3u8/src/ chilen_daemon/lrc_rs/src

release:
	cargo build --release

clean-dirs:
	rm -rf ~/.cache/chilen/
	rm -rf ~/.local/share/chilen/
