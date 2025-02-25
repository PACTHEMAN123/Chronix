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
TARGET := riscv64gc-unknown-none-elf
MODE := debug

KERNEL_ELF := os/target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := $(KERNEL_ELF).asm

USER_APPS_DIR := ./user/src/bin
USER_TARGET_DIR := ./user/target/$(TARGET)/$(MODE)
USER_APPS := $(wildcard $(USER_APPS_DIR)/*.rs)
USER_ELFS := $(patsubst $(USER_APPS_DIR)/%.rs, $(USER_TARGET_DIR)/%, $(USER_APPS))
USER_BINS := $(patsubst $(USER_APPS_DIR)/%.rs, $(USER_TARGET_DIR)/%.bin, $(USER_APPS))

TEST_DIR := ./test/

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := bootloader/$(SBI)-$(BOARD).bin

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
GDB ?= riscv64-unknown-elf-gdb

# Disassembly
DISASM ?= -x


build: env $(KERNEL_BIN) user fs-img

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Platform: $(BOARD)
	@cp os/src/linker-$(BOARD).ld os/src/linker.ld
	@cd os && cargo build $(MODE_ARG)
	@rm os/src/linker.ld

user:
	@echo "building user..."
	@cd user && make build MODE=$(MODE)
	@$(foreach elf, $(USER_ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)
	@echo "building user finished"

FS_IMG_DIR := .
FS_IMG := $(FS_IMG_DIR)/fs.img
fs-img: $(KERNEL_BIN) user
	@echo "building file system image"
	@echo "cleaning up..."
	@rm -f $(FS_IMG)
	@echo "creating dir..."
	@mkdir -p $(FS_IMG_DIR)
	@mkdir -p mnt
	dd if=/dev/zero of=$(FS_IMG) bs=1M count=2048
	@mkfs.ext4 -F -O ^metadata_csum_seed $(FS_IMG)
	@echo "making ext4 image by using $(TEST_DIR)"
	@sudo mount $(FS_IMG) mnt
	@echo "copying user apps and tests to the fs.img"
	@sudo cp -r $(TEST_DIR)/* mnt
	@sudo cp -r $(USER_ELFS) mnt
	@sudo umount mnt
	@sudo rm -rf mnt
	@sudo chmod 777 $(FS_IMG)
	@echo "building fs-img finished"

clean:
	@cd os && cargo clean
	@cd user && cargo clean
	@sudo rm -f $(FS_IMG)
	@sudo rm -rf mnt 

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

########################################################
# QEMU
########################################################
QEMU_ARGS := 
QEMU_ARGS += -machine virt
QEMU_ARGS += -nographic
QEMU_ARGS += -bios $(BOOTLOADER)
QEMU_ARGS += -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

# for fs.img
QEMU_ARGS += -drive file=$(FS_IMG),format=raw,id=x0,if=none
QEMU_ARGS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

QEMU := qemu-system-riscv64
qemu-version-check:
	@sh scripts/qemu-ver-check.sh $(QEMU)

run-inner: qemu-version-check build
	$(QEMU) $(QEMU_ARGS)

run: run-inner

debug: qemu-version-check build
	@tmux new-session -d \
		"qemu-system-riscv64 $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdbserver: qemu-version-check build
	$(QEMU) $(QEMU_ARGS) -s -S

gdbclient:
	$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

.PHONY: build env kernel clean disasm disasm-vim run-inner gdbserver gdbclient qemu-version-check fs-img user kernel
