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
QEMU_DEV_ARGS += -drive file=/dev/null,if=none,format=raw,id=x0
QEMU_DEV_ARGS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
else ifeq ($(ARCH), loongarch64)
QEMU_RUN_ARGS += -kernel $(KERNEL_ELF) -m 1G
QEMU_DEV_ARGS += -drive file=/dev/null,if=none,format=raw,id=x0
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