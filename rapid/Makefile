INSTALL = install -v
INSTALL_DIR = /usr/local/bin
INSTALL_LOCAL_DIR = $(HOME)/.local/bin

.PHONY: all clean install install-local

all:
	cargo build --release

install: all
	$(INSTALL) target/release/rapid $(INSTALL_DIR)

install-local: all
	$(INSTALL) target/release/rapid $(INSTALL_LOCAL_DIR)

clean:
	cargo clean
