########################################################
# BUILD ARGUMENTS
########################################################
ARCH := riscv64

# run mode
AUTOTEST :=

# smp
export SMP := 


IP_C ?= 10.250.225.200
GW ?= 10.250.0.1
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