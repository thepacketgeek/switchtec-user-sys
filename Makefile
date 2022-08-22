test: lint
	RUST_MIN_STACK=7680000 cargo test -q

doc:
	cargo doc --open

lint:
	cargo fmt --message-format human -- --check
	cargo check
	RUSTDOCFLAGS=-Dwarnings cargo doc -q --lib
	cargo clippy -q --no-deps -- -D warnings