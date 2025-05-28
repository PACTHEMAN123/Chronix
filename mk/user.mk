########################################################
# USER
########################################################

# user build mode
USER_MODE := release

# kernel target
USER_TARGET_RV := riscv64gc-unknown-none-elf
USER_TARGET_LA := loongarch64-unknown-none

# configs
USER_APPS_DIR := ./user/src/bin
USER_TARGET_RV_DIR := ./target/$(USER_TARGET_RV)/$(USER_MODE)
USER_TARGET_LA_DIR := ./target/$(USER_TARGET_LA)/$(USER_MODE)
USER_APPS := $(wildcard $(USER_APPS_DIR)/*.rs)
USER_ELFS_RV := $(patsubst $(USER_APPS_DIR)/%.rs, $(USER_TARGET_RV_DIR)/%, $(USER_APPS))
USER_ELFS_LA := $(patsubst $(USER_APPS_DIR)/%.rs, $(USER_TARGET_LA_DIR)/%, $(USER_APPS))

# user build
user:
	$(call building, "building user apps in la and rv")
	rm -rf user/.cargo
	cp -r user/cargo user/.cargo
	cd user && make build MODE=$(USER_MODE) ARCH=riscv64
	cd user && make build MODE=$(USER_MODE) ARCH=loongarch64
	$(call success, "finish building user apps")

.PHONY: user