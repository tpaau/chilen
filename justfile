check:
	cargo fmt --check --all && cargo test --workspace && cargo deny check

fmt:
	cargo fmt --all

release:
	cargo build --release
