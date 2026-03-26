check:
	cargo fmt --check --all && cargo test --workspace && cargo deny check

fmt:
	cargo fmt --all

loc:
	cloc src/ mpipc/src/ daemon/src/

release:
	cargo build --release

clean-dirs:
	rm -rf ~/.cache/music-player/
	rm -rf ~/.local/share/music-player/
