use core::ops::Range;

use alloc::sync::Arc;
use hal::{addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, RangePPNHal, VirtAddr, VirtAddrHal, VirtPageNum, VirtPageNumHal}, allocator::FrameAllocatorHal, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::{MapPerm, PageLevel, PageTableEntry, PageTableEntryHal, PageTableHal, VpnPageRangeIter}};
use range_map::RangeMap;

use crate::{fs::vfs::File, mm::{allocator::FrameAllocator, vm::KernVmAreaType, PageTable}};

use super::super::{KernVmArea, KernVmSpaceHal, PageFaultAccessType, UserVmSpace, UserVmSpaceHal};

/// Kernel's VmSpace
pub struct KernVmSpace {
    page_table: PageTable,
    areas: RangeMap<VirtPageNum, KernVmArea>,
}

impl KernVmSpace {
    /// The second-level page table in the kernel virtual mapping area is pre-allocated to avoid synchronization
    fn map_vm_area_huge_pages(&mut self) {
        let ptes = self.page_table.root_ppn
            .start_addr().get_mut::<[PageTableEntry; Constant::PTES_PER_PAGE]>();

        const HUGE_PAGES: usize = Constant::KERNEL_VM_SIZE / (Constant::PAGE_SIZE * 512 * 512);
        const VM_START: usize = (Constant::KERNEL_VM_BOTTOM & ((1 << Constant::VA_WIDTH)-1)) / (Constant::PAGE_SIZE * 512 * 512);
        let range_ppn = FrameAllocator.alloc(HUGE_PAGES).unwrap();
        range_ppn.get_slice_mut::<u8>().fill(0);
        let ppn = range_ppn.start;
        for (i, pte_i) in (VM_START..VM_START+HUGE_PAGES).enumerate() {
            ptes[pte_i] = PageTableEntry::new(ppn+i, MapPerm::empty());
            ptes[pte_i].set_valid(true);
        }
    }
}

impl KernVmSpaceHal for KernVmSpace {

    fn enable(&self) {
        unsafe {
            self.page_table.enable_high();
        }
    }

    fn get_page_table(&self) -> &PageTable {
        &self.page_table
    }

    fn new() -> Self{

        unsafe extern "C" {
            fn stext();
            fn etext();
            fn srodata();
            fn erodata();
            fn sdata();
            fn edata();
            fn sbss_with_stack();
            fn ebss();
            fn ekernel();
        }

        let mut ret = Self {
            page_table: PageTable::new_in(0, FrameAllocator),
            areas: RangeMap::new(),
        };

        ret.map_vm_area_huge_pages();

        ret.push_area(KernVmArea::new(
                (stext as usize).into()..(etext as usize).into(), 
                KernVmAreaType::Data, 
                MapPerm::R | MapPerm::X,
            ),
            None
        );

        ret.push_area(KernVmArea::new(
                (srodata as usize).into()..(erodata as usize).into(), 
                KernVmAreaType::Data, 
                MapPerm::R,
            ),
            None
        );

        ret.push_area(KernVmArea::new(
                (sdata as usize).into()..(edata as usize).into(), 
                KernVmAreaType::Data, 
                MapPerm::R | MapPerm::W,
            ),
            None
        );

        ret.push_area(KernVmArea::new(
                (sdata as usize).into()..(edata as usize).into(), 
                KernVmAreaType::Data, 
                MapPerm::R | MapPerm::W,
            ),
            None
        );

        ret.push_area(KernVmArea::new(
                (sbss_with_stack as usize).into()..(ebss as usize).into(), 
                KernVmAreaType::Data, 
                MapPerm::R | MapPerm::W, 
            ),
            None
        );

        ret.push_area(KernVmArea::new(
                Constant::KERNEL_STACK_BOTTOM.into()..Constant::KERNEL_STACK_TOP.into(), 
                KernVmAreaType::KernelStack, 
                MapPerm::R | MapPerm::W,
            ),
            None
        );

        ret.push_area(KernVmArea::new(
                Constant::SIGRET_TRAMPOLINE_BOTTOM.into()..Constant::SIGRET_TRAMPOLINE_TOP.into(), 
                KernVmAreaType::SigretTrampoline, 
                MapPerm::R | MapPerm::X | MapPerm::U,
            ),
            None
        );

        ret.push_area(KernVmArea::new(
                (ekernel as usize).into()..(Constant::MEMORY_END + Constant::KERNEL_ADDR_SPACE.start).into(), 
                KernVmAreaType::PhysMem, 
                MapPerm::R | MapPerm::W,
            ),
            None
        );
        
        for pair in hal::board::MMIO {
            ret.push_area(
                KernVmArea::new(
                    ((*pair).0 + Constant::KERNEL_ADDR_SPACE.start).into()..((*pair).0 + Constant::KERNEL_ADDR_SPACE.start + (*pair).1).into(),
                    KernVmAreaType::MemMappedReg, 
                    MapPerm::R | MapPerm::W,
                ),
                None
            );
        }
        ret
    }

