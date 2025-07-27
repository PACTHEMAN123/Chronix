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

ifeq ($(AUTOTEST),y)
KERNEL_FEATURES += autotest
endif

# kernel target
ifeq ($(ARCH), riscv64)
KERNEL_TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), loongarch64)
KERNEL_TARGET := loongarch64-unknown-none
endif
KERNEL_TARGET_ARG := --target $(KERNEL_TARGET)

# kernel build mode
# default using release to speed up
KERNEL_MODE := release
ifeq ($(KERNEL_MODE), debug)
	KERNEL_MODE_ARG := 
else
	KERNEL_MODE_ARG := --release
endif


# kernel entry
ifeq ($(ARCH), riscv64)
KERNEL_ENTRY_PA := 0x80200000
else ifeq ($(ARCH), loongarch64)
KERNEL_ENTRY_PA := 0x1c000000
endif

KERNEL_ELF := ./target/$(KERNEL_TARGET)/$(KERNEL_MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := $(KERNEL_ELF).asm

# kernel in binary
kernel-bin: kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $(KERNEL_BIN)

# kernel in elf
kernel:
	$(call building, "Architecture: $(ARCH)")
	$(call building, "Platform: $(BOARD)")
	@cp os/src/linker-$(ARCH)-$(BOARD).ld os/src/linker.ld
	@rm -rf os/.cargo
	@rm -rf hal/.cargo
	@cp -r os/cargo os/.cargo
	@cp -r hal/cargo hal/.cargo
	$(call building, "Kernel features: $(KERNEL_FEATURES)")
ifeq ($(KERNEL_FEATURES), ) 
	@cd os && cargo build $(KERNEL_TARGET_ARG) $(KERNEL_MODE_ARG)
else
	@cd os && cargo build $(KERNEL_TARGET_ARG) $(KERNEL_MODE_ARG) --features "$(KERNEL_FEATURES)"
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

.PHONY: kernel disasm disasm-vim fmt kernel-bin