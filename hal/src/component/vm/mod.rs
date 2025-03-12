use core::ops::Range;
use alloc::collections::btree_map::BTreeMap;
use crate::{allocator::FrameAllocatorHal, util::smart_point::StrongArc};
use super::{addr::{VirtAddr, VirtPageNum}, common::FrameTracker, pagetable::MapPerm};
use bitflags::bitflags;

#[derive(Debug, Clone, Copy)]
pub enum KernelVmAreaType {
    Data, PhysMem, MemMappedReg, KernelStack
}

#[derive(Debug, Clone, Copy)]
pub enum UserVmAreaType {
    Data, Heap, Stack, TrapContext
}

#[derive(Debug, Clone, Copy)]
pub enum VmAreaType {
    Kernel(KernelVmAreaType),
    User(UserVmAreaType)
}

pub struct VmArea<A: FrameAllocatorHal> {
    range_va: Range<VirtAddr>,
    pub vma_type: VmAreaType,
    pub map_perm: MapPerm,
    pub frames: BTreeMap<VirtPageNum, StrongArc<FrameTracker<A>>>,
    alloc: A,
}

bitflags! {
    /// PageFaultAccessType
    pub struct PageFaultAccessType: u8 {
        /// Read
        const READ = 1 << 0;
        /// Write
        const WRITE = 1 << 1;
        /// Execute
        const EXECUTE = 1 << 2;
    }
}

pub type VmSpaceUserStackTop = usize;
pub type VmSpaceEntryPoint = usize;

pub trait VmSpaceHal<A: FrameAllocatorHal>: Sized + Clone {
    fn new() -> Self;

    fn enable(&self);

    fn from_elf(elf_data: &[u8]) -> (Self, VmSpaceUserStackTop, VmSpaceEntryPoint);

    fn push_area(&mut self, area: VmArea<A>);

    fn reset_heap_break(&mut self, new_brk: VirtAddr) -> VirtAddr;

    fn handle_page_fault(&mut self, va: VirtAddr, access_type: PageFaultAccessType) -> Result<(), ()>;
}

#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::*;
