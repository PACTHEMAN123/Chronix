mod virt;
mod phys;
mod kern;

pub use virt::*;
pub use phys::*;
pub use kern::*;

use crate::hal::mem::{PageNumber, PageNumberHal};

impl PageNumberHal for PageNumber {
    const PAGE_SIZE: usize = 4096;
}

pub const KERNEL_ADDR_OFFSET: usize = 0xFFFF_FFC0_0000_0000;