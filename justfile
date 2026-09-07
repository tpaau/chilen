doc-no-all-features:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps -p chilen -p chilen_backend

doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps -p chilen -p chilen_backend --all-features

open-doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps -p chilen -p chilen_backend --all-features --open

test:
	cargo test --workspace
	cargo test --workspace --no-default-features

check:
	cargo fmt --check --all
	just test
	just doc-no-all-features
	just doc
	cargo deny check

loc:
	cloc src/ chilen_backend/src/

clean-cache:
	rm -rf ~/.cache/chilen/

clean-dirs:
	just clean-cache
	rm -rf ~/.local/share/chilen/
