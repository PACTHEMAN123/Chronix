########################################################
# USER
########################################################

# user build mode
USER_MODE := release

# kernel target
ifeq ($(ARCH), riscv64)
USER_TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), loongarch64)
USER_TARGET := loongarch64-unknown-none
endif

# configs
USER_APPS_DIR := ./user/src/bin
USER_TARGET_DIR := ./target/$(USER_TARGET)/$(USER_MODE)
USER_APPS := $(wildcard $(USER_APPS_DIR)/*.rs)
USER_ELFS := $(patsubst $(USER_APPS_DIR)/%.rs, $(USER_TARGET_DIR)/%, $(USER_APPS))

# user build
user:
	$(call building, "building user apps")
	@rm -rf user/.cargo
	@cp -r user/cargo user/.cargo
	@cd user && make build MODE=$(USER_MODE) ARCH=$(ARCH)
	$(call success, "user build finished")

.PHONY: user