//! The global allocator
const KERNEL_HEAP_SIZE: usize = 64*1024*1024; // 64 MiB
use core::{alloc::{GlobalAlloc, Layout}, ptr::NonNull};

use buddy_system_allocator::{Heap, LockedHeap};
use hal::println;

use crate::sync::mutex::SpinNoIrqLock;

#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: GlobalHeap = GlobalHeap::empty();
//static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
/// panic when heap allocation error occurs
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

struct GlobalHeap(SpinNoIrqLock<Heap>);

impl GlobalHeap {
    const fn empty() -> Self {
        Self(SpinNoIrqLock::new(Heap::empty()))
    }
}

unsafe impl GlobalAlloc for GlobalHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout).ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

/// heap space ([u8; KERNEL_HEAP_SIZE])
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// initiate heap allocator
/*
pub fn init_heap() {
    unsafe {
        #[allow(static_mut_refs)]
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}
*/
pub fn init_heap() {
    unsafe {
        #[allow(static_mut_refs)]
        let start = HEAP_SPACE.as_ptr() as usize;
        HEAP_ALLOCATOR.0.lock().init(start, KERNEL_HEAP_SIZE);
        log::info!(
            "[kernel] heap start {:#x}, end {:#x}",
            start as usize,
            start + KERNEL_HEAP_SIZE
        );
    }
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for (i, val) in v.iter().take(500).enumerate() {
        assert_eq!(*val, i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}
