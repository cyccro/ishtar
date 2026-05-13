.PHONY: all build check run clean plugin

all: plugin build

build:
	cargo build

release:
	cargo build --release

check:
	cargo check

run:
	cargo run

run-release:
	cargo run --release

clean:
	cargo clean

plugin:
	$(MAKE) -C test-plugin install
