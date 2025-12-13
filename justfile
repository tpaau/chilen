pre-commit:
	cargo fmt --check --all && cargo check && cargo deny check

loc:
	echo "$(cat src/* mpipc/src/* | wc -l) lines of code"
