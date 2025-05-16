//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

/// allocator
pub mod allocator;
mod page_table;
use core::ops::Deref;
/// virtual memory
pub mod vm;

mod user;

pub use user::*;

use hal::println;
use vm::KernVmSpaceHal;

pub use page_table::*;

#[allow(missing_docs)]
pub type KernVmSpace = vm::KernVmSpace;
#[allow(missing_docs)]
pub type UserVmSpace = vm::UserVmSpace;
#[allow(missing_docs)]
pub type PageTable = hal::pagetable::PageTable<allocator::FrameAllocator>;
#[allow(missing_docs)]
pub type FrameTracker = hal::common::FrameTracker<allocator::FrameAllocator>;

use super::sync::mutex::SpinNoIrqLock;
lazy_static::lazy_static! {
    #[allow(missing_docs)]
    pub static ref KVMSPACE: SpinNoIrqLock<KernVmSpace> = SpinNoIrqLock::new(KernVmSpace::new());
}

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    allocator::init_heap();
    allocator::init_frame_allocator();
    vm::KernVmSpaceHal::enable(KVMSPACE.lock().deref());
}
