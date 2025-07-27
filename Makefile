# global makefile for Chronix
PHONY_TARGET := 

PHONY_TARGET += all
all: disk-img kernel-rv-test kernel-la-test 

PHONY_TARGET += setup
setup:
	rm -rf .cargo
	cp -r cargo .cargo
	

PHONY_TARGET += kernel-rv
kernel-rv: setup
	make -f Makefile.sub kernel-bin ARCH=riscv64
	cp ./target/riscv64gc-unknown-none-elf/release/os.bin ./kernel-rv

PHONY_TARGET += kernel-la
kernel-la: setup
	make -f Makefile.sub kernel ARCH=loongarch64
	cp ./target/loongarch64-unknown-none/release/os ./kernel-la

PHONY_TARGET += kernel-rv-test
kernel-rv-test: setup
	make -f Makefile.sub kernel-bin ARCH=riscv64 AUTOTEST=y
	cp ./target/riscv64gc-unknown-none-elf/release/os.bin ./kernel-rv

PHONY_TARGET += kernel-la
kernel-la-test: setup
	make -f Makefile.sub kernel ARCH=loongarch64 AUTOTEST=y
	cp ./target/loongarch64-unknown-none/release/os ./kernel-la


PHONY_TARGET += disk-img
disk-img: setup
	make -f Makefile.sub disk-img

PHONY_TARGET += run-rv
run-rv: kernel-rv
	make -f Makefile.sub run ARCH=riscv64

PHONY_TARGET += run-la
run-la: kernel-la
	make -f Makefile.sub run ARCH=loongarch64

PHONY_TARGET += test-rv
test-rv: kernel-rv-test
	make -f Makefile.sub run ARCH=riscv64 AUTOTEST=y

PHONY_TARGET += test-la
test-la: kernel-la-test
	make -f Makefile.sub run ARCH=loongarch64 AUTOTEST=y

# replace the GDB to yours
PHONY_TARGET += debug-rv
debug-rv: kernel-rv
	make -f Makefile.sub debug ARCH=riscv64 GDB=gdb-multiarch

PHONY_TARGET += debug-la
debug-la: kernel-la
	make -f Makefile.sub debug ARCH=loongarch64 GDB=loongarch64-linux-gnu-gdb

PHONY_TARGET += zImage-rv
zImage-rv: kernel-rv
#	gzip -f kernel-rv
#	mkimage -A riscv -O linux -C gzip -T kernel -a 0x80200000 -e 0x80200000 -n Chronix -d kernel-rv.gz zImage
	mkimage -A riscv -O linux -C none -T kernel -a 0x80200000 -e 0x80200000 -n Chronix -d kernel-rv zImage

PHONY_TARGET += clean
clean:
	make -f Makefile.sub clean ARCH=loongarch64
	make -f Makefile.sub clean ARCH=riscv64
	rm -f kernel-rv kernel-la disk.img disk-copy.img
	rm -rf testcase

.PHONY: $(PHONY_TARGET)