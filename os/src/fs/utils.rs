use core::{ops::Range, ptr::{null_mut, slice_from_raw_parts}};

use alloc::{sync::Arc, vec::Vec};
use hal::{addr::{VirtAddr, VirtAddrHal, VirtPageNumHal}, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::MapFlags, println};
use range_map::RangeMap;
use xmas_elf::reader::Reader;

use crate::{mm::{vm::{KernVmSpaceHal, PageFaultAccessType}, KVMSPACE}, sync::UPSafeCell};

use super::vfs::{File, Inode};


#[allow(unused)]
pub struct FileReader {
    inode: Arc<dyn Inode>,
    va: VirtAddr,
    len: usize
} 

impl FileReader {
    pub fn new(file: Arc<dyn File>) -> Self {
        let va = KVMSPACE.lock().mmap(file.clone()).unwrap();
        let inode = file.inode().unwrap();
        let len = inode.getattr().st_size as usize;
        Self { 
            inode,
            va,
            len
        }
    }
}

impl Reader for FileReader {
    fn len(&self) -> usize {
        self.len
    }

    fn read(&self, offset: usize, len: usize) -> &[u8] {
        if offset + len <= self.len {
            let start = (self.va + offset).floor();
            let end = (self.va + offset + len).ceil();
            for addr in start..end {
                KVMSPACE.lock().handle_page_fault(addr.start_addr(), PageFaultAccessType::READ).unwrap();
            }
            return unsafe {
                &*slice_from_raw_parts((self.va.0 + offset) as *const u8, len)
            }
        }
        return unsafe {
            &*slice_from_raw_parts(null_mut(), 0)
        };
    }
}

impl Drop for FileReader {
    fn drop(&mut self) {
        KVMSPACE.lock().unmap(self.va).unwrap();
    }
}
