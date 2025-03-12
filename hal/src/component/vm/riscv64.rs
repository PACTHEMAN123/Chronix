use alloc::vec::Vec;

use crate::{allocator::FrameAllocatorHal, pagetable::{PageTable, PageTableHal}};

use super::{VmArea, VmSpaceHal};

pub struct VmSpace<A: FrameAllocatorHal> {
    pub page_table: PageTable<A>,
    areas: Vec<VmArea<A>>,
    alloc: A,
}

impl<A: FrameAllocatorHal> VmSpace<A> {
    pub fn new_kernel() {
        
    }
}

impl<A: FrameAllocatorHal> VmSpaceHal<A> for VmSpace<A> {
    fn new() -> Self {
        todo!()
    }

    fn enable(&self) {
        unsafe { 
            self.page_table.enable();
        }
    }

    fn from_elf(elf_data: &[u8]) -> (Self, super::VmSpaceUserStackTop, super::VmSpaceEntryPoint) {
        todo!()
    }

    fn push_area(&mut self, area: VmArea<A>) {
        todo!()
    }

    fn reset_heap_break(&mut self, new_brk: crate::addr::VirtAddr) -> crate::addr::VirtAddr {
        todo!()
    }

    fn handle_page_fault(&mut self, va: crate::addr::VirtAddr, access_type: super::PageFaultAccessType) -> Result<(), ()> {
        todo!()
    }
}

impl<A: FrameAllocatorHal> Clone for VmSpace<A> {
    fn clone(&self) -> Self {
        todo!()
    }
}