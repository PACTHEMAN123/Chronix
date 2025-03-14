use core::ops::Range;
use alloc::collections::btree_map::BTreeMap;
use crate::{allocator::FrameAllocatorHal, util::smart_point::StrongArc};
use super::{addr::{VirtAddr, VirtPageNum}, common::FrameTracker, pagetable::{MapPerm, PageTable, PageTableHal}};
use bitflags::bitflags;

#[derive(Debug, Clone, Copy,  PartialEq, Eq)]
pub enum KernVmAreaType {
    Data, PhysMem, MemMappedReg, KernelStack
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserVmAreaType {
    Data, Heap, Stack, TrapContext
}

pub struct UserVmArea<A: FrameAllocatorHal> {
    range_va: Range<VirtAddr>,
    pub vma_type: UserVmAreaType,
    pub map_perm: MapPerm,
    pub frames: BTreeMap<VirtPageNum, StrongArc<FrameTracker<A>>>,
    alloc: A,
}

impl<A: FrameAllocatorHal> UserVmArea<A> {
    fn new(
        range_va: Range<VirtAddr>, 
        vma_type: UserVmAreaType, 
        map_perm: MapPerm, 
        alloc: A
    ) -> Self {
        Self {
            range_va,
            vma_type,
            map_perm,
            frames: BTreeMap::new(),
            alloc
        }
    }
}

pub struct KernVmArea<A: FrameAllocatorHal> {
    range_va: Range<VirtAddr>,
    pub vma_type: KernVmAreaType,
    pub map_perm: MapPerm,
    pub frames: BTreeMap<VirtPageNum, FrameTracker<A>>,
    pub alloc: A
}

impl<A: FrameAllocatorHal> KernVmArea<A> {
    fn new(
        range_va: Range<VirtAddr>, 
        vma_type: KernVmAreaType, 
        map_perm: MapPerm, 
        alloc: A
    ) -> Self {
        Self {
            range_va,
            vma_type,
            map_perm,
            frames: BTreeMap::new(),
            alloc
        }
    }
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

impl PageFaultAccessType {
    
    pub fn can_access(self, flag: MapPerm) -> bool {
        if self.contains(Self::WRITE) && !flag.contains(MapPerm::W) && !flag.contains(MapPerm::C) {
            return false;
        }
        if self.contains(Self::EXECUTE) && !flag.contains(MapPerm::X) {
            return false;
        }
        true
    }
}

pub type VmSpaceUserStackTop = usize;
pub type VmSpaceEntryPoint = usize;

pub trait KernVmSpaceHal<A: FrameAllocatorHal> {

    fn get_page_table(&self) -> &PageTable<A>;

    fn enable(&self) {
        unsafe {
            self.get_page_table().enable();
        }
    }

    fn new_in(alloc: A) -> Self;

    fn push_area(&mut self, area: KernVmArea<A>, data: Option<&[u8]>);
}

pub trait UserVmSpaceHal<A: FrameAllocatorHal, K: KernVmSpaceHal<A>>: Sized {

    fn new_in(alloc: A) -> Self;

    fn get_page_table(&self) -> &PageTable<A>;

    fn enable(&self) {
        unsafe {
            self.get_page_table().enable();
        }
    }

    fn from_kernel(kvm_space: &K) -> Self;

    fn from_elf(elf_data: &[u8], kvm_space: &K) -> (Self, VmSpaceUserStackTop, VmSpaceEntryPoint);

    fn from_existed(uvm_space: &mut Self, kvm_space: &K) -> Self;

    fn push_area(&mut self, area: UserVmArea<A>, data: Option<&[u8]>);

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
