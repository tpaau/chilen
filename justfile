check:
	cargo fmt --check --all && cargo build && cargo test && cargo deny check

fmt:
	cargo fmt --all

loc:
	echo "$(cat src/* mpipc/src/* | wc -l) lines of code"

release:
	cargo build --release
