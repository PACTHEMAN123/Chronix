########################################################
# QEMU
########################################################

ifeq ($(ARCH), riscv64)
QEMU := qemu-system-riscv64
else ifeq ($(ARCH), loongarch64)
QEMU := qemu-system-loongarch64
endif

CPU := 4
QEMU_ARGS := 
QEMU_ARGS += -machine virt
QEMU_ARGS += -nographic
QEMU_ARGS += -rtc base=utc
QEMU_ARGS += -no-reboot

ifeq ($(ARCH), riscv64)
QEMU_ARGS += -cpu rv64,m=true,a=true,f=true,d=true -m 1G
else ifeq ($(ARCH), loongarch64)
QEMU_ARGS += -m 1G
endif


ifeq ($(ARCH), riscv64)
QEMU_RUN_ARGS := -kernel $(KERNEL_BIN)
else ifeq ($(ARCH), loongarch64)
QEMU_RUN_ARGS := -kernel $(KERNEL_ELF)
endif

ifneq ($(SMP),)
QEMU_ARGS += -smp $(CPU)
endif

ifeq ($(ARCH), riscv64)
QEMU_ARGS += -drive file=$(DISK_IMG_COPY),if=none,format=raw,id=x0
QEMU_ARGS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
QEMU_ARGS += -drive file=sdcard-rv.img,if=none,format=raw,id=x1
QEMU_ARGS += -device virtio-blk-device,drive=x1,bus=virtio-mmio-bus.1
else ifeq ($(ARCH), loongarch64)
QEMU_ARGS += -drive file=$(DISK_IMG_COPY),if=none,format=raw,id=x0
QEMU_ARGS += -device virtio-blk-pci,drive=x0
QEMU_ARGS += -drive file=sdcard-la.img,if=none,format=raw,id=x1
QEMU_ARGS += -device virtio-blk-pci,drive=x1
endif

$(info "enable qemu net device")
ifeq ($(ARCH), riscv64)
QEMU_ARGS += -device virtio-net-device,bus=virtio-mmio-bus.2,netdev=net0\
             -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555
QEMU_ARGS += -d guest_errors\
			 -d unimp
else ifeq ($(ARCH), loongarch64)
QEMU_ARGS += -device virtio-net-pci,netdev=net0\
             -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555
QEMU_ARGS += -d guest_errors\
			 -d unimp
endif

# device tree
DT := $(ARCH)-$(BOARD)
DTB := $(DT).dtb
DTB_DST := hal/src/board/$(ARCH)-$(BOARD).dtb
dumpdtb:
	$(call building, "start to dumpdtb")
	$(QEMU) $(QEMU_ARGS) -machine dumpdtb=$(DTB)
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
debug: build
	$(call building, "cp $(DISK_IMG) to $(DISK_IMG_COPY)")
	@cp $(DISK_IMG) $(DISK_IMG_COPY)
	@tmux new-session -d \
		"$(QEMU) $(QEMU_ARGS) $(QEMU_RUN_ARGS) -s -S" && \
		tmux split-window -h "$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch $(GDB_ARCH)' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

# debug: using gdb server
gdbserver: build
	$(QEMU) $(QEMU_ARGS) $(QEMU_RUN_ARGS) -s -S

# debug: using gdb cilent
gdbclient:
	$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch $(GDB_ARCH)' -ex 'target remote localhost:1234'

.PHONY: qemu-version-check dumpdtb debug gdbserver gdbclient