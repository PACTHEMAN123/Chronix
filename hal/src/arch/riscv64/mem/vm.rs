use crate::hal::mem::{FrameAllocatorHal, FrameTracker, KernPageNumHal, MapPerm, PageLevelHal, PageTableEntryHal, PageTableHal, PhysAddr, PhysPageNum, PhysPageNumHal, VirtAddr, VirtPageNum, VirtPageNumHal};
use alloc::vec::Vec;

#[allow(unused, missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLevel {
    Huge,
    Big,
    Small
}

#[allow(unused, missing_docs)]
impl PageLevelHal for PageLevel {
    fn lower(&self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Big,
            PageLevel::Big => PageLevel::Small,
            PageLevel::Small => PageLevel::Small,
        }
    }

    fn upper(&self) -> Self {
        match self {
            PageLevel::Huge => PageLevel::Huge,
            PageLevel::Big => PageLevel::Huge,
            PageLevel::Small => PageLevel::Big,
        }
    }

    const LOWEST: Self = Self::Small;

    const HIGHEST: Self = Self::Huge;
    
    fn page_cnt(&self) -> usize {
        match self {
            PageLevel::Huge => 512 * 512,
            PageLevel::Big => 512,
            PageLevel::Small => 1,
        }
    }
}

impl From<usize> for PageLevel {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Huge,
            1 => Self::Big,
            2 => Self::Small,
            _ => panic!("unsupported page level")
        }
    }
}
#[allow(unused, missing_docs)]
pub struct PageTableEntry {
    bits: usize
}

#[allow(unused, missing_docs)]
impl PageTableEntryHal for PageTableEntry {
    type PhysPageNum = PhysPageNum;

    fn new(ppn: Self::PhysPageNum, perm: MapPerm) -> Self {
        todo!()
    }

    fn perm(&self) -> MapPerm {
        todo!()
    }

    fn set_perm(&mut self, perm: MapPerm) {
        todo!()
    }
    
    fn bits(&self) -> usize {
        self.bits
    }
    
    fn ppn(&self) -> Self::PhysPageNum {
        todo!()
    }
    
    fn is_valid(&self) -> bool {
        todo!()
    }
}

#[allow(unused, missing_docs)]
pub struct PageTable<A: FrameAllocatorHal<PhysPageNum = PhysPageNum>> {
    pub root_ppn: PhysPageNum,
    frames: Vec<FrameTracker<PhysPageNum, A>>,
    alloc: A
}

#[allow(unused, missing_docs)]
impl<A> PageTable<A>
    where A: FrameAllocatorHal<PhysPageNum = PhysPageNum>
{
    fn find_pte_create(&mut self, vpn: <Self as PageTableHal>::VirtPageNum, level: <Self as PageTableHal>::PageLevel) -> Option<&mut <Self as PageTableHal>::PageTableEntry> {
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in 
            (0..<<Self as PageTableHal>::VirtPageNum as VirtPageNumHal>::LEVEL)
            .map(|i| (i, vpn.index(i)))
        {
            let pte = &mut ppn.to_kern().get_mut::<[<Self as PageTableHal>::PageTableEntry; 512]>()[idx];
            if PageLevel::from(i) == level {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = self.alloc.alloc(level.page_cnt()).unwrap();
                *pte = <Self as PageTableHal>::PageTableEntry::new(PhysPageNum(frame.start.0), MapPerm::empty());
                self.frames.push(FrameTracker::<PhysPageNum, A>::new(frame, self.alloc.clone()));
            }
            ppn = pte.ppn();
        }
        result
    }
}

#[allow(unused, missing_docs)]
impl<A> PageTableHal for PageTable<A>
    where A: FrameAllocatorHal<PhysPageNum = PhysPageNum>
{
    type VirtAddr = VirtAddr;

    type VirtPageNum = VirtPageNum;

    type PhysAddr = PhysAddr;

    type PhysPageNum = PhysPageNum;

    type PageLevel = PageLevel;

    type PageTableEntry = PageTableEntry;

    type FrameAllocator = A;
    
    fn new(ppn: Self::PhysPageNum, asid: usize, alloc: A) -> Self {
        Self { 
            root_ppn: ppn, 
            frames: Vec::new(),
            alloc
        }
    }
    
    fn find_pte(&self, vpn: Self::VirtPageNum) -> Option<(&mut Self::PageTableEntry, Self::PageLevel)> {
        todo!()
    }

    fn get_token(&self) -> usize {
        8 << 60 | self.root_ppn.0
    }

    fn translate_vpn(&self, vpn: Self::VirtPageNum) -> Self::PhysPageNum {
        todo!()
    }

    fn translate_va(&self, va: Self::VirtAddr) -> Self::PhysAddr {
        todo!()
    }

    fn map(&mut self, vpn: Self::VirtPageNum, ppn: Self::PhysPageNum, perm: crate::hal::mem::MapPerm, level: Self::PageLevel) {
        todo!()
    }

    fn unmap(&mut self, range_vpn: core::ops::Range<Self::VirtPageNum>) {
        todo!()
    }

    unsafe fn enable(&self) {
        todo!()
    }
    

}