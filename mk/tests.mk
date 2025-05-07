########################################################
# TESTS (build from testsuits-for-oskernel)
########################################################

# test-suite
TEST_SUITE_DIR := ./vendor/testsuits-for-oskernel

# test-script
TEST_SCRIPT_DIR := ./vendor/oskernel-testsuits-cooperation

# toolchains
ifeq ($(ARCH), riscv64)
CC := riscv64-linux-musl-gcc
STRIP := riscv64-linux-musl-strip
else ifeq ($(ARCH), loongarch64)
CC := loongarch64-linux-musl-gcc
STRIP := loongarch64-linux-musl-strip
endif

ifeq ($(ARCH), riscv64)
TOOLCHAIN_PREFIX ?= riscv64-linux-gnu-
else ifeq ($(ARCH), loongarch64)
TOOLCHAIN_PREFIX ?= loongarch64-linux-gnu-
endif

# Basic test
BASIC_TEST_DIR := $(TEST_SUITE_DIR)/basic/user/build/${ARCH}
basic_test:
	$(call building, "building basic test")
	@cd cross-compiler && tar -xf kendryte-toolchain-ubuntu-amd64-8.2.0-20190409.tar.xz
	@chmod +x vendor/testsuits-for-oskernel/basic/user/build-oscomp.sh 
	@export PATH=$$PATH:cross-compiler/kendryte-toolchain/bin
	$(call success, "unpack and export cross compiler finish")
	@export ARCH=$(ARCH) && cd vendor/testsuits-for-oskernel/basic/user && ./build-oscomp.sh
	@rm -rf cross-compiler/kendryte-toolchain
	$(call success, "basic test build finished")

# Busy box
BUSY_BOX_DIR := $(TEST_SUITE_DIR)/busybox
BUSY_BOX := $(TEST_SUITE_DIR)/busybox/busybox_unstripped
BUSY_BOX_TEST_DIR := $(TEST_SCRIPT_DIR)/doc/busybox
busybox:
	$(call building, "building busybox")
	@make -C $(BUSY_BOX_DIR) clean
	@cp $(TEST_SUITE_DIR)/config/busybox-config-$(ARCH) $(BUSY_BOX_DIR)/.config
	@make -C $(BUSY_BOX_DIR) CC="$(CC) -static -g -Og" STRIP=$(STRIP) -j
	$(call success, "busybox build finished")

# lua
LUA_DIR := $(TEST_SUITE_DIR)/lua
LUA := $(LUA_DIR)/src/lua
LUA_TEST_DIR := $(TEST_SUITE_DIR)/scripts/lua
lua:
	$(call building, "building lua")
	@make -C $(LUA_DIR) clean
	@make -C $(LUA_DIR) CC="$(CC) -static -g -Og" -j $(NPROC) 
	$(call success, "lua build finished")

# libc-test
LIBC_TEST_BIR := $(TEST_SUITE_DIR)/libc-test
LIBC_TEST_DISK := $(LIBC_TEST_BIR)/disk
libc-test:
	$(call building, "building libc-test")
	@make -C $(LIBC_TEST_BIR) PREFIX=riscv64-linux-gnu- clean disk
	$(call success, "libc test build finished")


# iperf test
IPERF_TEST_DIR := $(TEST_SUITE_DIR)/iperf/riscv-musl

# netperf test
NETPERF_TEST_DIR := $(TEST_SUITE_DIR)/netperf

.PHONY: basic_test busybox lua libc-test