use core::{arch::asm, ops::Range};

use alloc::vec::Vec;
use bitflags::bitflags;

use crate::{addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, RangePPNHal, VirtAddrHal, VirtPageNum, VirtPageNumHal}, allocator::{DynamicFrameAllocator, FrameAllocatorHal}, common::FrameTracker, constant::{Constant, ConstantsHal}};

use super::{MapFlags, PageTableEntryHal, PageTableHal};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageLevel {
    Huge = 0,
    Big = 1,
    Small = 2
}

impl PageLevel {
    pub const fn page_count(self) -> usize {
        match self {
            PageLevel::Huge => 512 * 512,
            PageLevel::Big => 512,
            PageLevel::Small => 1,
        }
    }

    pub const fn lower(self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Big,
            PageLevel::Big => PageLevel::Small,
            PageLevel::Small => PageLevel::Small,
        }
    }

    pub const fn higher(self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Huge,
            PageLevel::Big => PageLevel::Huge,
            PageLevel::Small => PageLevel::Big,
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
            0x200 => Some(Self::Big),
            0x40000 => Some(Self::Huge),
            _ => None
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
            let ret = (self.range_vpn.start, PageLevel::Small);
            self.range_vpn.start += PageLevel::Small.page_count();
            Some(ret)
        }
    }
}


bitflags! {
    /// page table entry flags
    pub(crate) struct PTEFlags: u16 {
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
        /// 
        const G = 1 << 5;
        /// Accessed
        const A = 1 << 6;
        /// Dirty
        const D = 1 << 7;
        /// Copy On Write
        const C = 1 << 8;
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
    #[allow(unused)]
    const RESERVE_MASK: usize = 0xFFC0_0000_0000_0000;
    const PPN_MASK: usize = 0x003F_FFFF_FFFF_FC00;
    const PTE_FLAGS_MASK: usize = 0x0000_0000_0000_03FF;
    const FLAGS_MASK: usize = {
        PTEFlags::U.bits | PTEFlags::R.bits |
        PTEFlags::W.bits | PTEFlags::X.bits
    } as usize;

    pub(crate) fn pteflags(&self) -> PTEFlags {
        PTEFlags::from_bits((self.bits & Self::PTE_FLAGS_MASK) as u16).unwrap()
    }
}

impl From<MapFlags> for PTEFlags {
    fn from(value: MapFlags) -> Self {
        let mut ret = Self::empty();
        if value.contains(MapFlags::U) {
            ret.insert(PTEFlags::U);
        }
        if value.contains(MapFlags::R) {
            ret.insert(PTEFlags::R);
        }
        if value.contains(MapFlags::W) {
            ret.insert(PTEFlags::W);
        }
        if value.contains(MapFlags::X) {
            ret.insert(PTEFlags::X);
        }
        ret
    }
}

impl PageTableEntryHal for PageTableEntry {
    fn new(ppn: PhysPageNum, map_flags: super::MapFlags) -> Self {
        let pte: PTEFlags = map_flags.into();
        Self {
            bits: (ppn.0 << 10) | pte.bits as usize
        }
    }
    
    fn flags(&self) -> super::MapFlags {
        let pte = self.pteflags();
        let mut ret = MapFlags::empty();
        if pte.contains(PTEFlags::U) {
            ret.insert(MapFlags::U);
        }
        if pte.contains(PTEFlags::R) {
            ret.insert(MapFlags::R);
        }
        if pte.contains(PTEFlags::W) {
            ret.insert(MapFlags::W);
        }
        if pte.contains(PTEFlags::X) {
            ret.insert(MapFlags::X);
        }
        ret
    }
    
    fn set_flags(&mut self, map_flags: MapFlags) {
        let pte: PTEFlags = map_flags.into();
        self.bits &= !Self::FLAGS_MASK;
        self.bits |= pte.bits as usize & Self::FLAGS_MASK;
    }
    
    fn ppn(&self) -> PhysPageNum {
        PhysPageNum((self.bits >> 10) & ((1usize << Constant::PPN_WIDTH) - 1))
    }
    
    fn set_ppn(&mut self, ppn: PhysPageNum) {
        self.bits &= !Self::PPN_MASK;
        self.bits |= (ppn.0 << 10) & Self::PPN_MASK;
    }

    fn is_leaf(&self) -> bool {
        self.pteflags().contains(PTEFlags::V) && 
        (
            self.pteflags().contains(PTEFlags::R) ||
            self.pteflags().contains(PTEFlags::W) ||
            self.pteflags().contains(PTEFlags::X)
        )
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
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, &idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.start_addr().get_mut::<[PageTableEntry; 512]>()[idx];
            if PageLevel::from(i) == level {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = self.alloc.alloc(1).unwrap();
                frame.get_slice_mut::<u8>().fill(0);
                *pte = PageTableEntry::new(frame.start, MapFlags::empty());
                pte.set_valid(true);
                self.frames.push(FrameTracker::new_in(frame, self.alloc.clone()));
            }
            ppn = pte.ppn();
        }
        result
    }

}

impl<A: FrameAllocatorHal + Clone> PageTableHal<PageTableEntry, A> for PageTable<A> {

    fn from_token(token: usize, alloc: A) -> Self {
        Self {
            root_ppn: PhysPageNum(token & ((1 << Constant::PPN_WIDTH) - 1)), 
            frames: Vec::new(),
            alloc
        }
    }

    fn get_token(&self) -> usize {
        (8usize << 60) | self.root_ppn.0
    }

    fn new_in(_: usize, alloc: A) -> Self {
        let frame = alloc.alloc(1).unwrap();
        frame.get_slice_mut::<u8>().fill(0);
        Self {
            root_ppn: frame.start,
            frames: Vec::new(),
            alloc
        }
    }

    fn find_pte(&self, vpn: crate::addr::VirtPageNum) -> Option<(&mut PageTableEntry, usize)> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.start_addr().get_mut::<[PageTableEntry; 512]>()[*idx];
            if !pte.is_valid() {
                return None;
            }
            if pte.is_leaf() || i == 2 {
                return Some((pte, i));
            }
            ppn = pte.ppn();
        }
        None
    }

    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, perm: super::MapFlags, level: PageLevel) -> Result<&mut PageTableEntry, ()>{
        if let Some(pte) = self.find_pte_create(vpn, level) {
            *pte = PageTableEntry::new(ppn, perm);
            pte.set_valid(true);
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
        asm!("csrw satp, {}", in(reg)(self.get_token()), options(nostack));
    }

    unsafe fn enable_low(&self) {
        asm!("csrw satp, {}", in(reg)(self.get_token()), options(nostack));
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
    

}