    fn to_user<T: UserVmSpaceHal>(&self) -> T {
        let ret = T::new();

        ret.get_page_table().root_ppn
            .start_addr()
            .get_mut::<[PageTableEntry; 512]>()[256..]
            .copy_from_slice(
                &self.page_table.root_ppn
                    .start_addr()
                    .get_mut::<[PageTableEntry; 512]>()[256..]
            );
        ret
    }
    
    fn push_area(&mut self, mut area: KernVmArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table);
        if let Some(data) = data{
            area.copy_data(&mut self.page_table, data);
        }
        let _ = self.areas.try_insert(area.range_vpn(), area);
    }
    
    fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PhysPageNum>{
        self.page_table.translate_vpn(vpn)
    }
    
    fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.page_table.translate_va(va)
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

    fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        let mut start: usize = 0;
        let len = data.len();
        for vpn in self.range_vpn() {
            let src = &data[start..len.min(start + Constant::PAGE_SIZE)];
            if let Some(ppn)  = page_table.translate_vpn(vpn) {
                let dst = &mut ppn.start_addr()
                    .get_mut::<[u8; Constant::PAGE_SIZE]>()[..src.len()];
                dst.copy_from_slice(src);
                start += Constant::PAGE_SIZE;
                if start >= len {
                    break;
                }
            } else {
                panic!("copy data to unmap frame");
            }
        }
    }

    fn split_off(&mut self, p: VirtPageNum) -> Self {
        let ret = Self {
            range_va: p.start_addr()..self.range_va.end,
            frames: self.frames.split_off(&p),
            map_perm: self.map_perm,
            vma_type: self.vma_type,
            file: self.file.clone()
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

    fn map(&self, page_table: &mut PageTable) {
        unsafe extern "C" {
            fn kernel_stack_bottom();
            fn sigreturn_trampoline();
        }
        let range_vpn = self.range_va.start.floor()..self.range_va.end.ceil();
        match self.vma_type {
            KernVmAreaType::Data |
            KernVmAreaType::PhysMem |
            KernVmAreaType::MemMappedReg => {
                self.map_range_to(
                    page_table,
                    range_vpn.clone(), 
                    PhysPageNum(range_vpn.start.0 & !(Constant::KERNEL_ADDR_SPACE.start >> Constant::PAGE_SIZE_BITS))
                );
            },
            KernVmAreaType::SigretTrampoline => {
                self.map_range_to(
                    page_table, 
                    range_vpn.clone(),
                    PhysPageNum((sigreturn_trampoline as usize & !(Constant::KERNEL_ADDR_SPACE.start)) >> 12)
                );
            }
            KernVmAreaType::KernelStack => {
                self.map_range_to(
                    page_table, 
                    range_vpn.clone(),
                    PhysPageNum((kernel_stack_bottom as usize & !(Constant::KERNEL_ADDR_SPACE.start)) >> 12)
                );
            },
            KernVmAreaType::VirtMemory => {
                for (&vpn, frame) in self.frames.iter() {
                    let _ = page_table.map(vpn, frame.range_ppn.start, self.map_perm, PageLevel::Small);
                }
            },
            KernVmAreaType::Mmap => {}
        }
    }

    fn unmap(&mut self, page_table: &mut PageTable) {
        for &vpn in self.frames.keys() {
            let _ = page_table.unmap(vpn);
            unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
        }
    }
}
