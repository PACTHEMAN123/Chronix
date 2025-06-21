//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.
use crate::sync::mutex::spin_mutex::SpinMutex;
use crate::sync::mutex::{Spin, SpinNoIrqLock};
use crate::sync::UPSafeCell;
use alloc::alloc::Allocator;
use alloc::vec::Vec;
use bitmap_allocator::{BitAlloc, BitAlloc16M, BitAlloc4K};
use buddy_system_allocator::Heap;
use hal::addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, RangePPNHal};
use hal::allocator::FrameAllocatorHal;
use hal::constant::{Constant, ConstantsHal};
use hal::println;
use log::info;
use core::alloc::Layout;
use core::fmt::{self, Debug, Formatter};
use core::ops::Range;
use core::ptr::NonNull;
use lazy_static::*;

trait FrameAllocatorTrait {
    const DEFAULT: Self;
    fn init(&mut self, range_pa: Range<PhysAddr>);
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<Range<PhysPageNum>>;
    fn dealloc_contiguous(&mut self, range_ppn: Range<PhysPageNum>);
}

/// Bitmap Frame Allocator, the supported maximum memory space is 64GiB
struct BitMapFrameAllocator {
    range: Range<PhysPageNum>,
    align_log2: usize,
    inner: bitmap_allocator::BitAlloc16M,
    last: usize,
}

impl FrameAllocatorTrait for BitMapFrameAllocator {
    const DEFAULT: Self = BitMapFrameAllocator {
        range: PhysPageNum(0)..PhysPageNum(0),
        align_log2: 8,
        inner: bitmap_allocator::BitAlloc16M::DEFAULT,
        last: 0
    };

    fn init(&mut self, range_pa: Range<PhysAddr>) {
        // aligned start --- real start --- real end
        // aligned start..real start is unused
        let start = range_pa.start.floor();
        let aligned_start = start.0 & !((1 << self.align_log2) - 1);
        let aligned_range_ppn = PhysPageNum::from(aligned_start)..range_pa.end.floor();
        self.range = aligned_range_ppn.clone();
        let beg = start.0 - aligned_range_ppn.start.0;
        let end = aligned_range_ppn.end.0 - aligned_range_ppn.start.0;
        self.last = end - beg;
        info!("[FrameAllocator] pages: {}", self.last);
        self.inner.insert(beg..end);
    }
    
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<Range<PhysPageNum>> {
        if align_log2 > self.align_log2 {
            log::warn!("BitMapFrameAllocator cannot support align to {:#x}", 1 << align_log2);
        }
        let mut start = match self.inner.alloc_contiguous(None, size, align_log2) {
            Some(v) => v,
            None => {
                log::warn!("cannot alloc size: {size}, align_log2: {align_log2}, last: {}", self.last);
                return None
            },
        };
        self.last -= size;
        start += self.range.start.0;
        let range_ppn = PhysPageNum(start)..PhysPageNum(start + size);
        Some(range_ppn)
    }
    
    fn dealloc_contiguous(&mut self, range_ppn: Range<PhysPageNum>) {
        let size = range_ppn.clone().count();
        if size == 0 {
            return;
        }
        let start = range_ppn.start.0 - self.range.start.0;
        assert!(self.inner.dealloc_contiguous(start, size));
        self.last += size;
    }
    
}

/// frame allocator
static FRAME_ALLOCATOR: SpinNoIrqLock<BitMapFrameAllocator> = SpinNoIrqLock::new(BitMapFrameAllocator::DEFAULT);


#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct FrameAllocator;

pub type FrameTracker = hal::common::FrameTracker<FrameAllocator>;

impl FrameAllocatorHal for FrameAllocator {

    fn alloc_with_align(&self, cnt: usize, align_log2: usize) -> Option<Range<PhysPageNum>> {
        if cnt == 0 {
            return None
        }
        let mut alloc_guard = FRAME_ALLOCATOR.lock();
        alloc_guard.alloc_contiguous(cnt, align_log2)
    }

    fn dealloc(&self, range_ppn: Range<PhysPageNum>) {
        let mut alloc_guard = FRAME_ALLOCATOR.lock();
        alloc_guard.dealloc_contiguous(range_ppn)
    }
}

unsafe impl Allocator for FrameAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        let pg_cnt = (layout.size() - 1 + Constant::PAGE_SIZE) / Constant::PAGE_SIZE;
        let pg_align = (layout.align() - 1 + Constant::PAGE_SIZE) / Constant::PAGE_SIZE;
        let frame = FrameAllocatorHal::alloc_with_align(self, pg_cnt, super::log2(pg_align))
            .ok_or(alloc::alloc::AllocError)?;
        NonNull::new(&mut frame.get_slice_mut::<u8>()[..layout.size()])
            .ok_or(alloc::alloc::AllocError)
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        let pg_cnt = (layout.size() - 1 + Constant::PAGE_SIZE) / Constant::PAGE_SIZE;
        let start_ppn = PhysAddr(ptr.as_ptr() as usize & !Constant::KERNEL_ADDR_SPACE.start).floor();
        FrameAllocatorHal::dealloc(self, start_ppn..start_ppn+pg_cnt);
    }
}

/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }

    FRAME_ALLOCATOR.lock().init(
        PhysAddr::from(ekernel as usize & !Constant::KERNEL_ADDR_SPACE.start)..PhysAddr::from(Constant::MEMORY_END),
    );
}

/// allocate frames
pub fn frames_alloc(size: usize) -> Option<FrameTracker> {
    FrameAllocator
        .alloc(size)
        .map(|ppn| {
            FrameTracker::new_in(ppn, FrameAllocator)
        })
}

/// allocate frames and clean
pub fn frames_alloc_clean(size: usize) -> Option<FrameTracker> {
    frames_alloc(size).map(|f| {
        f.range_ppn.get_slice_mut::<u8>().fill(0);
        f
    })
}

/// deallocate frames
pub fn frames_dealloc(range_ppn: Range<PhysPageNum>) {
    if range_ppn.clone().count() > 0 {
        FrameAllocator.dealloc(range_ppn);
    }
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frames_alloc(1).unwrap();
        // println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frames_alloc(1).unwrap();
        // println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
