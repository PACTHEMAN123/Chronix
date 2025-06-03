use crate::allocator::FrameAllocatorHal;
use crate::addr::{PhysPageNum, VirtPageNum, PhysAddr, VirtAddr};

use bitflags::bitflags;

bitflags! {
    pub struct MapPerm: u8 {
        /// Readable
        const R = 1 << 0;
        /// Writable
        const W = 1 << 1;
        /// Executable
        const X = 1 << 2;
        /// User-mode accessible
        const U = 1 << 3;
    }
}

pub trait PageTableEntryHal {
    fn new(ppn: PhysPageNum, map_flags: MapPerm) -> Self;

    fn flags(&self) -> MapPerm;

    fn set_flags(&mut self, map_flags: MapPerm);

    fn ppn(&self) -> PhysPageNum;

    fn set_ppn(&mut self, ppn: PhysPageNum);

    fn is_valid(&self) -> bool;
    
    fn set_valid(&mut self, val: bool);

    fn is_user(&self) -> bool {
        self.flags().contains(MapPerm::U)
    }

    fn set_user(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapPerm::U);
        } else {
            self.set_flags(self.flags() & !MapPerm::U);
        }
    }

    fn is_readable(&self) -> bool {
        self.flags().contains(MapPerm::R)
    }

    fn set_readable(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapPerm::R);
        } else {
            self.set_flags(self.flags() & !MapPerm::R);
        }
    }

    fn is_writable(&self) -> bool {
        self.flags().contains(MapPerm::W)
    }

    fn set_writable(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapPerm::W);
        } else {
            self.set_flags(self.flags() & !MapPerm::W);
        }
    }

    fn is_executable(&self) -> bool {
        self.flags().contains(MapPerm::X)
    }

    fn set_executable(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapPerm::X);
        } else {
            self.set_flags(self.flags() & !MapPerm::X);
        }
    }

    fn is_cow(&self) -> bool;

    fn set_cow(&mut self, val: bool);

    fn is_dirty(&self) -> bool;

    fn set_dirty(&mut self, val: bool);
    
    fn is_leaf(&self) -> bool;
}

pub trait PageTableHal<PTE: PageTableEntryHal, A: FrameAllocatorHal> {
    fn from_token(token: usize, alloc: A) -> Self;
    fn get_token(&self) -> usize;
    fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr>;
    fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PhysPageNum>;
    fn new_in(asid: usize, alloc: A) -> Self;
    fn find_pte(&self, vpn: VirtPageNum) -> Option<(&mut PTE, usize)>;
    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, perm: MapPerm, level: PageLevel) -> Result<&mut PTE, ()>;
    fn unmap(&mut self, vpn: VirtPageNum) -> Result<PTE, ()>;
    fn clear(&mut self);
    unsafe fn enable_high(&self);
    unsafe fn enable_low(&self);
}

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

