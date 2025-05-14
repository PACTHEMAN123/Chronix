use core::{ops::Range, ptr::slice_from_raw_parts};

use alloc::{sync::Arc, vec::Vec};
use hal::{addr::{VirtAddr, VirtAddrHal}, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::MapFlags, println};
use range_map::RangeMap;
use xmas_elf::reader::Reader;

use crate::{mm::{vm::KernVmSpaceHal, KVMSPACE}, sync::UPSafeCell};

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
        unsafe {
            &*slice_from_raw_parts((self.va.0 + offset) as *const u8, len)
        }
    }
}

impl Drop for FileReader {
    fn drop(&mut self) {
        KVMSPACE.lock().unmap(self.va).unwrap();
    }
}
