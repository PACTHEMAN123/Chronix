use bitmap_allocator::BitAlloc4K;
use hal::addr::VirtPageNum;
use core::ops::Range;

/// User Vitrual Memoty Allocator
#[allow(unused)]
pub struct UserVmAllocator {
    range: Range<VirtPageNum>,
    inner: BitAlloc4K,
}