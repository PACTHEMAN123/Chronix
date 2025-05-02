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