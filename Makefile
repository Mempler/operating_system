ROOT=$(CURDIR)

XTASK=cargo run --manifest-path $(ROOT)/xtask/Cargo.toml --release --

run:
	$(XTASK) run

boot: run

build:
	$(XTASK) build

watch:
	cargo watch -C $(ROOT)/kernel -- $(MAKE) -C $(ROOT) run
