ARCH := loongarch64

ifeq ($(ARCH), riscv64)
TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), loongarch64)
TARGET := loongarch64-unknown-none
endif

MODE := release
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

MODE_ARG += --target $(TARGET)

APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := rust-objdump --arch-name=$(ARCH)
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

elf: $(APPS)
	CARGO_TARGET_DIR=target cargo build $(MODE_ARG)

binary: elf
	@$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)

build: binary

clean:
	@cargo clean

.PHONY: elf binary build clean
