use core::ops::Range;

use alloc::vec::Vec;

use crate::{addr::{PhysPageNum, VirtPageNum}, allocator::FrameAllocatorHal, common::FrameTracker};

use super::{MapPerm, PageTableEntryHal, PageTableHal};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageLevel {
    Huge = 0,
    Big = 1,
    Small = 2
}

impl PageLevel {
    pub const fn page_count(&self) -> usize {
        match self {
            PageLevel::Huge => 512 * 512,
            PageLevel::Big => 512,
            PageLevel::Small => 1,
        }
    }

    pub const fn lower(&self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Big,
            PageLevel::Big => PageLevel::Small,
            PageLevel::Small => PageLevel::Small,
        }
    }

    pub const fn higher(&self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Huge,
            PageLevel::Big => PageLevel::Huge,
            PageLevel::Small => PageLevel::Big,
        }
    }

    pub const fn lowest(&self) -> bool {
        match self {
            PageLevel::Small => true,
            _ => false
        }
    }

    pub const fn highest(&self) -> bool {
        match self {
            PageLevel::Huge => true,
            _ => false
        }
    }
}

impl From<usize> for PageLevel {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Huge,
            1 => Self::Big,
            2 => Self::Small,
            _ => panic!("unsupport Page Level")
        }
    }
}


#[allow(missing_docs)]
pub struct VpnPageRangeIter {
    pub range_vpn: Range<VirtPageNum>
}

#[allow(missing_docs)]
impl VpnPageRangeIter {
    pub fn new(range_vpn: Range<VirtPageNum>) -> Self {
        Self { range_vpn }
    }
}

impl Iterator for VpnPageRangeIter {
    type Item = (VirtPageNum, PageLevel);

    fn next(&mut self) -> Option<Self::Item> {
        if self.range_vpn.is_empty() {
            None
        } else {
            if self.range_vpn.start.0 % PageLevel::Huge.page_count() == 0 
            && self.range_vpn.clone().count() >= PageLevel::Huge.page_count() {
                let ret = (self.range_vpn.start, PageLevel::Huge);
                self.range_vpn.start += PageLevel::Huge.page_count();
                Some(ret)
            } else if self.range_vpn.start.0 % PageLevel::Big.page_count() == 0
            && self.range_vpn.clone().count() >= PageLevel::Big.page_count() {
                let ret = (self.range_vpn.start, PageLevel::Big);
                self.range_vpn.start += PageLevel::Big.page_count();
                Some(ret)
            } else {
                let ret = (self.range_vpn.start, PageLevel::Small);
                self.range_vpn.start += PageLevel::Small.page_count();
                Some(ret)
            }
        }
    }
}


bitflags::bitflags! {
    /// Possible flags for a page table entry.
    pub struct PTEFlags: usize {
        /// Page Valid
        const V = 1 << 0;
        /// Dirty, The page has been writed.
        const D = 1 << 1;

        const PLV_USER = 0b11 << 2;

        const MAT_NOCACHE = 0b01 << 4;

        /// Designates a global mapping OR Whether the page is huge page.
        const GH = 1 << 6;

        /// Page is existing.
        const P = 1 << 7;
        /// Page is writeable.
        const W = 1 << 8;
        /// Page is CoW
        const C = 1 << 9;
        /// Is a Global Page if using huge page(GH bit).
        const G = 1 << 12;
        /// Page is not readable.
        const NR = 1 << 61;
        /// Page is not executable.
        /// FIXME: Is it just for a huge page?
        /// Linux related url: https://github.com/torvalds/linux/blob/master/arch/loongarch/include/asm/pgtable-bits.h
        const NX = 1 << 62;
        /// Whether the privilege Level is restricted. When RPLV is 0, the PTE
        /// can be accessed by any program with privilege Level highter than PLV.
        const RPLV = 1 << 63;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
#[allow(missing_docs)]
/// page table entry structure
pub struct PageTableEntry {
    pub bits: usize,
}

#[allow(missing_docs)]
impl PageTableEntry {

    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.bits >> 10 & ((1usize << 44) - 1))
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits((self.bits & ((1usize << 10) - 1)) as usize).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::NR) == PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::NX) == PTEFlags::empty()
    }
    pub fn is_leaf(&self) -> bool {
        todo!()
    }
    pub fn set_flags(&mut self, flags: PTEFlags) {
        self.bits = ((self.bits >> 10) << 10) | flags.bits() as usize;
    }
}

impl From<MapPerm> for PTEFlags {
    fn from(value: MapPerm) -> Self {
        let mut ret = Self::empty();
        if value.contains(MapPerm::U) {
            ret.insert(PTEFlags::PLV_USER);
        }
        if !value.contains(MapPerm::R) {
            ret.insert(PTEFlags::NR);
        }
        if value.contains(MapPerm::W) {
            ret.insert(PTEFlags::W);
        }
        if !value.contains(MapPerm::X) {
            ret.insert(PTEFlags::NX);
        }
        if value.contains(MapPerm::C) {
            ret.insert(PTEFlags::C);
        }
        ret
    }
}

impl PageTableEntryHal for PageTableEntry {
    fn new(ppn: PhysPageNum, map_perm: super::MapPerm, valid: bool) -> Self {
        let mut pte: PTEFlags = map_perm.into();
        if valid {
            pte.insert(PTEFlags::V);
        }
        Self {
            bits: ppn.0 << 10 | pte.bits as usize
        }
    }

    fn set_valid(&mut self) {
        self.bits |= PTEFlags::V.bits as usize;
    }

    fn is_valid(&self) -> bool {
        self.bits & PTEFlags::V.bits as usize != 0
    }
    
    fn map_perm(&self) -> super::MapPerm {
        let pte = self.flags();
        let mut ret = MapPerm::empty();
        if pte.contains(PTEFlags::PLV_USER) {
            ret.insert(MapPerm::U);
        }
        if !pte.contains(PTEFlags::NR) {
            ret.insert(MapPerm::R);
        }
        if pte.contains(PTEFlags::W) {
            ret.insert(MapPerm::W);
        }
        if !pte.contains(PTEFlags::NX) {
            ret.insert(MapPerm::X);
        }
        if pte.contains(PTEFlags::C) {
            ret.insert(MapPerm::C);
        }
        ret
    }
}

/// page table structure
pub struct PageTable<A: FrameAllocatorHal> {
    /// root ppn
    pub root_ppn: PhysPageNum,
    frames: Vec<FrameTracker<A>>,
    alloc: A,
}

impl<A: FrameAllocatorHal> PageTableHal<PageTableEntry, A> for PageTable<A> {
    fn from_token(token: usize, alloc: A) -> Self {
        todo!()
    }

    fn get_token(&self) -> usize {
        todo!()
    }

    fn translate_va(&self, va: crate::addr::VirtAddr) -> Option<crate::addr::PhysAddr> {
        todo!()
    }

    fn translate_vpn(&self, vpn: crate::addr::VirtPageNum) -> Option<PhysPageNum> {
        todo!()
    }

    fn new_in(asid: usize, alloc: A) -> Self {
        todo!()
    }

    fn find_pte(&self, vpn: crate::addr::VirtPageNum) -> Option<(&mut PageTableEntry, usize)> {
        todo!()
    }

    fn map(&mut self, vpn: crate::addr::VirtPageNum, ppn: PhysPageNum, perm: super::MapPerm, level: PageLevel) {
        todo!()
    }

    fn unmap(&mut self, vpn: crate::addr::VirtPageNum) {
        todo!()
    }

    unsafe fn enable(&self) {
        todo!()
    }
}
