use super::{Constant, ConstantsHal};

impl ConstantsHal for Constant {
    const KERNEL_ENTRY_PA: usize = 0x9000_0000;

    const KERNEL_ADDR_SPACE: core::ops::Range<usize> = 0x9000_0000_0000_0000..0x9000_ffff_ffff_ffff;

    const USER_ADDR_SPACE: core::ops::Range<usize> = 0x0000_0000_0000_0000..0x0000_7fff_ffff_ffff;

    const PA_WIDTH: usize = 48;

    const VA_WIDTH: usize = 48;

    const PAGE_SIZE: usize = 4096;

    const PAGE_SIZE_BITS: usize = 12;

    const PG_LEVEL: usize = 4;
    
    const PTE_WIDTH: usize = 64;
    
    const MEMORY_END: usize = 0x9000_0000_A000_0000;
    
    const KERNEL_STACK_SIZE: usize = 16 * 4096;
    
    const KERNEL_STACK_TOP: usize = Self::KERNEL_ADDR_SPACE.end;
    
    const USER_STACK_SIZE: usize = 16 * 4096;
    
    const USER_STACK_TOP: usize = Self::USER_TRAP_CONTEXT_BOTTOM;
    
    const USER_TRAP_CONTEXT_SIZE: usize = Self::PAGE_SIZE;
    
    const USER_TRAP_CONTEXT_TOP: usize = Self::USER_ADDR_SPACE.end;
}