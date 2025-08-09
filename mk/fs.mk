########################################################
# DISK IMAGE
########################################################

DISK_IMG := ./disk.img
DISK_IMG_COPY := ./disk-copy.img
TEST_CASE_DIR := ./testcase
disk-img: user
	$(call building, "building $(DISK_IMG) for online judge")

	$(call building, "cleaning up")
	rm -f $(DISK_IMG)

	$(call building, extracting testcase)
	chmod +x scripts/archive.sh
	./scripts/archive.sh extract

	$(call building, "making disk-img dir")
	mkdir -p mnt
	dd if=/dev/zero of=$(DISK_IMG) bs=1M count=4096
	mkfs.ext4 -F -O ^metadata_csum_seed $(DISK_IMG)
	sudo mount $(DISK_IMG) mnt

	$(call building, "start to copy tests from $(TEST_CASE_DIR)")
	sudo cp -r $(TEST_CASE_DIR)/* mnt/

	$(call building, "copying user apps and tests to the $(DISK_IMG)")
	sudo cp -r $(USER_ELFS_RV) mnt/riscv
	sudo cp -r $(USER_ELFS_LA) mnt/loongarch

	$(call building, "copying auto test scripts")
	sudo cp ./scripts/run-rv-oj.sh mnt/riscv/run-oj.sh
	sudo cp ./scripts/run-la-oj.sh mnt/loongarch/run-oj.sh
	sudo cp ./scripts/run-ltp-rv.sh mnt/riscv/run-ltp-rv.sh
	sudo cp ./scripts/run-ltp-la.sh mnt/loongarch/run-ltp-la.sh

	$(call building, "create bin/ lib/ dir")
	sudo mkdir mnt/bin
	sudo mkdir mnt/lib
	sudo mkdir mnt/lib64
	sudo mkdir mnt/usr
	sudo mkdir mnt/usr/lib64

	sudo cp -r attach mnt/
	sudo cp -r etc mnt/

	sudo umount mnt
	sudo rm -rf mnt
	sudo chmod 777 $(DISK_IMG)
	$(call success, "building $(DISK_IMG) finished")
	cp $(DISK_IMG) $(DISK_IMG_COPY)
	$(call success, "cp $(DISK_IMG) to $(DISK_IMG_COPY) finished")

.PHONY: disk-img
