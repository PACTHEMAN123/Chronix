use core::ops::Range;

use alloc::{format, sync::Arc};
use hal::{addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, RangePPNHal, VirtAddr, VirtAddrHal, VirtPageNum, VirtPageNumHal}, allocator::FrameAllocatorHal, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::{MapPerm, PageLevel, PageTableEntry, PageTableEntryHal, PageTableHal, VpnPageRangeIter}};
use range_map::RangeMap;

use crate::{fs::vfs::File, mm::{allocator::FrameAllocator, vm::KernVmAreaType, PageTable}};

use super::super::{KernVmArea, KernVmSpaceHal, PageFaultAccessType, UserVmSpace, UserVmSpaceHal};

/// Kernel's VmSpace
pub struct KernVmSpace {
    page_table: PageTable,
    areas: RangeMap<VirtPageNum, KernVmArea>,
}

impl KernVmSpaceHal for KernVmSpace {

    fn enable(&self) {
        unsafe { 
            self.page_table.enable_high();
            Instruction::tlb_flush_all();
        }
    }

    fn get_page_table(&self) -> &PageTable {
        &self.page_table
    }

    fn new() -> Self {
        let mut ret = Self {
            page_table: PageTable::new_in(0, FrameAllocator),
            areas: RangeMap::new()
        };

        ret.push_area(
            KernVmArea::new(
                Constant::SIGRET_TRAMPOLINE_BOTTOM.into()..Constant::SIGRET_TRAMPOLINE_TOP.into(),
                KernVmAreaType::SigretTrampoline,
                MapPerm::U | MapPerm::R | MapPerm::X, 
            ),
            None
        );

        ret
    }

    fn to_user<T: UserVmSpaceHal>(&self) -> T {
        T::new()
    }
    
    fn push_area(&mut self, area: KernVmArea, _data: Option<&[u8]>) {
        if area.map(&mut self.page_table).is_ok() {
            let _ = self.areas.try_insert(area.range_vpn(), area);
        }
    }

    fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PhysPageNum>{
        Some(PhysPageNum(vpn.0 & !(0x8_0000_0000_0000)))
    }
    
    fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        Some(PhysAddr(va.0 & !(0x8000_0000_0000_0000)))
    }

    fn mmap(&mut self, file: Arc<dyn File>) -> Result<VirtAddr, ()> {
        let len = file.inode().ok_or(())?.getattr().st_size as usize;
        let len = (len - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE - 1);
        let range_vpn = self.areas.find_free_range(   
            VirtAddr::from(Constant::KERNEL_VM_BOTTOM).floor()..VirtAddr::from(Constant::KERNEL_VM_TOP).floor(), 
            len / Constant::PAGE_SIZE
        ).ok_or(())?;
        let range_va = range_vpn.start.start_addr()..range_vpn.end.start_addr();
        let mut vma = KernVmArea::new(range_va.clone(), KernVmAreaType::Mmap, MapPerm::R);
        vma.file = Some(file.clone());
        self.push_area(vma, None);
        
        Ok(range_va.start)
    }
    
    fn unmap(&mut self, va: VirtAddr) -> Result<(), ()> {
        let (range, area) = self.areas.get_key_value_mut(va.floor()).ok_or(())?;
        area.unmap(&mut self.page_table);
        self.areas.force_remove_one(range);
        Ok(())
    }
    
    fn handle_page_fault(&mut self, va: VirtAddr, access_type: PageFaultAccessType) -> Result<(), ()> {
        let area = self.areas.get_mut(va.floor()).ok_or(())?;
        match area.vma_type {
            KernVmAreaType::Mmap => {
                if access_type.contains(PageFaultAccessType::WRITE) || access_type.contains(PageFaultAccessType::EXECUTE) {
                    return Err(())
                }
                let file = area.file.clone().ok_or(())?;
                let inode = file.inode().ok_or(())?;
                let vpn = va.floor();
                let offset = (vpn.0 - area.range_vpn().start.0) * Constant::PAGE_SIZE;
                let page = inode.read_page_at(offset).ok_or(())?;
                let _ = self.page_table.map(vpn, page.ppn(), MapPerm::R, PageLevel::Small);
                area.frames.insert(vpn, page.frame());
                unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
                Ok(())
            },
            _ => Err(())
        }
    }

}


#[allow(missing_docs, unused)]
impl KernVmArea {

    fn range_vpn(&self) -> Range<VirtPageNum> {
        self.range_va.start.floor()..self.range_va.end.ceil()
    }

    fn split_off(&mut self, p: VirtPageNum) -> Self {
        let ret = Self {
            range_va: p.start_addr()..self.range_va.end,
            frames: self.frames.split_off(&p),
            map_perm: self.map_perm,
            vma_type: self.vma_type,
            file: self.file.clone(),
        };
        self.range_va = self.range_va.start..p.start_addr();
        ret
    }

    fn map_range_to(&self, page_table: &mut PageTable, range_vpn: Range<VirtPageNum>, mut start_ppn: PhysPageNum) {
        VpnPageRangeIter::new(range_vpn)
        .for_each(|(vpn, level)| {
            let ppn = PhysPageNum(start_ppn.0);
            start_ppn += level.page_count();
            let _ = page_table.map(vpn, ppn, self.map_perm, level);
        });
    }

    fn map(&self, page_table: &mut PageTable) -> Result<(), ()>{
        unsafe extern "C" {
            fn sigreturn_trampoline();
        }
        let range_vpn = self.range_va.start.floor()..self.range_va.end.ceil();
        match self.vma_type {
            KernVmAreaType::Data |
            KernVmAreaType::PhysMem |
            KernVmAreaType::MemMappedReg |
            KernVmAreaType::KernelStack => {
                Err(())
            },
            KernVmAreaType::SigretTrampoline => {
                let sigret_trampoline_ppn = 
                    PhysPageNum((sigreturn_trampoline as usize & !(Constant::KERNEL_ADDR_SPACE.start)) >> 12);
                for (vpn, ppn) in self.range_vpn().zip(sigret_trampoline_ppn..sigret_trampoline_ppn+1) {
                    let pte = page_table.map(vpn, ppn, self.map_perm, PageLevel::Small)
                        .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
                    pte.set_dirty(true);
                    pte.set_valid(true);
                }
                Ok(())
            }
            KernVmAreaType::VirtMemory => {
                for (&vpn, frame) in self.frames.iter() {
                    let pte = page_table.map(vpn, frame.range_ppn.start, self.map_perm, PageLevel::Small)
                        .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
                    pte.set_dirty(true);
                    pte.set_valid(true);
                }
                Ok(())
            }
            KernVmAreaType::Mmap => Ok(())
        }
    }

    fn unmap(&mut self, page_table: &mut PageTable) {
        for &vpn in self.frames.keys() {
            page_table.unmap(vpn);
            unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
        }
        self.frames.clear();
    }
}