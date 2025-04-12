# docker
DOCKER_TAG ?= rcore-tutorial-v3:latest
.PHONY: docker build_docker
	
docker:
	docker run --rm -it -v ${PWD}:/mnt -w /mnt --name rcore-tutorial-v3 ${DOCKER_TAG} bash

build_docker: 
	docker build -t ${DOCKER_TAG} --target build .

fmt:
	cd os ; cargo fmt;  cd ..


# copy from os/Makefile


########################################################
# Building
########################################################
ARCH := riscv64

ifeq ($(ARCH), riscv64)
TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), loongarch64)
TARGET := loongarch64-unknown-none
endif

MODE := debug
USER_MODE := $(MODE)

# Kernel
KERNEL_ELF := os/target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := $(KERNEL_ELF).asm

# User
USER_APPS_DIR := ./user/src/bin
USER_TARGET_DIR := ./user/target/$(TARGET)/$(MODE)
USER_APPS := $(wildcard $(USER_APPS_DIR)/*.rs)
USER_ELFS := $(patsubst $(USER_APPS_DIR)/%.rs, $(USER_TARGET_DIR)/%, $(USER_APPS))

# test-suite
TEST_SUITE_DIR := ./vendor/testsuits-for-oskernel

# test-script
TEST_SCRIPT_DIR := ./vendor/oskernel-testsuits-cooperation

# Basic test
BASIC_TEST_DIR := $(TEST_SUITE_DIR)/basic/user/build/${ARCH}

# Busy box
BUSY_BOX_DIR := $(TEST_SUITE_DIR)/busybox
BUSY_BOX := $(TEST_SUITE_DIR)/busybox/busybox_unstripped
BUSY_BOX_TEST_DIR := $(TEST_SCRIPT_DIR)/doc/busybox

# libc-test
LIBC_TEST_BIR := $(TEST_SUITE_DIR)/libc-test
LIBC_TEST_DISK := $(LIBC_TEST_BIR)/disk

export NT :=
# iperf test
IPERF_TEST_DIR := $(TEST_SUITE_DIR)/iperf/riscv-musl

# netperf test
NETPERF_TEST_DIR := $(TEST_SUITE_DIR)/netperf

# BOARD
BOARD := qemu
SBI ?= rustsbi
ifeq ($(ARCH), riscv64)
BOOTLOADER := bootloader/$(SBI)-$(BOARD).bin
else ifeq ($(ARCH), loongarch64)
BOOTLOADER := bootloader/loongarch_bios_0310.bin
endif

KERNEL_FEATURES := 
# Disk file system (default: ext4)
FS := ext4
ifeq ($(FS), fat32)
KERNEL_FEATURES += fat32
endif

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

MODE_ARG += --target $(TARGET)

# Crate features
export SMP := 

ifneq ($(SMP),)
	KERNEL_FEATURES += smp
endif
# KERNEL ENTRY
ifeq ($(ARCH), riscv64)
KERNEL_ENTRY_PA := 0x80200000
else ifeq ($(ARCH), loongarch64)
KERNEL_ENTRY_PA := 0x1c000000
endif

# net
NET_C ?=n
IP_C ?= 10.0.2.15
GW ?= 10.0.2.2
export GATEWAY=$(GW)
export IP=$(IP_C)

ifeq ($(NET_C),y)
	KERNEL_FEATURES += net
endif
# Binutils
OBJDUMP := rust-objdump --arch-name=${ARCH}
OBJCOPY := rust-objcopy --binary-architecture=${ARCH}

ifeq ($(ARCH), riscv64)
GDB ?= riscv64-unknown-elf-gdb
else ifeq ($(ARCH), loongarch64)
GDB ?= loongarch64-linux-gnu-gdb
endif

ifeq ($(ARCH), riscv64)
TOOLCHAIN_PREFIX ?= riscv64-linux-gnu-
else ifeq ($(ARCH), loongarch64)
TOOLCHAIN_PREFIX ?= loongarch64-linux-gnu-
endif


# Disassembly
DISASM ?= -x

build: env $(KERNEL_BIN) user #fs-img: should make fs-img first 

env:
	(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Architecture: $(ARCH)
	@echo Platform: $(BOARD)
	@cp os/src/linker-$(ARCH)-$(BOARD).ld os/src/linker.ld
ifeq ($(KERNEL_FEATURES), ) 
	@cd os && cargo  build $(MODE_ARG)
else
	@cd os && cargo  build $(MODE_ARG) --features "$(KERNEL_FEATURES)"
endif
	@rm os/src/linker.ld

user:
	@echo "building user..."
	@cd user && make build MODE=$(USER_MODE) ARCH=$(ARCH)
	@echo "building user finished"

basic_test:
	@echo "building basic test"
	@cd cross-compiler && tar -xf kendryte-toolchain-ubuntu-amd64-8.2.0-20190409.tar.xz
	@chmod +x vendor/testsuits-for-oskernel/basic/user/build-oscomp.sh 
	@export PATH=$$PATH:cross-compiler/kendryte-toolchain/bin
	@echo "unpack and export cross compiler finish"
	@export ARCH=$(ARCH) && cd vendor/testsuits-for-oskernel/basic/user && ./build-oscomp.sh
	@rm -rf cross-compiler/kendryte-toolchain
	@echo "clean up the cross compiler dir"

ifeq ($(ARCH), riscv64)
CC := riscv64-linux-musl-gcc
STRIP := riscv64-linux-musl-strip
else ifeq ($(ARCH), loongarch64)
CC := loongarch64-linux-musl-gcc
STRIP := loongarch64-linux-musl-strip
endif

busybox:
	@echo "building busybox"
	@make -C $(BUSY_BOX_DIR) clean
	@cp $(TEST_SUITE_DIR)/config/busybox-config-$(ARCH) $(BUSY_BOX_DIR)/.config
	@make -C $(BUSY_BOX_DIR) CC="$(CC) -static -g -Og" STRIP=$(STRIP) -j

libc-test:
	@echo "building libc-test"
	@make -C $(LIBC_TEST_BIR) PREFIX=$(TOOLCHAIN_PREFIX) clean disk


FS_IMG_DIR := .
FS_IMG_NAME := fs-$(ARCH)
FS_IMG := $(FS_IMG_DIR)/$(FS_IMG_NAME).img
fs-img: user basic_test busybox libc-test 
	@echo "building file system image"
	@echo "cleaning up..."
	@rm -f $(FS_IMG)
	@echo "creating dir..."
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
	@echo "making $(FS) image by using $(BASIC_TEST_DIR)"
#	@sudo dd if=/dev/zero of=mnt/swap bs=1M count=128
#	@sudo chmod 0600 mnt/swap
#	@sudo mkswap -L swap mnt/swap
	@echo "copying user apps and tests to the $(FS_IMG)"
	@sudo cp -r $(BASIC_TEST_DIR)/* mnt
	@sudo cp -r $(USER_ELFS) mnt
	@echo "copying busybox to the $(FS_IMG)"
	@sudo cp $(BUSY_BOX) mnt/busybox
	@sudo cp -r $(BUSY_BOX_TEST_DIR)/* mnt
	@sudo mkdir mnt/bin
	@echo "copying libc-test to the $(FS_IMG)"
	@sudo mkdir mnt/libc-test
	@sudo cp $(LIBC_TEST_DISK)/* mnt/libc-test
	@sudo rm mnt/libc-test/run-all.sh
	@sudo mv mnt/libc-test/* mnt/
	@sudo rm -rf mnt/libc-test
ifneq ($(NT),)
	@sudo cp $(IPERF_TEST_DIR)/* mnt/
	@sudo cp $(NETPERF_TEST_DIR)/netserver mnt/
	@sudo cp $(NETPERF_TEST_DIR)/netperf mnt/
	@sudo cp $(NETPERF_TEST_DIR)/netperf_testcode.sh mnt/
endif
	@sudo umount mnt
	@sudo rm -rf mnt
	@sudo chmod 777 $(FS_IMG)
	@echo "building fs-img finished"

clean:
	@cd os && cargo clean
	@cd user && cargo clean
	@sudo rm -f $(FS_IMG)
	@sudo rm -rf mnt
	@sudo rm -rf cross-compiler/kendryte-toolchain
	@make -C $(BUSY_BOX_DIR) clean

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

########################################################
# QEMU
########################################################
CPU := 4
QEMU_ARGS := 
QEMU_ARGS += -machine virt
QEMU_ARGS += -nographic

ifeq ($(ARCH), riscv64)
QEMU_ARGS += -cpu rv64,m=true,a=true,f=true,d=true
QEMU_ARGS += -bios $(BOOTLOADER)
else ifeq ($(ARCH), loongarch64)
endif

ifneq ($(SMP),)
QEMU_ARGS += -smp $(CPU)
endif


ifeq ($(ARCH), riscv64)
QEMU_ARGS += -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
QEMU_ARGS += -drive file=$(FS_IMG),if=none,format=raw,id=x0
QEMU_ARGS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
else ifeq ($(ARCH), loongarch64)
QEMU_ARGS += -kernel $(KERNEL_ELF) -m 1G
QEMU_ARGS += -drive file=$(FS_IMG),if=none,format=raw,id=x0
QEMU_ARGS += -device virtio-blk-pci,drive=x0
endif

ifeq ($(NET_C),y)
$(info "enable qemu net device")
QEMU_ARGS += -device virtio-net-device,bus=virtio-mmio-bus.1,netdev=net0\
             -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555
QEMU_ARGS += -d guest_errors\
			 -d unimp
endif

ifeq ($(ARCH), riscv64)
QEMU := qemu-system-riscv64
GDB_ARCH := riscv:rv64
else ifeq ($(ARCH), loongarch64)
QEMU := qemu-system-loongarch64
GDB_ARCH := Loongarch64
endif

qemu-version-check:
	@sh scripts/qemu-ver-check.sh $(QEMU)

run-inner: qemu-version-check build
	$(QEMU) $(QEMU_ARGS)

run: run-inner

debug: qemu-version-check build
	@tmux new-session -d \
		"$(QEMU) $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch $(GDB_ARCH)' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdbserver: qemu-version-check build
	$(QEMU) $(QEMU_ARGS) -s -S

gdbclient:
	$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch $(GDB_ARCH)' -ex 'target remote localhost:1234'

.PHONY: build env kernel clean disasm disasm-vim run-inner gdbserver gdbclient qemu-version-check fs-img user kernel busybox
