check:
	cargo fmt --check --all
	cargo test --workspace
	cargo test --workspace --no-default-features
	cargo deny check

fmt:
	cargo fmt --all

loc:
	cloc src/ chilen_ipc/src/ chilen_daemon/src/

release:
	cargo build --release

clean-dirs:
	rm -rf ~/.cache/chilen/
	rm -rf ~/.local/share/chilen/
