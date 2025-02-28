//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.

use super::{PhysAddr, PhysPageNum};
use crate::config::{KERNEL_ADDR_OFFSET, MEMORY_END};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use bitmap_allocator::{BitAlloc, BitAlloc16M, BitAlloc4K};
use log::info;
use core::fmt::{self, Debug, Formatter};
use core::ops::Range;
use lazy_static::*;

/// manage a frame which has the same lifecycle as the tracker
#[allow(missing_docs)]
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    /// new FrameTracker from a Physical Page Number
    /// It is the caller's duty to clean the frame.
    pub fn new(ppn: PhysPageNum) -> Self {
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

trait FrameAllocator {
    fn init(&mut self, range_pa: Range<PhysAddr>);
    fn alloc(&mut self, size: usize) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum, size: usize) -> bool;
}

pub struct BitMapFrameAllocator {
    range: Range<PhysPageNum>,
    inner: bitmap_allocator::BitAlloc16M,
}

impl BitMapFrameAllocator {
    const fn new() -> Self {
        BitMapFrameAllocator {
            range: PhysPageNum(0)..PhysPageNum(1),
            inner: bitmap_allocator::BitAlloc16M::DEFAULT
        }
    }
}

impl FrameAllocator for BitMapFrameAllocator {


    fn alloc(&mut self, size: usize) -> Option<PhysPageNum> {
        self.inner.alloc_contiguous(None, size, 0).map(|u| { PhysPageNum(self.range.start.0 + u) })
    }

    fn dealloc(&mut self, ppn: PhysPageNum, size: usize) -> bool {
        self.inner.dealloc_contiguous(ppn.0 - self.range.start.0, size)
    }
    
    fn init(&mut self, range_pa: Range<PhysAddr>) {
        self.range = range_pa.start.ceil()..range_pa.end.floor();
        self.inner.insert(0..(range_pa.end.floor().0 - range_pa.start.ceil().0));
    }
}

pub static mut FRAME_ALLOCATOR: BitMapFrameAllocator = BitMapFrameAllocator::new();

// lazy_static! {
//     /// frame allocator instance through lazy_static!
//     pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
//         unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
// }

/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    unsafe {
        #[allow(static_mut_refs)]
        FRAME_ALLOCATOR.init(
            PhysAddr::from(ekernel as usize - KERNEL_ADDR_OFFSET)..PhysAddr::from(MEMORY_END),
        );
    }
}

#[allow(unused)]
/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    unsafe {
        #[allow(static_mut_refs)]
        FRAME_ALLOCATOR
            .alloc(1)
            .map(FrameTracker::new)
    }
}

#[allow(unused)]
/// allocate a frame
pub fn frame_alloc_clean() -> Option<FrameTracker> {
    frame_alloc().map(|f| { f.ppn.get_bytes_array().fill(0); f })
}


/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    unsafe {
        #[allow(static_mut_refs)]
        FRAME_ALLOCATOR.dealloc(ppn, 1);
    }
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
