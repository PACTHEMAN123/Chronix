use core::{cell::SyncUnsafeCell, ops::Range};

use crate::{addr::PhysPageNum, common::FrameTracker};

pub trait FrameAllocatorHal: Sync {
    fn alloc(&self, cnt: usize) -> Option<Range<PhysPageNum>> {
        self.alloc_with_align(cnt, 0)
    }
    fn alloc_with_align(&self, cnt: usize, align_log2: usize) -> Option<Range<PhysPageNum>>;
    fn dealloc(&self, range_ppn: Range<PhysPageNum>);
}

pub trait FrameAllocatorTrackerExt: FrameAllocatorHal + Clone {
    fn alloc_tracker(&self, cnt: usize) -> Option<FrameTracker<Self>> {
        self.alloc_with_align(cnt, 0).map(
            |range_ppn| FrameTracker::new_in(range_ppn, self.clone())
        )
    }
}

impl<T: FrameAllocatorHal + Clone> FrameAllocatorTrackerExt for T {}

/// a fake frame allocator
#[derive(Clone)]
pub struct FakeFrameAllocator;

impl FrameAllocatorHal for FakeFrameAllocator {
    fn alloc_with_align(&self, _cnt: usize, _align_log2: usize) -> Option<Range<PhysPageNum>> {
        panic!("alloc is not implemented by FakeFrameAllocator")
    }

    fn dealloc(&self, _range_ppn: Range<PhysPageNum>) {
        panic!("dealloc is not implemented by FakeFrameAllocator")
    }
}

static mut DYNAMIC_FRAME_ALLOCATOR: SyncUnsafeCell<&'static dyn FrameAllocatorHal> = SyncUnsafeCell::new(&FakeFrameAllocator);

/// a dynamic frame allocator
#[derive(Clone)]
pub struct DynamicFrameAllocator;

impl FrameAllocatorHal for DynamicFrameAllocator {
    fn alloc_with_align(&self, cnt: usize, align_log2: usize) -> Option<Range<PhysPageNum>> {
        let allocator = unsafe {
            #[allow(static_mut_refs)]
            *DYNAMIC_FRAME_ALLOCATOR.get()
        };
        allocator.alloc_with_align(cnt, align_log2)
    }

    fn dealloc(&self, range_ppn: Range<PhysPageNum>) {
        let allocator = unsafe {
            #[allow(static_mut_refs)]
            *DYNAMIC_FRAME_ALLOCATOR.get()
        };
        allocator.dealloc(range_ppn)
    }
}

/// unsafe: this function changes a static variable without synchronization
#[allow(static_mut_refs)]
pub unsafe fn register_frame_allocator(allocator: &'static dyn FrameAllocatorHal) {
    *DYNAMIC_FRAME_ALLOCATOR.get_mut() = allocator;
}