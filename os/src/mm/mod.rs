//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod page_table;
mod user_check;
mod smart_pointer;
/// virtual memory
pub mod vm;
/// allocator
pub mod allocator;

pub use address::*;
pub use page_table::{translated_byte_buffer, translated_str, translated_ref, translated_refmut, 
    copy_out, copy_out_str, PageTableEntry, UserBuffer, PTEFlags, PageTable};
pub use user_check::UserCheck;
pub use smart_pointer::{StrongArc, SlabArc, SlabWeak, ArcNewInSlab};


/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    allocator::init_heap();
    allocator::init_frame_allocator();
    vm::VmSpace::enable(vm::KERNEL_SPACE.exclusive_access());
}
