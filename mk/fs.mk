########################################################
# ROOT FILE SYSTEM IMAGE
########################################################

# fs-img for local develop
FS_IMG_DIR := .
FS_IMG_NAME := fs-$(ARCH)
FS_IMG := $(FS_IMG_DIR)/$(FS_IMG_NAME).img
FS_IMG_COPY := $(FS_IMG_DIR)/fs.img
fs-img: user basic_test busybox libc-test lua
	$(call building, "building file system image")
	$(call building, "cleaning up...")
	@rm -f $(FS_IMG)
	$(call building, "making fs-img dir")
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

	$(call building, "making $(FS) image")
#	@sudo dd if=/dev/zero of=mnt/swap bs=1M count=128
#	@sudo chmod 0600 mnt/swap
#	@sudo mkswap -L swap mnt/swap
	$(call building, "copying user apps and tests to the $(FS_IMG)")
	@sudo cp -r $(BASIC_TEST_DIR)/* mnt
	@sudo cp -r $(USER_ELFS) mnt

	$(call building, "copying busybox to the $(FS_IMG)")
	@sudo cp $(BUSY_BOX) mnt/busybox
	@sudo cp -r $(BUSY_BOX_TEST_DIR)/* mnt
	@sudo mkdir mnt/bin

	$(call building, "copying lua to the $(FS_IMG)")
	@sudo cp $(LUA) mnt/
	@sudo cp $(LUA_TEST_DIR)/* mnt/

	$(call building, "copying libc-test to the $(FS_IMG)")
	@sudo mkdir mnt/libc-test
	@sudo cp $(LIBC_TEST_DISK)/* mnt/libc-test
	@sudo cp $(LIBC_TEST_BIR)/all.sh mnt/libc-test

ifneq ($(NT),)
	$(call building, "copying netperf to the $(FS_IMG)")
	@sudo cp $(IPERF_TEST_DIR)/* mnt/
	@sudo cp $(NETPERF_TEST_DIR)/netserver mnt/
	@sudo cp $(NETPERF_TEST_DIR)/netperf mnt/
	@sudo cp $(NETPERF_TEST_DIR)/netperf_testcode.sh mnt/
endif

	$(call building, "copying libc.so")
	@sudo mkdir -p sdcard
	@sudo mount $(SDCARD) sdcard
	@sudo mkdir -p mnt/lib
	@sudo cp -r sdcard/musl/lib/libc.so mnt/lib
ifeq ($(ARCH), riscv64)
#	@sudo ln mnt/lib/libc.so mnt/lib/ld-linux-riscv64-lp64.so.1
#	@sudo ln mnt/lib/libc.so mnt/lib/ld-musl-riscv64.so.1
	@sudo ln -s /lib/libc.so mnt/lib/ld-linux-riscv64-lp64.so.1
	@sudo ln -s /lib/libc.so mnt/lib/ld-musl-riscv64.so.1
else ifeq ($(ARCH), loongarch64)
	@sudo mkdir -p mnt/lib64
	@sudo ln mnt/lib/libc.so mnt/lib64/ld-linux-loongarch-lp64d.so.1
	@sudo ln mnt/lib/libc.so mnt/lib/ld-musl-riscv64.so.1
endif

	@sudo umount sdcard
	@sudo rm -rf sdcard
	@sudo umount mnt
	@sudo rm -rf mnt
	@sudo chmod 777 $(FS_IMG)
	$(call success, "building $(FS_IMG) finished")
	@cp $(FS_IMG) $(FS_IMG_COPY)
	$(call success, "cp $(FS_IMG) to $(FS_IMG_COPY) finished")

.PHONY: fs-img

########################################################
# DISK IMAGE (for online judge)
########################################################

ifeq ($(ARCH), riscv64)
DISK_IMG := ./disk-rv.img
TEST_CASE_DIR := ./testcase/riscv
else ifeq ($(ARCH), loongarch64)
DISK_IMG := ./disk-la.img
TEST_CASE_DIR := ./testcase/loongarch
endif
DISK_IMG_COPY := ./disk.img
disk-img: user
	$(call building, "building $(DISK_IMG) for online judge")

	$(call building, "cleaning up")
	@rm -f $(DISK_IMG)

	$(call building, "making disk-img dir")
	@mkdir -p mnt
	dd if=/dev/zero of=$(DISK_IMG) bs=1M count=4096
	@mkfs.ext4 -F -O ^metadata_csum_seed $(DISK_IMG)
	@sudo mount $(DISK_IMG) mnt

	$(call building, "start to copy tests from $(TEST_CASE_DIR)")
	@sudo cp -r $(TEST_CASE_DIR)/* mnt/

	$(call building, "copying user apps and tests to the $(FS_IMG)")
	@sudo cp -r $(USER_ELFS) mnt

	$(call building, "create bin/ lib/ dir")
	@sudo mkdir mnt/bin
	@sudo mkdir mnt/lib
	@sudo mkdir mnt/lib64

	$(call building, "sym-linking libc.so")
ifeq ($(ARCH), riscv64)
	@sudo ln -s /glibc/lib/ld-linux-riscv64-lp64d.so.1 mnt/lib/ld-linux-riscv64-lp64.so.1
	@sudo ln -s /glibc/lib/libc.so.6 mnt/lib/libc.so.6
	@sudo ln -s /musl/lib/libc.so mnt/lib/ld-musl-riscv64.so.1
else ifeq ($(ARCH), loongarch64)
	@sudo ln -s /glibc/lib/ld-linux-loongarch-lp64d.so.1 mnt/lib64/ld-linux-loongarch-lp64d.so.1
	@sudo ln -s /glibc/lib/libc.so.6 mnt/lib64/libc.so.6
	@sudo ln -s /musl/lib/libc.so mnt/lib/ld-musl-riscv64.so.1
endif

	@sudo umount mnt
	@sudo rm -rf mnt
	@sudo chmod 777 $(DISK_IMG)
	$(call success, "building $(DISK_IMG) finished")
	@cp $(DISK_IMG) $(DISK_IMG_COPY)
	$(call success, "cp $(DISK_IMG) to $(DISK_IMG_COPY) finished")
