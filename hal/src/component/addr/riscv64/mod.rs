use crate::component::constant::{Constant, ConstantsHal};

use super::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, VirtAddr, VirtAddrHal, VirtPageNum, VirtPageNumHal};

impl VirtAddrHal for VirtAddr {
    fn floor(&self) -> VirtPageNum {
        VirtPageNum((self.0 >> Constant::PAGE_SIZE_BITS) & ((1usize << Constant::VPN_WIDTH) - 1) )
    }

    fn ceil(&self) -> VirtPageNum {
        if self.0 == 0{
            VirtPageNum(0)
        } else {
            VirtPageNum(((self.0 + Constant::PAGE_SIZE - 1) >> Constant::PAGE_SIZE_BITS) & ((1usize << Constant::VPN_WIDTH) - 1))
        }
    }
}

impl VirtPageNumHal for VirtPageNum {
    fn indexes(&self) -> [usize; Constant::PG_LEVEL] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
    
    fn start_addr(&self) -> VirtAddr {
        if self.0 & (1 << (Constant::VPN_WIDTH - 1)) == 1 {
            VirtAddr((self.0 << Constant::PAGE_SIZE_BITS) | !((1usize << Constant::PA_WIDTH) - 1))
        } else {
            VirtAddr(self.0 << Constant::PAGE_SIZE_BITS)
        }
        
    }
    
    fn end_addr(&self) -> VirtAddr {
        VirtAddr(self.start_addr().0 | ((1usize << Constant::PAGE_SIZE_BITS) - 1))
    }
}

impl PhysAddrHal for PhysAddr {
    fn get_ptr<T>(&self) -> *mut T {
        (self.0 + Constant::KERNEL_ADDR_SPACE.start) as *mut T
    }

    fn floor(&self) -> PhysPageNum {
        PhysPageNum((self.0 >> Constant::PAGE_SIZE_BITS) & ((1usize << Constant::PPN_WIDTH) - 1) )
    }

    fn ceil(&self) -> PhysPageNum {
        if self.0 == 0{
            PhysPageNum(0)
        } else {
            PhysPageNum(((self.0 + Constant::PAGE_SIZE - 1) >> Constant::PAGE_SIZE_BITS) & ((1usize << Constant::PPN_WIDTH) - 1))
        }
    }
}

impl PhysPageNumHal for PhysPageNum {
    fn start_addr(&self) -> PhysAddr {
        PhysAddr(self.0 << Constant::PAGE_SIZE_BITS)
    }
    
    fn end_addr(&self) -> PhysAddr {
        PhysAddr(self.start_addr().0 | ((1usize << Constant::PAGE_SIZE_BITS) - 1))
    }
}