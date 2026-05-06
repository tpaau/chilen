doc:
	cargo doc --frozen --no-deps --all-features

open-doc:
	cargo doc --frozen --no-deps --all-features --open

test:
	cargo test --workspace
	cargo test --workspace --no-default-features

check:
	cargo fmt --check --all
	just test
	cargo deny check
	just doc

fmt:
	cargo fmt --all

loc:
	cloc src/ chilen_ipc/src/ chilen_daemon/src/

release:
	cargo build --release

clean-dirs:
	rm -rf ~/.cache/chilen/
	rm -rf ~/.local/share/chilen/
