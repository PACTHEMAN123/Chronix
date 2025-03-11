use crate::hal::mem::{PageNumberHal, KernAddr, KernPageNum, PageNumber, PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal};

use super::KERNEL_ADDR_OFFSET;

impl PhysAddrHal for PhysAddr {
    const PA_WIDTH: usize = 44;

    type KernAddr = KernAddr;

    fn to_kern(&self) -> Self::KernAddr {
        KernAddr(self.0 + KERNEL_ADDR_OFFSET)
    }
}

impl PhysPageNumHal for PhysPageNum {
    type AddrType = PhysAddr;

    type PageNumType = PageNumber;

    type KernPageNum = KernPageNum;

    fn to_kern(&self) -> Self::KernPageNum {
        KernPageNum(self.0 + (KERNEL_ADDR_OFFSET >> Self::PageNumType::PAGE_SIZE_BITS))
    }
}

