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
    inner: bitmap_allocator::BitAlloc16M,
}

impl FrameAllocatorTrait for BitMapFrameAllocator {
    const DEFAULT: Self = BitMapFrameAllocator {
        range: PhysPageNum(0)..PhysPageNum(0),
        inner: bitmap_allocator::BitAlloc16M::DEFAULT
    };

    fn init(&mut self, range_pa: Range<PhysAddr>) {
        self.range = range_pa.start.ceil()..range_pa.end.floor();
        info!("[FrameAllocator] range: {:#x}..{:#x}", range_pa.start.0, range_pa.end.0);
        self.inner.insert(0..(range_pa.end.floor().0 - range_pa.start.ceil().0));
    }
    
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<Range<PhysPageNum>> {
        if align_log2 > 0 {
            log::warn!("BitMapFrameAllocator cannot support aligned allocate");
        }
        let mut start = self.inner.alloc_contiguous(None, size, 0)?;
        start += self.range.start.0;
        let range_ppn = PhysPageNum(start)..PhysPageNum(start + size);
        Some(range_ppn)
    }
    
    fn dealloc_contiguous(&mut self, range_ppn: Range<PhysPageNum>) {
        if range_ppn.clone().count() == 0 {
            return;
        }
        let start = range_ppn.start.0 - self.range.start.0;
        self.inner.dealloc_contiguous(start, range_ppn.count());
    }
    
}

/// Buddy Frame Allocator
struct BuddyFrameAllocator {
    inner: Heap,
}

impl FrameAllocatorTrait for BuddyFrameAllocator {
    const DEFAULT: Self = Self {
        inner: Heap::empty()
    };

    fn init(&mut self, range_pa: Range<PhysAddr>) {
        info!("[FrameAllocator] range: {:#x}..{:#x}", range_pa.start.0, range_pa.end.0);
        unsafe { self.inner.init(range_pa.start.get_ptr::<u8>() as usize, range_pa.end.get_ptr::<u8>() as usize - range_pa.start.get_ptr::<u8>() as usize) };
    }
    
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<Range<PhysPageNum>> {
        let size = size << Constant::PAGE_SIZE_BITS;
        let align_log2 = align_log2 + Constant::PAGE_SIZE_BITS;
        self.inner.alloc(Layout::from_size_align(size, 1 << align_log2).unwrap()).ok().map(|p| {
            let pa = PhysAddr::from(p.addr().get() & !Constant::KERNEL_ADDR_SPACE.start);
            let ppn = pa.floor();
            ppn..ppn+size
        })
    }
    
    fn dealloc_contiguous(&mut self, range_ppn: Range<PhysPageNum>) {
        let ptr = NonNull::new(range_ppn.start.start_addr().get_ptr::<u8>()).unwrap();
        let size = range_ppn.count() << Constant::PAGE_SIZE_BITS;
        self.inner.dealloc(ptr, Layout::from_size_align(size, Constant::PAGE_SIZE).unwrap());
    }
}


/// frame allocator
static FRAME_ALLOCATOR: SpinNoIrqLock<BitMapFrameAllocator> = SpinNoIrqLock::new(BitMapFrameAllocator::DEFAULT);


#[allow(missing_docs)]
#[derive(Clone)]
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

#[allow(unused)]
/// allocate frames
pub fn frames_alloc(size: usize) -> Option<FrameTracker> {
    FrameAllocator
        .alloc(size)
        .map(|ppn| {
            FrameTracker::new_in(ppn, FrameAllocator)
        })
}

#[allow(unused)]
/// allocate frames and clean
pub fn frames_alloc_clean(size: usize) -> Option<FrameTracker> {
    frames_alloc(size).map(|f| {
        f.range_ppn.get_slice_mut::<u8>().fill(0);
        f
    })
}

/// deallocate frames
#[allow(unused)]
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
