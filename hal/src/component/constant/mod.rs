use core::ops::Range;

pub trait ConstantsHal {
    const KERNEL_ADDR_SPACE: Range<usize>;
    const USER_ADDR_SPACE: Range<usize>;

    const PA_WIDTH: usize;
    const VA_WIDTH: usize;

    const PAGE_SIZE: usize;
    const PAGE_SIZE_BITS: usize = {
        let mut i = 63;
        loop {
            if Self::PAGE_SIZE & (1usize << i) == 1 {
                break i + 1;
            } else if i == 0 {
                break 0;
            } else {
                i -= 1;
            }
        }
    };

    const PTE_WIDTH: usize;
    const PTES_PER_PAGE: usize = Self::PAGE_SIZE / (Self::PTE_WIDTH >> 3);
    const PPN_WIDTH: usize = Self::PA_WIDTH - Self::PAGE_SIZE_BITS;
    const VPN_WIDTH: usize = Self::VA_WIDTH - Self::PAGE_SIZE_BITS;

    const PG_LEVEL: usize;

}

pub struct Constant;

impl Constant {
    pub const KERNEL_STACK_SIZE: usize = 65536;
}


#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;