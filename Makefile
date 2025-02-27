.PHONY: build clean default test

default: build

build:
	cargo build --release

clean:
	rm -rf Cargo.lock target/

outdated:
	cargo-outdated

print:
	git status --porcelain

test:
	cargo test --workspace
	cargo clippy --workspace --tests --examples
	cargo machete
