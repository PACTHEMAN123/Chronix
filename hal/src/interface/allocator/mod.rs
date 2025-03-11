use core::ops::Range;

use crate::addr::PhysPageNum;

pub trait FrameAllocatorHal: Sync + Clone {
    fn alloc(&self, cnt: usize) -> Option<Range<PhysPageNum>>;
    fn dealloc(&self, range_ppn: Range<PhysPageNum>);
}
