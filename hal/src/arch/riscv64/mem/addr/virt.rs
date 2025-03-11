use core::{iter::Step, ops::{Add, Sub}};

use crate::hal::mem::{PageNumber, PageNumberHal, VirtAddr, VirtAddrHal, VirtPageNum, VirtPageNumHal};

impl VirtAddrHal for VirtAddr {
    const VA_WIDTH: usize = 39;
    
    type VirtPageNum = VirtPageNum;
    
    fn floor(&self) -> Self::VirtPageNum {
        VirtPageNum(self.0 >> PageNumber::PAGE_SIZE_BITS)
    }
    
    fn ceil(&self) -> Self::VirtPageNum {
        if self.0 == 0 {
            VirtPageNum(0)
        } else {
            VirtPageNum((self.0 + PageNumber::PAGE_SIZE - 1) >> PageNumber::PAGE_SIZE_BITS)
        }
    }
}

impl VirtPageNumHal for VirtPageNum {
    type AddrType = VirtAddr;

    type PageNumType = PageNumber;

    const LEVEL: usize = 3;

    fn index(&self, i: usize) -> usize {
        (self.0 >> (2 - i) * 3) & ((1 << 9) - 1)
    }
}
