check:
	cargo fmt --check --all && cargo test --workspace && cargo deny check

fmt:
	cargo fmt --all

loc:
	echo "$(cat src/* mpipc/src/* qml/* | wc -l) lines of code"

release:
	cargo build --release
