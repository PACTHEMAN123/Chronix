use core::ops::Range;

use bitflags::bitflags;

use super::{addr::{PhysAddrHal, PhysPageNumHal, VirtAddrHal, VirtPageNumHal}, FrameAllocatorHal};

pub trait VmSpaceHal {
    const KERNEL_SPACE_RANGE: Range<usize>;
    const USER_SPACE_RANGE: Range<usize>;
    const KERNEL_ADDR_OFFSET: usize;
    const KERNEL_STACK_SIZE: usize;
    const KERNEL_STACK_BOTTOM: usize;
    const KERNEL_STACK_TOP: usize = Self::KERNEL_STACK_BOTTOM + Self::KERNEL_STACK_SIZE;

    type PageTable: PageTableHal;

    fn get_page_table(&self) -> Self::PageTable;
}

pub trait PageLevelHal: From<usize> + Clone + Copy + PartialEq + Eq {
    fn lower(&self) -> Self;
    fn upper(&self) -> Self;
    fn page_cnt(&self) -> usize;
    const LOWEST: Self;
    const HIGHEST: Self;
}

pub trait PageTableEntryHal {
    type PhysPageNum: PhysPageNumHal;

    fn new(ppn: Self::PhysPageNum, perm: MapPerm) -> Self;
    fn bits(&self) -> usize;
    fn ppn(&self) -> Self::PhysPageNum;
    fn is_valid(&self) -> bool;
    fn perm(&self) -> MapPerm;
    fn set_perm(&mut self, perm: MapPerm);
}

bitflags! {
    /// map permission corresponding to that in pte: `R W X U C`
    pub struct MapPerm: u8 {
        /// Readable
        const R = 1 << 0;
        /// Writable
        const W = 1 << 1;
        /// Executable
        const X = 1 << 2;
        /// User-mode accessible
        const U = 1 << 3;
        /// Copy On Write
        const C = 1 << 4;

        /// Read-write
        const RW = Self::R.bits() | Self::W.bits();
        /// Read-execute
        const RX = Self::R.bits() | Self::X.bits();
        /// Reserved
        const WX = Self::W.bits() | Self::X.bits();
        /// Read-write-execute
        const RWX = Self::R.bits() | Self::W.bits() | Self::X.bits();

        /// User Write-only
        const UW = Self::U.bits() | Self::W.bits();
        /// User Read-write
        const URW = Self::U.bits() | Self::RW.bits();
        /// Uer Read-execute
        const URX = Self::U.bits() | Self::RX.bits();
        /// Reserved
        const UWX = Self::U.bits() | Self::WX.bits();
        /// User Read-write-execute
        const URWX = Self::U.bits() | Self::RWX.bits();
        
        /// Read freely, copy on write
        const RC = Self::R.bits() | Self::C.bits();
        /// User Read-COW
        const URC = Self::U.bits() | Self::RC.bits();
    }
}

pub trait PageTableHal {
    type VirtAddr: VirtAddrHal;
    type VirtPageNum: VirtPageNumHal;
    type PhysAddr: PhysAddrHal;
    type PhysPageNum: PhysPageNumHal;
    type PageLevel: PageLevelHal;
    type PageTableEntry: PageTableEntryHal;
    type FrameAllocator: FrameAllocatorHal<PhysPageNum = Self::PhysPageNum>;

    fn new(ppn: Self::PhysPageNum, asid: usize, alloc: Self::FrameAllocator) -> Self;
    fn get_token(&self) -> usize;
    fn translate_vpn(&self, vpn: Self::VirtPageNum) -> Self::PhysPageNum;
    fn translate_va(&self, va: Self::VirtAddr) -> Self::PhysAddr;
    fn find_pte(&self, vpn: Self::VirtPageNum) -> Option<(&mut Self::PageTableEntry, Self::PageLevel)>;
    fn map(&mut self, vpn: Self::VirtPageNum, ppn: Self::PhysPageNum, perm: MapPerm, level: Self::PageLevel);
    fn unmap(&mut self, range_vpn: Range<Self::VirtPageNum>);

    unsafe fn enable(&self);
}
