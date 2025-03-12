//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.

use crate::config::{KERNEL_ADDR_OFFSET, MEMORY_END};
use crate::mm::address::{PhysAddr, PhysPageNum};
use crate::mm::{RangeKpnData, ToRangeKpn};
use crate::sync::mutex::spin_mutex::SpinMutex;
use crate::sync::mutex::Spin;
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use bitmap_allocator::{BitAlloc, BitAlloc16M, BitAlloc4K};
use hal::println;
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

    /// leak
    pub fn leak(mut self) -> PhysPageNum {
        let ret = self.ppn;
        self.ppn.0 = 0;
        ret
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        if self.ppn.0 == 0 {
            return;
        }
        frame_dealloc(self.ppn);
    }
}

#[allow(unused, missing_docs)]
pub struct FrameRangeTracker {
    pub range_ppn: Range<PhysPageNum>
}

#[allow(unused, missing_docs)]
impl FrameRangeTracker {
    /// new FrameRangeTracker from a range of Physical Page Number
    /// It is the caller's duty to clean the frame.
    pub fn new(range_ppn: Range<PhysPageNum>) -> Self {
        Self { range_ppn }
    }

    pub fn clean(&self) {
        self.range_ppn.to_kern().get_slice::<u8>().fill(0);
    }

    /// leak
    pub fn leak(mut self) -> Range<PhysPageNum> {
        let ret = self.range_ppn.clone();
        self.range_ppn.end = self.range_ppn.start;
        ret
    }
}

impl Debug for FrameRangeTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameRangeTracker:PPN Range={:#x?}", self.range_ppn))
    }
}

impl Drop for FrameRangeTracker {
    fn drop(&mut self) {
        frames_dealloc(self.range_ppn.clone());
    }
}

trait FrameAllocator {
    fn init(&mut self, range_pa: Range<PhysAddr>);
    fn alloc(&mut self, size: usize) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum, size: usize) -> bool;
}

struct BitMapFrameAllocator {
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

/// frame allocator
static FRAME_ALLOCATOR: SpinMutex<BitMapFrameAllocator, Spin> = SpinMutex::new(BitMapFrameAllocator::new());

/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }

    FRAME_ALLOCATOR.lock().init(
        PhysAddr::from(ekernel as usize - KERNEL_ADDR_OFFSET)..PhysAddr::from(MEMORY_END),
    );
}

#[allow(unused)]
/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .lock()
        .alloc(1)
        .map(FrameTracker::new)
}

#[allow(unused)]
/// allocate frames
pub fn frames_alloc(size: usize) -> Option<FrameRangeTracker> {
    FRAME_ALLOCATOR
        .lock()
        .alloc(size)
        .map(|ppn| {
            FrameRangeTracker::new(ppn..ppn+size)
        })
}

#[allow(unused)]
/// allocate frames and clean
pub fn frames_alloc_clean(size: usize) -> Option<FrameRangeTracker> {
    frames_alloc(size).map(|f| {
        f.range_ppn.to_kern().get_slice::<u8>().fill(0);
        f
    })
}


#[allow(unused)]
/// allocate a frame
pub fn frame_alloc_clean() -> Option<FrameTracker> {
    frame_alloc().map(|f| { f.ppn.to_kern().get_bytes_array().fill(0); f })
}


/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(ppn, 1);
}

/// deallocate frames
#[allow(unused)]
pub fn frames_dealloc(range_ppn: Range<PhysPageNum>) {
    if range_ppn.clone().count() > 0 {
        FRAME_ALLOCATOR.lock().dealloc(range_ppn.start, range_ppn.count());
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
