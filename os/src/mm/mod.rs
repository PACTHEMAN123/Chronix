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

use hal::constant::{Constant, ConstantsHal};
use vm::{KernVmArea, KernVmSpaceHal};

pub use page_table::*;

#[allow(missing_docs)]
pub type KernVmSpace = vm::KernVmSpace;
#[allow(missing_docs)]
pub type UserVmSpace = vm::UserVmSpace;
#[allow(missing_docs)]
pub type PageTable = hal::pagetable::PageTable<allocator::FrameAllocator>;
#[allow(missing_docs)]
pub type FrameTracker = hal::common::FrameTracker<allocator::FrameAllocator>;

pub struct MmioMapper;

impl hal::mapper::MmioMapperHal for MmioMapper {
    #[cfg(target_arch = "riscv64")]
    fn map_mmio_area(&self, range: core::ops::Range<usize>) -> core::ops::Range<usize> {
        use hal::println;
        let va_start = hal::addr::VirtAddr::from(range.start | Constant::KERNEL_ADDR_SPACE.start);
        let va_end = hal::addr::VirtAddr::from(range.end | Constant::KERNEL_ADDR_SPACE.start);
        KVMSPACE.lock().push_area(KernVmArea::new(
                va_start..va_end, 
                vm::KernVmAreaType::MemMappedReg, 
                hal::pagetable::MapPerm::R | hal::pagetable::MapPerm::W
            ), 
            None
        );
        va_start.0..va_end.0
    }

    #[cfg(target_arch = "loongarch64")]
    fn map_mmio_area(&self, range: core::ops::Range<usize>) -> core::ops::Range<usize> {
        let va_start = range.start | 0x8000_0000_0000_0000;
        let va_end = range.end | 0x8000_0000_0000_0000;
        va_start..va_end
    }
}

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
