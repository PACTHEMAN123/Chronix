use core::ops::Range;

pub trait ConstantsHal {
    const KERNEL_ENTRY_PA: usize;

    const KERNEL_ADDR_SPACE: Range<usize>;
    const USER_ADDR_SPACE: Range<usize>;

    const PA_WIDTH: usize;
    const VA_WIDTH: usize;

    const PAGE_SIZE: usize;
    const PAGE_SIZE_BITS: usize;

    const PTE_WIDTH: usize;
    const PTES_PER_PAGE: usize = Self::PAGE_SIZE / (Self::PTE_WIDTH >> 3);
    const PPN_WIDTH: usize = Self::PA_WIDTH - Self::PAGE_SIZE_BITS;
    const VPN_WIDTH: usize = Self::VA_WIDTH - Self::PAGE_SIZE_BITS;

    const PG_LEVEL: usize;

    const MEMORY_END: usize;

    const MMIO: &[(usize, usize)];

    const KERNEL_STACK_SIZE: usize;
    const KERNEL_STACK_BOTTOM: usize = Self::KERNEL_STACK_TOP - Self::KERNEL_STACK_SIZE + 1;
    const KERNEL_STACK_TOP: usize;

    const USER_STACK_SIZE: usize;
    const USER_STACK_BOTTOM: usize = Self::USER_STACK_TOP - Self::USER_STACK_SIZE;
    const USER_STACK_TOP: usize;

    const USER_TRAP_CONTEXT_SIZE: usize;
    const USER_TRAP_CONTEXT_TOP: usize;
    const USER_TRAP_CONTEXT_BOTTOM: usize = Self::USER_TRAP_CONTEXT_TOP - Self::USER_TRAP_CONTEXT_SIZE + 1;

}

pub struct Constant;

#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
#[allow(unused)]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
#[allow(unused)]
pub use loongarch64::*;