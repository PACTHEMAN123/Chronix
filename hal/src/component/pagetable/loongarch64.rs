use core::ops::Range;

use alloc::vec::Vec;
use loongArch64::register;

use crate::{addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, RangePPNHal, VirtAddrHal, VirtPageNum, VirtPageNumHal}, allocator::{FrameAllocatorHal, FrameAllocatorTrackerExt, DynamicFrameAllocator}, common::FrameTracker, constant::{Constant, ConstantsHal}, println};

use super::{MapPerm, PageTableEntryHal, PageTableHal};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageLevel {
    Huge = 0,
    Big = 1,
    Middle = 2,
    Small = 3
}

impl PageLevel {
    pub const fn page_count(self) -> usize {
        match self {
            PageLevel::Huge => 512 * 512 * 512,
            PageLevel::Big => 512 * 512,
            PageLevel::Middle => 512,
            PageLevel::Small => 1,
        }
    }

    pub const fn lower(self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Big,
            PageLevel::Big => PageLevel::Middle,
            PageLevel::Middle => PageLevel::Small,
            PageLevel::Small => PageLevel::Small,
        }
    }

    pub const fn higher(self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Huge,
            PageLevel::Big => PageLevel::Huge,
            PageLevel::Middle => PageLevel::Big,
            PageLevel::Small => PageLevel::Middle,
        }
    }

    pub const fn lowest(self) -> bool {
        match self {
            PageLevel::Small => true,
            _ => false
        }
    }

    pub const fn highest(self) -> bool {
        match self {
            PageLevel::Huge => true,
            _ => false
        }
    }

    pub const fn from_count(count: usize) -> Option<Self> {
        match count {
            0x1 => Some(Self::Small),
            0x200 => Some(Self::Middle),
            0x40000 => Some(Self::Big),
            0x8000000 => Some(Self::Huge),
            _ => None
        }
    }
}

impl From<usize> for PageLevel {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Huge,
            1 => Self::Big,
            2 => Self::Middle,
            3 => Self::Small,
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
            let ret = (self.range_vpn.start, PageLevel::Small);
            self.range_vpn.start += PageLevel::Small.page_count();
            Some(ret)
        }
    }
}


bitflags::bitflags! {
    /// Possible flags for a page table entry.
    pub(crate) struct PTEFlags: usize {
        /// Page Valid
        const V = 1 << 0;
        /// Dirty, The page has been writed.
        const D = 1 << 1;

        /// PLV low bit
        const PLV_L = 1 << 2;
        /// PLV hign bit
        const PLV_H = 1 << 3;

        /// MAT low bit
        const MAT_L = 1 << 4;
        /// MAT high bit
        const MAT_H = 1 << 5;

        /// Designates a global mapping OR Whether the page is huge page.
        const GH = 1 << 6;

        /// Page is existing.
        const P = 1 << 7;
        /// Page is writeable.
        const W = 1 << 8;
        /// Page is CoW
        const C = 1 << 9;
        // /// Is a Global Page if using huge page(GH bit).
        // const G = 1 << 12;
        /// Page is not readable.
        const NR = 1 << 61;
        /// Page is not executable.
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
    const FLAGS_MASK: usize = {
        PTEFlags::PLV_L.bits | PTEFlags::PLV_H.bits |
        PTEFlags::W.bits     | PTEFlags::NX.bits    |
        PTEFlags::NR.bits
    };
    const PTE_FLAGS_MASK: usize = 0xE000_0000_0000_0FFF;
    const PPN_MASK: usize = 0x1FFF_FFFF_FFFF_F000;

    pub(crate) fn pteflags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits & Self::PTE_FLAGS_MASK).unwrap()
    }

}

impl From<MapPerm> for PTEFlags {
    fn from(value: MapPerm) -> Self {
        let mut ret = Self::empty();
        if value.contains(MapPerm::U) {
            ret.insert(PTEFlags::PLV_L | PTEFlags::PLV_H);
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
        ret
    }
}

impl PageTableEntryHal for PageTableEntry {
    fn new(ppn: PhysPageNum, map_perm: super::MapPerm) -> Self {
        let pte: PTEFlags = map_perm.into();
        Self {
            bits: (ppn.0 << Constant::PAGE_SIZE_BITS) | (pte.bits as usize)
        }
    }
    
