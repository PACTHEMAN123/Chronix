mod frame_allocator;
mod heap_allocator;
// mod slab_allocator;

/// User Virtual Memory Allocator
mod uvm_allocator;

#[allow(unused)]
pub use frame_allocator::{FrameAllocator, init_frame_allocator, frames_alloc, frames_alloc_clean, frames_dealloc};
#[allow(unused)]
pub use heap_allocator::{handle_alloc_error, init_heap};
// #[allow(unused)]
// pub use slab_allocator::{SlabAllocator, slab_alloc, slab_dealloc, slab_shrink};