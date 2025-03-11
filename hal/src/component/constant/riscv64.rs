use super::{Constant, ConstantsHal};

impl ConstantsHal for Constant {
    const KERNEL_ADDR_SPACE: core::ops::Range<usize> = 0xffff_ffc0_0000_0000..0xffff_ffff_ffff_ffff;

    const USER_ADDR_SPACE: core::ops::Range<usize> = 0x0000_0000_0000_0000..0x0000_003f_ffff_ffff;

    const PA_WIDTH: usize = 39;

    const VA_WIDTH: usize = 44;

    const PAGE_SIZE: usize = 4096;

    const PG_LEVEL: usize = 3;
}