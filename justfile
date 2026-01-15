check:
	cargo fmt --check --all && cargo test --workspace && cargo deny check

fmt:
	cargo fmt --all

loc:
	cloc build.rs src/ mpipc/src/ daemon/src/ qml/

release:
	cargo build --release
