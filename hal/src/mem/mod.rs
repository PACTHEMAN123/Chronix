mod addr;
mod vm;

use core::{iter::Step, ops::Range};

pub use addr::{KernAddrHal, PhysAddrHal, VirtAddrHal, KernPageNumHal, PhysPageNumHal, VirtPageNumHal, PageNumberHal};

pub use vm::{PageLevelHal, PageTableHal, MapPerm, PageTableEntryHal};

pub struct FrameTracker<PPN: PhysPageNumHal, A: FrameAllocatorHal<PhysPageNum = PPN>> {
    pub range_ppn: Range<PPN>,
    alloc: A
}

impl<PPN: PhysPageNumHal, A: FrameAllocatorHal<PhysPageNum = PPN>> FrameTracker<PPN, A> {
    pub fn new(range_ppn: Range<PPN>, alloc: A) -> Self {
        Self {
            range_ppn,
            alloc
        }
    }

    pub fn leak(mut self) -> Range<PPN> {
        let ret = self.range_ppn.clone();
        self.range_ppn.end = self.range_ppn.start;
        ret
    }
}

impl<PPN: PhysPageNumHal, A: FrameAllocatorHal<PhysPageNum = PPN>> Drop for FrameTracker<PPN, A> {
    fn drop(&mut self) {
        self.alloc.dealloc(self.range_ppn.clone());
    }
}

pub trait FrameAllocatorHal: Clone {
    type PhysPageNum: PhysPageNumHal;
    fn alloc(&mut self, size: usize) -> Option<Range<Self::PhysPageNum>>;
    fn dealloc(&mut self, range_ppn: Range<Self::PhysPageNum>);
}