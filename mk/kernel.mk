########################################################
# KERNEL (for local debug)
########################################################

# features
KERNEL_FEATURES := 

ifeq ($(FS), fat32)
KERNEL_FEATURES += fat32
endif

ifneq ($(SMP),)
KERNEL_FEATURES += smp
endif

ifeq ($(NET_C),y)
KERNEL_FEATURES += net
endif

# kernel entry
ifeq ($(ARCH), riscv64)
KERNEL_ENTRY_PA := 0x80200000
else ifeq ($(ARCH), loongarch64)
KERNEL_ENTRY_PA := 0x1c000000
endif

KERNEL_ELF := os/target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := $(KERNEL_ELF).asm

# kernel in binary
$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

# kernel in elf
kernel: dumpdtb
	$(call building, "Architecture: $(ARCH)")
	$(call building, "Platform: $(BOARD)")
	@cp os/src/linker-$(ARCH)-$(BOARD).ld os/src/linker.ld
	@rm -rf os/.cargo
	@rm -rf hal/.cargo
	@cp -r os/cargo-config os/.cargo
	@cp -r hal/cargo-config hal/.cargo
ifeq ($(KERNEL_FEATURES), ) 
	@cd os && CARGO_TARGET_DIR=target cargo build $(MODE_ARG)
else
	@cd os && CARGO_TARGET_DIR=target cargo build $(MODE_ARG) --features "$(KERNEL_FEATURES)"
endif
	@rm os/src/linker.ld
	$(call success, "kernel $(KERNEL_ELF) finish building")

# Disassembly
DISASM ?= -x
disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

# code format
fmt:
	cd os ; cargo fmt;  cd ..

.PHONY: kernel disasm disasm-vim fmt $(KERNEL_BIN)