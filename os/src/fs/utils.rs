use core::ops::Range;

use alloc::{sync::Arc, vec::Vec};
use hal::{addr::{VirtPageNum, VirtPageNumHal}, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::MapPerm, println};
use range_map::RangeMap;
use xmas_elf::reader::Reader;

use crate::{mm::vm::KernVmSpaceHal, sync::UPSafeCell};

use super::vfs::Inode;


pub struct FileReader<T: Inode + ?Sized> {
    inode: Arc<T>,
    mapped: UPSafeCell<RangeMap<usize, Range<VirtPageNum>>>,
}

impl<T: Inode + ?Sized> FileReader<T> {
    pub fn new(inode: Arc<T>) -> Self {
        Self { 
            inode,
            mapped: UPSafeCell::new(RangeMap::new())
        }
    }
}

impl<T: Inode + ?Sized> Reader for FileReader<T> {
    fn len(&self) -> usize {
        self.inode.getattr().st_size as usize
    }

    fn read(&self, offset: usize, len: usize) -> &[u8] {
        const MASK: usize = (1 << Constant::PAGE_SIZE_BITS) - 1;

        // align start and end
        let mut start = offset & !MASK;
        let mut end = (offset + len - 1 + Constant::PAGE_SIZE) & !MASK;

        loop {
            
            if let Some((range, range_vpn)) = self.mapped
                .exclusive_access()
                .range_contain_key_value(start..end) 
            {
                let area_offset = offset - range.start;
                return unsafe { 
                    core::slice::from_raw_parts(
                        (range_vpn.start.start_addr().0 + area_offset) as *const u8,
                        len
                    )
                };
            }

            loop {
                match self.mapped.exclusive_access().try_insert(start..end, 0.into()..0.into()) {
                    Ok(range_vpn_ref) => {
                        let mut frames = Vec::new();
                        for offset in (start..end).step_by(Constant::PAGE_SIZE) {
                            let page = self.inode.clone().read_page_at(offset).unwrap();
                            frames.push(page.frame().clone());
                        }
                        
                        // map file in kernel virtual memory
                        let range_vpn = crate::mm::INIT_VMSPACE.lock().map_vm_area(frames, MapPerm::R).unwrap();
                        for vpn in range_vpn.clone() {
                            unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
                        }

                        *range_vpn_ref = range_vpn;
                        break;
                    },
                    Err(_) => {
                        while let Some((range, range_vpn)) = self.mapped
                            .exclusive_access()
                            .range_contain_key_value(start..end) 
                        {
                            start = core::cmp::min(start, range.start);
                            end = core::cmp::max(end, range.end);
                            crate::mm::INIT_VMSPACE.lock().unmap_vm_area(range_vpn.clone());
                            self.mapped.exclusive_access().force_remove_one(range);
                        }
                    },
                }
            }
        }
    }
}

impl<T: Inode + ?Sized> Drop for FileReader<T> {
    fn drop(&mut self) {
        for (_, range_vpn) in self.mapped.exclusive_access().iter() {
            crate::mm::INIT_VMSPACE.lock().unmap_vm_area(range_vpn.clone());
        }
    }
}
