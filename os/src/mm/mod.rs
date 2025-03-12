//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod user_check;
/// allocator
pub mod allocator;
mod page_table;
use core::ops::Deref;

use hal::vm::VmSpaceHal;
pub use user_check::UserCheck;

pub use page_table::*;

#[allow(missing_docs)]
pub type VmSpace = hal::vm::VmSpace<allocator::FrameAllocator>;
#[allow(missing_docs)]
pub type PageTable = hal::pagetable::PageTable<allocator::FrameAllocator>;
#[allow(missing_docs)]
pub type FrameTracker = hal::common::FrameTracker<allocator::FrameAllocator>;

use super::sync::mutex::SpinNoIrqLock;
lazy_static::lazy_static! {
    #[allow(missing_docs)]
    pub static ref INIT_VMSPACE: SpinNoIrqLock<VmSpace> = SpinNoIrqLock::new(VmSpace::new());
}

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    allocator::init_heap();
    allocator::init_frame_allocator();
    hal::vm::VmSpaceHal::enable(INIT_VMSPACE.lock().deref());
}
