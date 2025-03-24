use core::ops::Range;

use crate::{addr::PhysPageNum, common::FrameTracker};

pub trait FrameAllocatorHal: Sync + Clone {
    fn alloc(&self, cnt: usize) -> Option<Range<PhysPageNum>>;
    fn dealloc(&self, range_ppn: Range<PhysPageNum>);

    fn alloc_tracker(&self, cnt: usize) -> Option<FrameTracker<Self>> {
        self.alloc(cnt).map(|range_ppn| FrameTracker::new_in(range_ppn, self.clone()))
    }

}

/// a fake frame allocator
#[derive(Clone)]
pub struct FakeFrameAllocator;

impl FrameAllocatorHal for FakeFrameAllocator {
    fn alloc(&self, _cnt: usize) -> Option<Range<PhysPageNum>> {
        panic!("alloc is not implemented by FakeFrameAllocator")
    }

    fn dealloc(&self, _range_ppn: Range<PhysPageNum>) {
        panic!("dealloc is not implemented by FakeFrameAllocator")
    }
}