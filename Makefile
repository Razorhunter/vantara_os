SHELL := /bin/bash
MAKEFLAGS += --warn-undefined-variables
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c

KERNEL_OUTPUT := build/vmlinuz
KERNEL_DIR := kernel
TARGET := build/VanOS.iso
ROOTFS := build/rootfs
ISO := iso
INITRAMFS := build/initramfs.cpio.gz
MUSL_TARGET := x86_64-unknown-linux-musl
USERLAND := vantara
MOUNT_DIR := build/mnt
IMAGE_FILE := build/vantara.ext4
IMAGE_SIZE := 4096
CHECKSUM_DIR := build/.checksums
OUT_DIR = ../$(ROOTFS)/bin
INIT_OUT_DIR = ../$(ROOTFS)/sbin
BUILD_TARGET = $(MUSL_TARGET)/release
COMMANDS_DIR := vantara/src/commands
PROJECTS := $(notdir $(wildcard $(COMMANDS_DIR)/*))
TZSRC := tzdata2025b
OUT_DIR_TZ := build/zoneinfo
ZONE_INFO_OUT := usr/share/zoneinfo

all: clean build-rootfs build-ext4-image

clean:
	@echo "[Clean] Removing old binaries & initramfs..."
	rm -rf $(ROOTFS)/bin/* $(ROOTFS)/sbin/* $(INITRAMFS) $(ISO) $(TARGET) $(ROOTFS)
	cargo clean --manifest-path $(USERLAND)/src/init/Cargo.toml
	for proj in $(PROJECTS); do \
		cargo clean --target $(MUSL_TARGET) --manifest-path $(USERLAND)/src/commands/$$proj/Cargo.toml; \
	done

copy-kernel:
	@echo "[Copy] Kernel to $(KERNEL_OUTPUT)..."
	cp $(KERNEL_DIR)/arch/x86/boot/bzImage $(KERNEL_OUTPUT)

build-rootfs: $(PROJECTS) init
	@echo "[Copy] Installing to $(ROOTFS)..."

	@echo "[Copy] Binary to $(ROOTFS)/bin..."
	for proj in $(PROJECTS); do \
		cp $(USERLAND)/target/$(MUSL_TARGET)/release/$$proj $(ROOTFS)/bin/$$proj; \
	done

	@echo "[Copy] Init to $(ROOTFS)/sbin..."
	cp $(USERLAND)/target/$(BUILD_TARGET)/init $(ROOTFS)/sbin/init
	
	chmod +x $(ROOTFS)/sbin/init $(ROOTFS)/bin/*

build-initramfs:
	@echo "[Initramfs] Creating initramfs image..."
	cd $(ROOTFS) && find . | cpio -H newc -o | gzip > ../$(INITRAMFS)

build-ext4-image: $(KERNEL_OUTPUT)
	@echo "[Image] Updating ext4 image if needed..."
	@mkdir -p $(MOUNT_DIR) $(CHECKSUM_DIR) $(ROOTFS)

	@echo "[Check] Creating image if not exist..."
	@if [ ! -f "$(IMAGE_FILE)" ]; then \
		echo "[Image] Creating new ext4 image..."; \
		dd if=/dev/zero of="$(IMAGE_FILE)" bs=1M count=$(IMAGE_SIZE); \
		mkfs.ext4 $(IMAGE_FILE); \
		echo "1" > .image_boot_flag; \
	else \
		echo "0" > .image_boot_flag; \
	fi

	@echo "[Mount] Mounting image..."
	sudo mount $(IMAGE_FILE) $(MOUNT_DIR)
	sudo mkdir -p $(MOUNT_DIR)/{bin,sbin,etc,dev,proc,sys,tmp,home,lib,usr,var,mnt}

	@echo "[TZ] Building timezone info..."
	mkdir -p $(OUT_DIR_TZ)
	zic -d $(OUT_DIR_TZ) $(TZSRC)/{africa,antarctica,asia,australasia,backward,backzone,etcetera,europe,northamerica,southamerica}

	@echo "[TZ] Copying to rootfs..."
	sudo mkdir -p $(MOUNT_DIR)/$(ZONE_INFO_OUT)
	sudo cp -a $(OUT_DIR_TZ)/* $(MOUNT_DIR)/$(ZONE_INFO_OUT)

	@echo "[Copy] Updating only changed files..."
	@find $(ROOTFS) -type f | while read f; do \
		dest="$(MOUNT_DIR)/$${f#$(ROOTFS)/}"; \
		src="$$f"; \
		sumfile="$(CHECKSUM_DIR)/$$(echo $$f | tr '/' '_').sha256"; \
		newsum=$$(sha256sum $$src | cut -d' ' -f1); \
		olds=$$(cat $$sumfile 2>/dev/null || echo "none"); \
		if [ "$$newsum" != "$$olds" ]; then \
			echo "	[Update] $$src → $$dest"; \
			sudo install -D $$src $$dest; \
			echo "$$newsum" > $$sumfile; \
		fi; \
	done

	@echo "[Marker] Checking if firstboot marker is needed..."
	@if [ "$$(cat .image_boot_flag)" = "1" ]; then \
		echo "[Marker] Creating /etc/.firstboot"; \
		sudo touch $(MOUNT_DIR)/etc/.firstboot; \
	fi

	sync
	sudo umount $(MOUNT_DIR)
	@rm -f .image_boot_flag
	@echo "[Done] ext4 image updated."


build-iso:
	@echo "[ISO] Generating bootable ISO..."
	mkdir -p $(ISO)/boot/grub
	cp $(KERNEL) $(ISO)/boot/vmlinuz
	cp $(INITRAMFS) $(ISO)/boot/initramfs.gz

	echo 'set timeout=0'                    >  $(ISO)/boot/grub/grub.cfg
	echo 'set default=0'                   >> $(ISO)/boot/grub/grub.cfg
	echo ''                                >> $(ISO)/boot/grub/grub.cfg
	echo 'menuentry "VanOS" {'             >> $(ISO)/boot/grub/grub.cfg
	echo '    linux /boot/vmlinuz'         >> $(ISO)/boot/grub/grub.cfg
	echo '    initrd /boot/initramfs.gz'   >> $(ISO)/boot/grub/grub.cfg
	echo '}'                               >> $(ISO)/boot/grub/grub.cfg

	grub-mkrescue -o $(TARGET) $(ISO)

run-qemu:
	qemu-system-x86_64 \
		-kernel $(KERNEL) \
		-initrd $(INITRAMFS) \
		-device virtio-gpu \
		-append "console=ttyS0 clocksource=tsc" \
		-nographic

run-image:
	qemu-system-x86_64 \
		-kernel $(KERNEL_OUTPUT) \
		-hda $(IMAGE_FILE) \
		-device virtio-rng-pci \
		-append "root=/dev/sda rw console=ttyS0 loglevel=3 clocksource=tsc" \
		-enable-kvm \
		-cpu host \
		-smp 2 \
		-m 2048 \
		-nographic \
  		-serial mon:stdio

clean-checksum:
	rm -rf $(CHECKSUM_DIR)

$(PROJECTS):
	@echo "[*] Building $@..."
	cd $(USERLAND)
	cargo build --release --target $(MUSL_TARGET) -p $@
	mkdir -p $(OUT_DIR)
	cp target/$(BUILD_TARGET)/$@ $(OUT_DIR)/
	chmod u+s $(OUT_DIR)/$@
	@echo "[✓] $@ built and copied to $(OUT_DIR)/"

init:
	@echo "[*] Building init..."
	cd $(USERLAND)
	cargo build --release --target $(MUSL_TARGET) -p init
	mkdir -p $(INIT_OUT_DIR)
	cp target/$(BUILD_TARGET)/init $(INIT_OUT_DIR)/
	chmod u+s $(INIT_OUT_DIR)/init
	@echo "[✓] init built and copied to $(INIT_OUT_DIR)/"

timezone:
	@echo "[TZ] Building timezone info..."
	mkdir -p $(OUT_DIR_TZ)

	zic -d $(OUT_DIR_TZ) $(TZSRC)/{africa,antarctica,asia,australasia,backward,backzone,etcetera,europe,northamerica,southamerica}

	@echo "[TZ] Copying to rootfs..."
	mkdir -p $(MOUNT_DIR)/$(ZONE_INFO_OUT)
	cp -a $(OUT_DIR_TZ)/* $(MOUNT_DIR)/$(ZONE_INFO_OUT)

.PHONY: all clean build-rootfs build-initramfs build-iso run-qemu
