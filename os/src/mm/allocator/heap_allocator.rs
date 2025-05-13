//! The global allocator
const KERNEL_HEAP_SIZE: usize = 64*1024*1024; // 64 MiB kept for operating system
use core::{alloc::{GlobalAlloc, Layout}, ptr::NonNull};

use alloc::alloc::Allocator;
use buddy_system_allocator::{Heap, LockedHeap};
use hal::println;

use crate::sync::mutex::SpinNoIrqLock;

/// heap allocator instance
static HEAP_INSTANCE: SpinNoIrqLock<Heap> = SpinNoIrqLock::new(Heap::empty());

#[allow(unused)]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator;

#[alloc_error_handler]
/// panic when heap allocation error occurs
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

/// Kernel Heap Allocator
#[derive(Clone)]
pub struct HeapAllocator;

unsafe impl Allocator for HeapAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        HEAP_INSTANCE
            .lock()
            .alloc(layout)
            .map(|ptr| NonNull::slice_from_raw_parts(ptr, layout.size()))
            .map_err(|_| alloc::alloc::AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        HEAP_INSTANCE.lock().dealloc(ptr, layout)
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HEAP_INSTANCE
            .lock()
            .alloc(layout).ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP_INSTANCE.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

/// heap space ([u8; KERNEL_HEAP_SIZE])
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        #[allow(static_mut_refs)]
        let start = HEAP_SPACE.as_ptr() as usize;
        HEAP_INSTANCE.lock().init(start, KERNEL_HEAP_SIZE);
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