    fn flags(&self) -> super::MapPerm {
        let pte = self.pteflags();
        let mut ret = MapPerm::empty();
        if pte.contains(PTEFlags::PLV_H) & 
            pte.contains(PTEFlags::PLV_L) {
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
        ret
    }
    
    fn set_flags(&mut self, map_flags: MapPerm) {
        let pte: PTEFlags = map_flags.into();
        self.bits &= !Self::FLAGS_MASK;
        self.bits |= pte.bits as usize & Self::FLAGS_MASK;
    }
    
    fn ppn(&self) -> PhysPageNum {
        PhysPageNum((self.bits & Self::PPN_MASK) >> 12)
    }
    
    fn set_ppn(&mut self, ppn: PhysPageNum) {
        self.bits &= !Self::PPN_MASK;
        self.bits |= (ppn.0 << 12) & Self::PPN_MASK;
    }

    fn is_dirty(&self) -> bool {
        self.pteflags().contains(PTEFlags::D)
    }
    
    fn set_dirty(&mut self, val: bool) {
        if val {
            self.bits |= PTEFlags::D.bits as usize;
        } else {
            self.bits &= !(PTEFlags::D.bits as usize);
        }
    }

    fn is_valid(&self) -> bool {
        self.pteflags().contains(PTEFlags::V)
    }
    
    fn set_valid(&mut self, val: bool) {
        if val {
            self.bits |= PTEFlags::V.bits as usize
        } else {
            self.bits &= !(PTEFlags::V.bits as usize)
        }
    }

    fn is_cow(&self) -> bool {
        self.pteflags().contains(PTEFlags::C)
    }
    
    fn set_cow(&mut self, val: bool) {
        if val {
            self.bits |= PTEFlags::C.bits as usize
        } else {
            self.bits &= !(PTEFlags::C.bits as usize)
        }
    }
    
    fn is_leaf(&self) -> bool {
        false 
    }
}

/// page table structure
pub struct PageTable<A: FrameAllocatorHal + Clone = DynamicFrameAllocator> {
    /// root ppn
    pub root_ppn: PhysPageNum,
    frames: Vec<FrameTracker<A>>,
    alloc: A,
}

impl<A: FrameAllocatorHal + Clone> PageTable<A> {
    fn find_pte_create(&mut self, vpn: VirtPageNum, level: PageLevel) -> Option<&mut PageTableEntry> {
        assert!(level.lowest());
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, &idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.start_addr().get_mut::<[PageTableEntry; 512]>()[idx];
            if PageLevel::from(i) == level {
                result = Some(pte);
                break;
            }
            // don't use is_valid() because we can't set flags
            if pte.bits == 0 {
                let frame = self.alloc.alloc_tracker(1).unwrap();
                frame.range_ppn.get_slice_mut::<u8>().fill(0);
                *pte = PageTableEntry {
                    bits: (frame.range_ppn.start.0 << Constant::PAGE_SIZE_BITS) // can't set flags
                };
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
}

impl<A: FrameAllocatorHal + Clone> PageTableHal<PageTableEntry, A> for PageTable<A> {
    fn from_token(token: usize, alloc: A) -> Self {
        Self { 
            root_ppn: PhysPageNum(token >> Constant::PAGE_SIZE_BITS), 
            frames: Vec::new(), 
            alloc
        }
    }

    fn get_token(&self) -> usize {
        self.root_ppn.start_addr().0 & !Constant::KERNEL_ADDR_SPACE.start
    }

    fn translate_va(&self, va: crate::addr::VirtAddr) -> Option<crate::addr::PhysAddr> {
        let (pte, level) = self.find_pte(va.floor())?;
        if !pte.is_valid() {
            return None;
        }
        let ppn = pte.ppn();
        let level = PageLevel::from(level);
        let offset = va.0 % (level.page_count() * Constant::PAGE_SIZE);
        Some(PhysAddr(ppn.start_addr().0 + offset))
    }
    
    fn translate_vpn(&self, vpn: VirtPageNum) -> Option<crate::addr::PhysPageNum> {
        let (pte, level) = self.find_pte(vpn)?;
        if !pte.is_valid() {
            return None;
        }
        let ppn = pte.ppn();
        let level = PageLevel::from(level);
        let offset = vpn.0 % level.page_count();
        Some(PhysPageNum(ppn.0 + offset))
    }
 
    fn new_in(_asid: usize, alloc: A) -> Self {
        let frame = alloc.alloc_tracker(1).unwrap();
        frame.range_ppn.get_slice_mut::<u8>().fill(0);
        Self {
            root_ppn: frame.range_ppn.start,
            frames: alloc::vec![frame],
            alloc
        }
    }

    fn find_pte(&self, vpn: crate::addr::VirtPageNum) -> Option<(&mut PageTableEntry, usize)> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.start_addr().get_mut::<[PageTableEntry; 512]>()[*idx];
            if pte.bits == 0 {
                return None;
            }
            if i == Constant::PG_LEVEL - 1 {
                return Some((pte, i));
            }
            ppn = pte.ppn();
        }
        None
    }

    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, perm: super::MapPerm, level: PageLevel) -> Result<&mut PageTableEntry, ()>{
        if let Some(pte) = self.find_pte_create(vpn, level) {
            *pte = PageTableEntry::new(ppn, perm);
            pte.set_valid(true);
            pte.bits |= PTEFlags::MAT_L.bits; // Coherent Cached
            Ok(pte)
        } else {
            log::warn!("vpn {} has been mapped", vpn.0);
            Err(())
        }
    }

    fn unmap(&mut self, vpn: VirtPageNum) -> Result<PageTableEntry, ()> {
        match self.find_pte(vpn) {
            Some((pte, _)) => {
                let ret = *pte;
                pte.bits = 0;
                Ok(ret)
            },
            None => {
                log::warn!("vpn {} is not mapped", vpn.0);
                Err(())
            }
        }
    }

    unsafe fn enable_high(&self) {
        register::asid::set_asid(0);
        register::pgdh::set_base(self.get_token());
    }

    unsafe fn enable_low(&self) {
        register::asid::set_asid(0);
        register::pgdl::set_base(self.get_token());
    }

    fn clear(&mut self) {
        let root = self.frames.swap_remove(0);
        self.frames.clear();
        self.frames.push(root);
    }
    
    fn enabled(&self) -> bool {
        let pgdl = loongArch64::register::pgdl::read().base();
        let pgdh = loongArch64::register::pgdh::read().base();
        let token = self.get_token();
        token == pgdl || token == pgdh
    }
}
