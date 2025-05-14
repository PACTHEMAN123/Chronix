########################################################
# DISK IMAGE
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

	$(call building, extracting testcase)
	chmod +x scripts/archive.sh
	./scripts/archive.sh extract

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
	@sudo ln -s /glibc/lib/ld-linux-riscv64-lp64d.so.1 mnt/lib/ld-linux-riscv64-lp64d.so.1
	@sudo ln -s /glibc/lib/libc.so mnt/lib/libc.so.6
	@sudo ln -s /glibc/lib/libm.so mnt/lib/libm.so.6
	@sudo ln -s /musl/lib/libc.so mnt/lib/ld-musl-riscv64-sf.so.1
else ifeq ($(ARCH), loongarch64)
	@sudo ln -s /glibc/lib/ld-linux-loongarch-lp64d.so.1 mnt/lib64/ld-linux-loongarch-lp64d.so.1
	@sudo ln -s /glibc/lib/libc.so.6 mnt/lib64/libc.so.6
	@sudo ln -s /glibc/lib/libm.so mnt/lib64/libm.so.6
	@sudo ln -s /musl/lib/libc.so mnt/lib/ld-musl-loongarch64-lp64d.so.1
	@sudo ln -s /musl/lib/libc.so mnt/lib64/ld-musl-loongarch-lp64d.so.1
endif

	@sudo umount mnt
	@sudo rm -rf mnt
	@sudo chmod 777 $(DISK_IMG)
	$(call success, "building $(DISK_IMG) finished")
	@cp $(DISK_IMG) $(DISK_IMG_COPY)
	$(call success, "cp $(DISK_IMG) to $(DISK_IMG_COPY) finished")

.PHONY: disk-img
