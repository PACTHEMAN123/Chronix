# Makefile for Chronix

########################################################
# BUILD ARGUMENTS
########################################################
ARCH := riscv64

# build target
ifeq ($(ARCH), riscv64)
TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), loongarch64)
TARGET := loongarch64-unknown-none
endif


# Building mode argument
MODE := debug
ifeq ($(MODE), release)
	MODE_ARG := --release
endif
MODE_ARG += --target $(TARGET)

# smp
export SMP := 

# net
NET_C ?=n
IP_C ?= 10.0.2.15
GW ?= 10.0.2.2
export GATEWAY=$(GW)
export IP=$(IP_C)
export NT :=

# Disk file system
FS := ext4

# board
BOARD := qemu
SBI ?= rustsbi

# Binutils
OBJDUMP := rust-objdump --arch-name=${ARCH}
OBJCOPY := rust-objcopy --binary-architecture=${ARCH}

# boot loader
ifeq ($(ARCH), riscv64)
BOOTLOADER := bootloader/$(SBI)-$(BOARD).bin
else ifeq ($(ARCH), loongarch64)
BOOTLOADER := bootloader/loongarch_bios_0310.bin
endif

# sdcard
ifeq ($(ARCH), riscv64)
SDCARD := sdcard-rv.img
else ifeq ($(ARCH), loongarch64)
SDCARD := sdcard-la.img
endif

########################################################
# KERNEL
########################################################

# features
KERNEL_FEATURES := 

ifeq ($(FS), fat32)
KERNEL_FEATURES += fat32
endif

ifneq ($(SMP),)
KERNEL_FEATURES += smp
endif

ifeq ($(NET_C),y)
KERNEL_FEATURES += net
endif

# kernel entry
ifeq ($(ARCH), riscv64)
KERNEL_ENTRY_PA := 0x80200000
else ifeq ($(ARCH), loongarch64)
KERNEL_ENTRY_PA := 0x1c000000
endif

KERNEL_ELF := os/target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := $(KERNEL_ELF).asm

# kernel in binary
$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

# kernel in elf
kernel: dumpdtb
	$(call building, "Architecture: $(ARCH)")
	$(call building, "Platform: $(BOARD)")
	@cp os/src/linker-$(ARCH)-$(BOARD).ld os/src/linker.ld
ifeq ($(KERNEL_FEATURES), ) 
	@cd os && cargo  build $(MODE_ARG)
else
	@cd os && cargo  build $(MODE_ARG) --features "$(KERNEL_FEATURES)"
endif
	@rm os/src/linker.ld
	$(call success, "kernel $(KERNEL_ELF) finish building")

# Disassembly
DISASM ?= -x
disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

# code format
fmt:
	cd os ; cargo fmt;  cd ..

.PHONY: kernel disasm disasm-vim fmt $(KERNEL_BIN)

########################################################
# ROOT FILE SYSTEM IMAGE
########################################################

FS_IMG_DIR := .
FS_IMG_NAME := fs-$(ARCH)
FS_IMG := $(FS_IMG_DIR)/$(FS_IMG_NAME).img
FS_IMG_COPY := $(FS_IMG_DIR)/fs.img
fs-img: user basic_test busybox libc-test lua
	$(call building, "building file system image")
	$(call building, "cleaning up...")
	@rm -f $(FS_IMG)
	$(call building, "making fs-img dir")
	@mkdir -p $(FS_IMG_DIR)
	@mkdir -p mnt

ifeq ($(FS), fat32)
	dd if=/dev/zero of=$(FS_IMG) bs=1k count=1363148
	@mkfs.vfat -F 32 -s 8 $(FS_IMG)
	@sudo mount -t vfat -o user,umask=000,utf8=1 --source $(FS_IMG) --target mnt
else
	dd if=/dev/zero of=$(FS_IMG) bs=1M count=2048
	@mkfs.ext4 -F -O ^metadata_csum_seed $(FS_IMG)
	@sudo mount $(FS_IMG) mnt
