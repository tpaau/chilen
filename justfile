doc:
	cargo doc --no-deps -p chilen -p chilen_backend -p iced_m3 --all-features

open-doc:
	cargo doc --no-deps -p chilen -p chilen_backend -p iced_m3 --all-features --open

test:
	cargo test --workspace
	cargo test --workspace --no-default-features

check:
	cargo fmt --check --all
	just test
	just doc
	cargo deny check

loc:
	cloc src/ iced_m3/src/

release:
	cargo build --release

clean-dirs:
	rm -rf ~/.cache/chilen/
	rm -rf ~/.local/share/chilen/
