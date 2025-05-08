# global makefile for Chronix

include mk/utils.mk

PHONY_TARGET := 

########################################################
# LEGACY COMMANDS
########################################################

PHONY_TARGET += fs-rv
fs-rv:
	make -f Makefile.sub fs-img ARCH=riscv64

PHONY_TARGET += fs-la
fs-la:
	make -f Makefile.sub fs-img ARCH=riscv64 

PHONY_TARGET += run-rv
run-rv:
	make -f Makefile.sub run ARCH=riscv64 MODE=release

PHONY_TARGET += run-la
run-la:
	make -f Makefile.sub run ARCH=loongarch64 MODE=release



########################################################
# ONLINE JUDGE COMMANDS
########################################################

PHONY_TARGET += all
all: kernel-rv kernel-la disk-img

PHONY_TARGET += setup
setup:
	rm -rf .cargo
	cp -r cargo-config .cargo
	chmod +x scripts/archive.sh
	./scripts/archive.sh extract
	

PHONY_TARGET += kernel-rv
kernel-rv:
	make -f Makefile.sub kernel ARCH=riscv64 MODE=release
	cp os/target/riscv64gc-unknown-none-elf/release/os ./kernel-rv

PHONY_TARGET += kernel-la
kernel-la:
	make -f Makefile.sub kernel ARCH=loongarch64 MODE=release
	cp os/target/loongarch64-unknown-none/release/os ./kernel-la

PHONY_TARGET += oj-run-rv
oj-run-rv:
	qemu-system-riscv64 -machine virt \
		-kernel kernel-rv -m 1G \
		-nographic -smp {smp} \
		-bios default \

#	-drive file={fs},if=none,format=raw,id=x0 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
		-no-reboot \
		-device virtio-net-device,netdev=net -netdev user,id=net \
		-rtc base=utc \
		-drive file=disk-rv.img,if=none,format=raw,id=x1 -device virtio-blk-device,drive=x1,bus=virtio-mmio-bus.1

PHONY_TARGET += oj-run-la
oj-run-la:
	qemu-system-loongarch64 
		-kernel {os_file} -m 1G \ 
		-nographic -smp {smp} \
		-drive file={fs},if=none,format=raw,id=x0  \
		-device virtio-blk-pci,drive=x0,bus=virtio-mmio-bus.0 -no-reboot  -device virtio-net-pci,netdev=net0 \
		-netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555  \
		-rtc base=utc \
		-drive file=disk-la.img,if=none,format=raw,id=x1 -device virtio-blk-pci,drive=x1,bus=virtio-mmio-bus.1

PHONY_TARGET += disk-img
disk-img: setup
	make -f Makefile.sub disk-img ARCH=loongarch64
	make -f Makefile.sub disk-img ARCH=riscv64

PHONY_TARGET += clean
clean:
	make -f Makefile.sub clean ARCH=loongarch64
	make -f Makefile.sub clean ARCH=riscv64
	rm -f kernel-rv kernel-la disk.img disk-rv.img disk-la.img
	rm -rf testcase

.PHONY: $(PHONY_TARGET)