endif

	$(call building, "making $(FS) image")
#	@sudo dd if=/dev/zero of=mnt/swap bs=1M count=128
#	@sudo chmod 0600 mnt/swap
#	@sudo mkswap -L swap mnt/swap
	$(call building, "copying user apps and tests to the $(FS_IMG)")
	@sudo cp -r $(BASIC_TEST_DIR)/* mnt
	@sudo cp -r $(USER_ELFS) mnt

	$(call building, "copying busybox to the $(FS_IMG)")
	@sudo cp $(BUSY_BOX) mnt/busybox
	@sudo cp -r $(BUSY_BOX_TEST_DIR)/* mnt
	@sudo mkdir mnt/bin

	$(call building, "copying lua to the $(FS_IMG)")
	@sudo cp $(LUA) mnt/
	@sudo cp $(LUA_TEST_DIR)/* mnt/

	$(call building, "copying libc-test to the $(FS_IMG)")
	@sudo mkdir mnt/libc-test
	@sudo cp $(LIBC_TEST_DISK)/* mnt/libc-test
#	@sudo cp $(LIBC_TEST_BIR)/all.sh mnt/libc-test

ifneq ($(NT),)
	$(call building, "copying netperf to the $(FS_IMG)")
	@sudo cp $(IPERF_TEST_DIR)/* mnt/
	@sudo cp $(NETPERF_TEST_DIR)/netserver mnt/
	@sudo cp $(NETPERF_TEST_DIR)/netperf mnt/
	@sudo cp $(NETPERF_TEST_DIR)/netperf_testcode.sh mnt/
endif

	$(call building, "copying libc.so")
	@sudo mkdir -p sdcard
	@sudo mount $(SDCARD) sdcard
	@sudo mkdir -p mnt/lib
	@sudo cp -r sdcard/musl/lib/libc.so mnt/lib
ifeq ($(ARCH), riscv64)
#	@sudo ln mnt/lib/libc.so mnt/lib/ld-linux-riscv64-lp64.so.1
#	@sudo ln mnt/lib/libc.so mnt/lib/ld-musl-riscv64.so.1
	@sudo ln -s /lib/libc.so mnt/lib/ld-linux-riscv64-lp64.so.1
	@sudo ln -s /lib/libc.so mnt/lib/ld-musl-riscv64.so.1
else ifeq ($(ARCH), loongarch64)
	@sudo mkdir -p mnt/lib64
	@sudo ln mnt/lib/libc.so mnt/lib64/ld-linux-loongarch-lp64d.so.1
	@sudo ln mnt/lib/libc.so mnt/lib/ld-musl-riscv64.so.1
endif

	@sudo umount sdcard
	@sudo rm -rf sdcard
	@sudo umount mnt
	@sudo rm -rf mnt
	@sudo chmod 777 $(FS_IMG)
	$(call success, "building $(FS_IMG) finished")
	@cp $(FS_IMG) $(FS_IMG_COPY)


.PHONY: fs-img

########################################################
# QEMU
########################################################

ifeq ($(ARCH), riscv64)
QEMU := qemu-system-riscv64
else ifeq ($(ARCH), loongarch64)
QEMU := qemu-system-loongarch64
endif

CPU := 4
QEMU_DEV_ARGS := 
QEMU_RUN_ARGS :=
QEMU_DEV_ARGS += -machine virt
QEMU_DEV_ARGS += -nographic

ifeq ($(ARCH), riscv64)
QEMU_DEV_ARGS += -cpu rv64,m=true,a=true,f=true,d=true
QEMU_DEV_ARGS += -bios $(BOOTLOADER)
else ifeq ($(ARCH), loongarch64)
endif

ifneq ($(SMP),)
QEMU_DEV_ARGS += -smp $(CPU)
endif


ifeq ($(ARCH), riscv64)
QEMU_RUN_ARGS += -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
# QEMU_ARGS += -kernel $(KERNEL_ELF) -m 1G
QEMU_DEV_ARGS += -drive file=$(FS_IMG_COPY),if=none,format=raw,id=x0
QEMU_DEV_ARGS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
else ifeq ($(ARCH), loongarch64)
QEMU_RUN_ARGS += -kernel $(KERNEL_ELF) -m 1G
QEMU_DEV_ARGS += -drive file=$(FS_IMG_COPY),if=none,format=raw,id=x0
QEMU_DEV_ARGS += -device virtio-blk-pci,drive=x0
endif

ifeq ($(NET_C),y)
$(info "enable qemu net device")
QEMU_DEV_ARGS += -device virtio-net-device,bus=virtio-mmio-bus.1,netdev=net0\
             -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555
QEMU_DEV_ARGS += -d guest_errors\
			 -d unimp
endif

# check the qemu version
qemu-version-check:
	@sh scripts/qemu-ver-check.sh $(QEMU)

# device tree
DT := $(ARCH)-$(BOARD)
DTB := $(DT).dtb
DTB_DST := os/src/devices/dtree.dtb
dumpdtb:
	$(call building, "start to dumpdtb")
	$(QEMU) $(QEMU_DEV_ARGS) -machine dumpdtb=$(DTB)
	$(call building, "moving $(DTB) to $(DTB_DST)")
	mv $(DTB) $(DTB_DST)
	$(call success, "dumpdtb finish")
		

# debug configs
ifeq ($(ARCH), riscv64)
GDB ?= riscv64-unknown-elf-gdb
GDB_ARCH := riscv:rv64
else ifeq ($(ARCH), loongarch64)
GDB ?= loongarch64-linux-gnu-gdb
GDB_ARCH := Loongarch64
endif

# debug: using tmux
debug: qemu-version-check build
	@tmux new-session -d \
		"$(QEMU) $(QEMU_DEV_ARGS) $(QEMU_RUN_ARGS) -s -S" && \
		tmux split-window -h "$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch $(GDB_ARCH)' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

# debug: using gdb server
gdbserver: qemu-version-check build
	$(QEMU) $(QEMU_DEV_ARGS) $(QEMU_RUN_ARGS) -s -S

# debug: using gdb cilent
gdbclient:
	$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch $(GDB_ARCH)' -ex 'target remote localhost:1234'

.PHONY: qemu-version-check dumpdtb debug gdbserver gdbclient

########################################################
# COMMANDS (global quick command)
########################################################

build: env $(KERNEL_BIN) #fs-img: should make fs-img first
	$(call building, "cp $(FS_IMG) to $(FS_IMG_COPY)")
	@cp $(FS_IMG) $(FS_IMG_COPY)

env:
	(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

run-inner: qemu-version-check build
	$(QEMU) $(QEMU_DEV_ARGS) $(QEMU_RUN_ARGS)

run: run-inner

clean:
	@cd os && cargo clean
	@cd user && cargo clean
	@sudo rm -f $(FS_IMG)
	@sudo rm -rf mnt
	@sudo rm -rf cross-compiler/kendryte-toolchain
	@make -C $(BUSY_BOX_DIR) clean

.PHONY: build env run-inner run clean

########################################################
# USER
########################################################

# configs
USER_APPS_DIR := ./user/src/bin
USER_TARGET_DIR := ./user/target/$(TARGET)/$(MODE)
USER_APPS := $(wildcard $(USER_APPS_DIR)/*.rs)
USER_ELFS := $(patsubst $(USER_APPS_DIR)/%.rs, $(USER_TARGET_DIR)/%, $(USER_APPS))

USER_MODE := $(MODE)

# user build
user:
	$(call building, "building user apps")
	@cd user && make build MODE=$(USER_MODE) ARCH=$(ARCH)
	$(call success, "user build finished")

.PHONY: user

########################################################
# TESTS
########################################################

# test-suite
TEST_SUITE_DIR := ./vendor/testsuits-for-oskernel

# test-script
TEST_SCRIPT_DIR := ./vendor/oskernel-testsuits-cooperation

# toolchains
ifeq ($(ARCH), riscv64)
CC := riscv64-linux-musl-gcc
STRIP := riscv64-linux-musl-strip
else ifeq ($(ARCH), loongarch64)
CC := loongarch64-linux-musl-gcc
STRIP := loongarch64-linux-musl-strip
endif

ifeq ($(ARCH), riscv64)
TOOLCHAIN_PREFIX ?= riscv64-linux-gnu-
else ifeq ($(ARCH), loongarch64)
TOOLCHAIN_PREFIX ?= loongarch64-linux-gnu-
endif

# Basic test
BASIC_TEST_DIR := $(TEST_SUITE_DIR)/basic/user/build/${ARCH}
basic_test:
	$(call building, "building basic test")
	@cd cross-compiler && tar -xf kendryte-toolchain-ubuntu-amd64-8.2.0-20190409.tar.xz
	@chmod +x vendor/testsuits-for-oskernel/basic/user/build-oscomp.sh 
	@export PATH=$$PATH:cross-compiler/kendryte-toolchain/bin
	$(call success, "unpack and export cross compiler finish")
	@export ARCH=$(ARCH) && cd vendor/testsuits-for-oskernel/basic/user && ./build-oscomp.sh
	@rm -rf cross-compiler/kendryte-toolchain
	$(call success, "basic test build finished")

# Busy box
BUSY_BOX_DIR := $(TEST_SUITE_DIR)/busybox
BUSY_BOX := $(TEST_SUITE_DIR)/busybox/busybox_unstripped
BUSY_BOX_TEST_DIR := $(TEST_SCRIPT_DIR)/doc/busybox
busybox:
	$(call building, "building busybox")
	@make -C $(BUSY_BOX_DIR) clean
	@cp $(TEST_SUITE_DIR)/config/busybox-config-$(ARCH) $(BUSY_BOX_DIR)/.config
	@make -C $(BUSY_BOX_DIR) CC="$(CC) -static -g -Og" STRIP=$(STRIP) -j
	$(call success, "busybox build finished")

# lua
LUA_DIR := $(TEST_SUITE_DIR)/lua
LUA := $(LUA_DIR)/src/lua
LUA_TEST_DIR := $(TEST_SUITE_DIR)/scripts/lua
lua:
	$(call building, "building lua")
	@make -C $(LUA_DIR) clean
	@make -C $(LUA_DIR) CC="$(CC) -static -g -Og" -j $(NPROC) 
	$(call success, "lua build finished")

# libc-test
LIBC_TEST_BIR := $(TEST_SUITE_DIR)/libc-test
LIBC_TEST_DISK := $(LIBC_TEST_BIR)/disk
libc-test:
	$(call building, "building libc-test")
	@make -C $(LIBC_TEST_BIR) PREFIX=riscv64-buildroot-linux-musl- clean disk
	$(call success, "libc test build finished")


# iperf test
IPERF_TEST_DIR := $(TEST_SUITE_DIR)/iperf/riscv-musl

# netperf test
NETPERF_TEST_DIR := $(TEST_SUITE_DIR)/netperf

.PHONY: basic_test busybox lua libc-test

########################################################
# DOCKER (unused)
########################################################

# docker
DOCKER_TAG ?= rcore-tutorial-v3:latest
.PHONY: docker build_docker
	
docker:
	docker run --rm -it -v ${PWD}:/mnt -w /mnt --name rcore-tutorial-v3 ${DOCKER_TAG} bash

build_docker: 
	docker build -t ${DOCKER_TAG} --target build .

.PHONY: docker build_docker

########################################################
# UTILS (for prettier building process)
########################################################

RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[0;33m
PURPLE = \033[0;95m
RESET = \033[0m
BOLD = \033[1m

define building
	@echo "${BOLD}${PURPLE} [BUILDING] ${1}${RESET}"
endef

define success
	@echo "${BOLD}${GREEN} [SUCCESS ] ${1}${RESET}"
endef
