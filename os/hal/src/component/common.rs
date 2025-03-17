use core::ops::Range;

use crate::allocator::FrameAllocatorHal;

use super::addr::PhysPageNum;

pub struct FrameTracker<A: FrameAllocatorHal> {
    pub range_ppn: Range<PhysPageNum>,
    alloc: A
}

impl<A: FrameAllocatorHal> FrameTracker<A> {
    pub fn new_in(range_ppn: Range<PhysPageNum>, alloc: A) -> Self {
        Self{ range_ppn, alloc }
    }

    pub fn leak(mut self) -> Range<PhysPageNum> {
        let ret = self.range_ppn.clone();
        self.range_ppn.end = self.range_ppn.start;
        ret
    }
}

impl<A: FrameAllocatorHal> Drop for FrameTracker<A> {
    fn drop(&mut self) {
        self.alloc.dealloc(self.range_ppn.clone());
    }
}
