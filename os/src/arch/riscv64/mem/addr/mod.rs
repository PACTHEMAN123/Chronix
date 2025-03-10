mod virt;
mod phys;
mod kern;

pub use virt::*;
pub use phys::*;
pub use kern::*;

use hal::mem::PageNumberHal;
pub struct PageNum;

impl PageNumberHal for PageNum {
    const PAGE_SIZE: usize = 4096;
}

pub const KERNEL_ADDR_OFFSET: usize = 0xFFFF_FFC0_0000_0000;