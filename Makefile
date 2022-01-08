.PHONY: build clean default test

default: build

clean:
	rm -rf Cargo.lock target/

build:
	cargo build --release

test:
	cargo test
	cargo clippy -- -D warnings
