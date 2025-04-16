mod frame_allocator;
mod heap_allocator;
mod slab_allocator;

#[allow(unused)]
pub use frame_allocator::{FrameAllocator, init_frame_allocator, frames_alloc, frames_alloc_clean, frames_dealloc};
#[allow(unused)]
pub use heap_allocator::{handle_alloc_error, init_heap};
#[allow(unused)]
pub use slab_allocator::SlabAllocator;

/// next power of two
#[cfg(target_pointer_width="32")]
pub const fn next_power_of_two(mut x: usize) -> usize {
    if x == 0 {
        return 1
    }
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    return x+1;
}

/// next power of two
#[cfg(target_pointer_width="64")]
pub const fn next_power_of_two(mut x: usize) -> usize {
    if x == 0 {
        return 1
    }
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x |= x >> 32;
    return x+1;
}

/// log2
pub fn log2(x: usize) -> usize {
    (size_of::<usize>() << 3) - 1 - (x.leading_zeros() as usize)
}
