use core::ops::Range;

use crate::{addr::PhysPageNum, common::FrameTracker};

pub trait FrameAllocatorHal: Sync + Clone {
    fn alloc(&self, cnt: usize) -> Option<Range<PhysPageNum>>;
    fn dealloc(&self, range_ppn: Range<PhysPageNum>);

    fn alloc_tracker(&self, cnt: usize) -> Option<FrameTracker<Self>> {
        self.alloc(cnt).map(|range_ppn| FrameTracker::new_in(range_ppn, self.clone()))
    }

}
