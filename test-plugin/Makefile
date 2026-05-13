.PHONY: all build install clean

TARGET = wasm32-unknown-unknown
WASM  = target/$(TARGET)/release/test_plugin.wasm

all: install

build:
	cargo build --release --target $(TARGET)

install: build
	cp $(WASM) ../plugin/
	@echo "installed test_plugin.wasm -> ../plugin/"

clean:
	cargo clean
	rm -f ../plugin/test_plugin.wasm
