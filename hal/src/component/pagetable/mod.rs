use crate::allocator::FrameAllocatorHal;
use crate::addr::{PhysPageNum, VirtPageNum, PhysAddr, VirtAddr};

use bitflags::bitflags;

bitflags! {
    pub struct MapFlags: u8 {
        /// Valid
        const V = 1 << 0;
        /// Readable
        const R = 1 << 1;
        /// Writable
        const W = 1 << 2;
        /// Executable
        const X = 1 << 3;
        /// User-mode accessible
        const U = 1 << 4;
        /// Copy On Write
        const C = 1 << 5;
        /// Dirty
        const D = 1 << 6;
    }
}

pub trait PageTableEntryHal {
    fn new(ppn: PhysPageNum, map_flags: MapFlags) -> Self;

    fn flags(&self) -> MapFlags;

    fn set_flags(&mut self, map_flags: MapFlags);

    fn ppn(&self) -> PhysPageNum;

    fn set_ppn(&mut self, ppn: PhysPageNum);

    fn is_valid(&self) -> bool {
        self.flags().contains(MapFlags::V)
    }
    
    fn set_valid(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapFlags::V);
        } else {
            self.set_flags(self.flags() & !MapFlags::V);
        }
    }

    fn is_user(&self) -> bool {
        self.flags().contains(MapFlags::U)
    }

    fn set_user(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapFlags::U);
        } else {
            self.set_flags(self.flags() & !MapFlags::U);
        }
    }

    fn is_readable(&self) -> bool {
        self.flags().contains(MapFlags::R)
    }

    fn set_readable(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapFlags::R);
        } else {
            self.set_flags(self.flags() & !MapFlags::R);
        }
    }

    fn is_writable(&self) -> bool {
        self.flags().contains(MapFlags::W)
    }

    fn set_writable(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapFlags::W);
        } else {
            self.set_flags(self.flags() & !MapFlags::W);
        }
    }

    fn is_executable(&self) -> bool {
        self.flags().contains(MapFlags::X)
    }

    fn set_executable(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapFlags::X);
        } else {
            self.set_flags(self.flags() & !MapFlags::X);
        }
    }

    fn is_cow(&self) -> bool {
        self.flags().contains(MapFlags::C)
    }

    fn set_cow(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapFlags::C);
        } else {
            self.set_flags(self.flags() & !MapFlags::C);
        }
    }

    fn is_dirty(&self) -> bool {
        self.flags().contains(MapFlags::D)
    }

    fn set_dirty(&mut self, val: bool) {
        if val {
            self.set_flags(self.flags() | MapFlags::D);
        } else {
            self.set_flags(self.flags() & !MapFlags::D);
        }
    }
    
    fn is_leaf(&self) -> bool;
}

pub trait PageTableHal<PTE: PageTableEntryHal, A: FrameAllocatorHal> {
    fn from_token(token: usize, alloc: A) -> Self;
    fn get_token(&self) -> usize;
    fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr>;
    fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PhysPageNum>;
    fn new_in(asid: usize, alloc: A) -> Self;
    fn find_pte(&self, vpn: VirtPageNum) -> Option<(&mut PTE, usize)>;
    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, perm: MapFlags, level: PageLevel) -> Result<&mut PTE, ()>;
    fn unmap(&mut self, vpn: VirtPageNum) -> Result<PTE, ()>;
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

