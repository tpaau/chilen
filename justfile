check:
	cargo fmt --check --all && cargo test --workspace && cargo deny check

fmt:
	cargo fmt --all

loc:
	cloc build.rs src/ qml/

release:
	cargo build --release
