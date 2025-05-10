# global makefile for Chronix
PHONY_TARGET := 

PHONY_TARGET += all
all: kernel-rv kernel-la disk-img

PHONY_TARGET += setup
setup:
	rm -rf .cargo
	cp -r cargo .cargo
	

PHONY_TARGET += kernel-rv
kernel-rv: setup
	make -f Makefile.sub os/target/riscv64gc-unknown-none-elf/release/os.bin ARCH=riscv64
	cp os/target/riscv64gc-unknown-none-elf/release/os.bin ./kernel-rv

PHONY_TARGET += kernel-la
kernel-la: setup
	make -f Makefile.sub kernel ARCH=loongarch64
	cp os/target/loongarch64-unknown-none/release/os ./kernel-la

PHONY_TARGET += disk-img
disk-img: setup
	make -f Makefile.sub disk-img ARCH=loongarch64
	make -f Makefile.sub disk-img ARCH=riscv64

PHONY_TARGET += run-rv
run-rv: kernel-rv
	make -f Makefile.sub run ARCH=riscv64

PHONY_TARGET += run-la
run-la: kernel-la
	make -f Makefile.sub run ARCH=loongarch64

PHONY_TARGET += debug-rv
debug-rv: kernel-rv
	make -f Makefile.sub debug ARCH=riscv64 GDB=gdb-multiarch

PHONY_TARGET += clean
clean:
	make -f Makefile.sub clean ARCH=loongarch64
	make -f Makefile.sub clean ARCH=riscv64
	rm -f kernel-rv kernel-la disk.img disk-rv.img disk-la.img
	rm -rf testcase

.PHONY: $(PHONY_TARGET)