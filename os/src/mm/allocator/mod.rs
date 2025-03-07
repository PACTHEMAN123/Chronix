mod frame_allocator;
mod heap_allocator;
mod slab_allocator;

#[allow(unused)]
pub use frame_allocator::{FRAME_ALLOCATOR, FrameTracker, frame_alloc, frame_alloc_clean, frame_dealloc, init_frame_allocator};
#[allow(unused)]
pub use heap_allocator::{handle_alloc_error, init_heap};
#[allow(unused)]
pub use slab_allocator::{SlabAllocator, slab_alloc, slab_dealloc, slab_shrink};