//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod page_table;
mod vm_area;
mod vm_space;
mod user_check;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
pub use frame_allocator::{frame_alloc, frame_alloc_clean, frame_dealloc, FrameTracker};
pub use page_table::{translated_byte_buffer, PageTableEntry, translated_str, translated_ref, translated_refmut, UserBuffer};
pub use page_table::{PTEFlags, PageTable};
#[allow(unused)]
pub use vm_area::{UserVmArea, KernelVmArea, VmArea, VmAreaFrameExt, MapPerm, KernelVmAreaType, UserVmAreaType};
pub use vm_space::{VmSpace, KERNEL_SPACE, UserVmSpace, remap_test, PageFaultAccessType, VmAreaContainer, VmSpacePageFaultExt, VmSpaceHeapExt};
pub use user_check::UserCheck;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().enable();
}
