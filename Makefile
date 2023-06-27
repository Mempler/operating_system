ROOT=$(CURDIR)

XTASK=cargo run --manifest-path $(ROOT)/xtask/Cargo.toml --release --

run: build-iso
	$(XTASK) run

build:
	$(XTASK) build

build-hdd: build
#   This is the default behavior of xtask build

build-iso: build
# TODO: move this to xtask for cross platform support
#       for now, we'll convert our bootable disk image to an iso
	$(MAKE) -C $(ROOT)/third_party/limine

	rm -rf $(ROOT)/target/isofiles/
	rm -rf $(ROOT)/target/cd.iso

	mkdir -p $(ROOT)/target/isofiles/
	7z x $(ROOT)/target/disk.img -o$(ROOT)/target/isofiles/ -y

	rm -rf $(ROOT)/target/isofiles/EFI

	cp $(ROOT)/third_party/limine/limine-bios-cd.bin $(ROOT)/target/isofiles/
	cp $(ROOT)/third_party/limine/limine-bios.sys $(ROOT)/target/isofiles/

	cp $(ROOT)/third_party/limine/limine-uefi-cd.bin $(ROOT)/target/isofiles/

	mv $(ROOT)/target/isofiles/LIMINE.CFG $(ROOT)/target/isofiles/limine.cfg

	xorriso -as mkisofs \
		-b limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label \
		$(ROOT)/target/isofiles -o $(ROOT)/target/cd.iso

watch:
	cargo watch -C $(ROOT)/kernel -- $(MAKE) -C $(ROOT) run
