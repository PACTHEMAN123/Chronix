use super::{Constant, ConstantsHal};

impl ConstantsHal for Constant {
    const KERNEL_ENTRY_PA: usize = 0x9000_0000;

    const KERNEL_ADDR_SPACE: core::ops::Range<usize> = 0x9000_0000_0000_0000..0x9001_0000_0000_0000;

    const USER_ADDR_SPACE: core::ops::Range<usize> = 0x0000_0000_0000_0000..0x0000_8000_0000_0000;

    const PA_WIDTH: usize = 48;

    const VA_WIDTH: usize = 48;

    const PAGE_SIZE: usize = 4096;

    const PAGE_SIZE_BITS: usize = 12;

    const PG_LEVEL: usize = 4;
    
    const PTE_WIDTH: usize = 64;
    
    const MEMORY_END: usize = 0x9000_0000_A000_0000;

    const SIGRET_TRAMPOLINE_SIZE: usize = Self::PAGE_SIZE;
    const SIGRET_TRAMPOLINE_TOP: usize = 0x0000_ffff_ffff_f000;
    
    const KERNEL_STACK_SIZE: usize = 16 * 4096;
    const KERNEL_STACK_TOP: usize = Self::KERNEL_ADDR_SPACE.end;

    const KERNEL_VM_SIZE: usize = 0x2_0000_0000;
    const KERNEL_VM_TOP: usize = 0x0000_ffff_ffff_ffff;
    const KERNEL_VM_BOTTOM: usize = Self::KERNEL_VM_TOP - Self::KERNEL_VM_SIZE + 1;

    const USER_STACK_SIZE: usize = 4096 * 4096;
    
    const USER_STACK_TOP: usize = Self::USER_ADDR_SPACE.end - Self::PAGE_SIZE;
    
    // put the file mmap area under user stack
    const USER_FILE_END: usize = Self::USER_STACK_BOTTOM;
    const USER_FILE_SIZE: usize = 0x2_0000_0000;

    // put the share mmap area under file mmap area
    const USER_SHARE_END: usize = Self::USER_FILE_BEG;
    const USER_SHARE_SIZE: usize = 0x2_0000_0000;
    
    const DL_INTERP_OFFSET: usize = 0x20_0000_0000;
}