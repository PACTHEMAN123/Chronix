########################################################
# BUILD ARGUMENTS
########################################################
ARCH := riscv64

# run mode
AUTOTEST :=

# smp
export SMP := 


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
VF2 :=

# Binutils
OBJDUMP := rust-objdump --arch-name=${ARCH}
OBJCOPY := rust-objcopy --binary-architecture=${ARCH}