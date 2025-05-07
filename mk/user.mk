########################################################
# USER
########################################################

# configs
USER_APPS_DIR := ./user/src/bin
USER_TARGET_DIR := ./user/target/$(TARGET)/$(MODE)
USER_APPS := $(wildcard $(USER_APPS_DIR)/*.rs)
USER_ELFS := $(patsubst $(USER_APPS_DIR)/%.rs, $(USER_TARGET_DIR)/%, $(USER_APPS))

USER_MODE := $(MODE)

# user build
user:
	$(call building, "building user apps")
	@cp -r user/cargo-config user/.cargo
	@cd user && make build MODE=$(USER_MODE) ARCH=$(ARCH)
	@rm -rf user/.cargo
	$(call success, "user build finished")

.PHONY: user