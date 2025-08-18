use core::ops::Range;

use crate::component::constant::{Constant, ConstantsHal};

use super::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, RangePPNHal, VirtAddr, VirtAddrHal, VirtPageNum, VirtPageNumHal};

impl VirtAddrHal for VirtAddr {
    fn floor(&self) -> VirtPageNum {
        VirtPageNum((self.0 >> Constant::PAGE_SIZE_BITS) & ((1usize << Constant::VPN_WIDTH) - 1) )
    }

    fn ceil(&self) -> VirtPageNum {
        if self.0 == 0 {
            VirtPageNum(0)
        } else {
            VirtPageNum(((self.0 - 1 + Constant::PAGE_SIZE) >> Constant::PAGE_SIZE_BITS) & ((1usize << Constant::VPN_WIDTH) - 1))
        }
    }
}

impl VirtPageNumHal for VirtPageNum {
    fn indexes(&self) -> [usize; Constant::PG_LEVEL] {
        let mut vpn = self.0;
        let mut idx = [0usize; Constant::PG_LEVEL];
        for i in (0..Constant::PG_LEVEL).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
    
    fn start_addr(&self) -> VirtAddr {
        VirtAddr::from(self.0 << Constant::PAGE_SIZE_BITS)
    }
    
    fn end_addr(&self) -> VirtAddr {
        VirtAddr(self.start_addr().0 + Constant::PAGE_SIZE)
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
            PhysPageNum(((self.0 - 1 + Constant::PAGE_SIZE) >> Constant::PAGE_SIZE_BITS) & ((1usize << Constant::PPN_WIDTH) - 1))
        }
    }
}

impl PhysPageNumHal for PhysPageNum {
    fn start_addr(&self) -> PhysAddr {
        PhysAddr(self.0 << Constant::PAGE_SIZE_BITS)
    }
    
    fn end_addr(&self) -> PhysAddr {
        PhysAddr(self.start_addr().0 + Constant::PAGE_SIZE)
    }
}

impl RangePPNHal for Range<PhysPageNum> {
    fn get_slice<T>(&self) -> &[T] {
        unsafe { 
            core::slice::from_raw_parts(self.start.start_addr().get_ptr(), 
            self.clone().count() * Constant::PAGE_SIZE / core::mem::size_of::<T>()) 
        }
    }

    fn get_slice_mut<T>(&self) -> &mut [T] {
        unsafe { 
            core::slice::from_raw_parts_mut(self.start.start_addr().get_ptr(), 
            self.clone().count() * Constant::PAGE_SIZE / core::mem::size_of::<T>()) 
        }
    }
} 