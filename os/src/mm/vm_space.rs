use core::{num::NonZeroUsize, ops::{Deref, DerefMut}};

use crate::{board::{MEMORY_END, MMIO}, config::{KERNEL_ADDR_OFFSET, PAGE_SIZE, TRAP_CONTEXT, USER_MEMORY_SPACE, USER_STACK_SIZE, USER_STACK_TOP}, mm::{vm_area::{KernelVmAreaType, MapPerm, UserVmArea, UserVmAreaType}, VirtAddr, VirtPageNum}, sync::UPSafeCell};

use super::{page_table::PageTable, vm_area::{KernelVmArea, VmArea, VmAreaCowExt, VmAreaPageFaultExt}, PageTableEntry};

use alloc::{format, vec::Vec};
use lazy_static::lazy_static;
use log::info;
use riscv::register::scause;

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: UPSafeCell<KernelVmSpace> = 
        unsafe { UPSafeCell::new(KernelVmSpace::new()) };
}

#[allow(missing_docs)]
pub trait VmAreaContainer<V: VmArea> {

    fn remove_with_va(&mut self, va: VirtAddr) -> Option<V>;
    fn find_with_va(&mut self, va: VirtAddr) -> Option<&V>;
    fn find_mut_with_va(&mut self, va: VirtAddr) -> Option<&mut V>;

    fn remove_with_vpn(&mut self, vpn: VirtPageNum) -> Option<V> {
        self.remove_with_va(vpn.into())
    }
    fn find_with_vpn(&mut self, vpn: VirtPageNum) -> Option<&V> {
        self.find_with_va(vpn.into())
    }
    fn find_mut_with_vpn(&mut self, vpn: VirtPageNum) -> Option<&mut V> {
        self.find_mut_with_va(vpn.into())
    }

    fn clear(&mut self);
    fn push(&mut self, area: V);
}

#[allow(missing_docs)]
pub trait VmSpace {
    type VmAreaType: VmArea;
    type VmAreaCntrType: VmAreaContainer<Self::VmAreaType>;

    fn get_page_table(&self) -> &PageTable;

    fn get_page_table_mut(&mut self) -> &mut PageTable;

    fn get_areas(&self) -> &Self::VmAreaCntrType;

    fn get_areas_mut(&mut self) -> &mut Self::VmAreaCntrType;

    fn get_pgt_areas_mut(&mut self) -> (&mut PageTable, &mut Self::VmAreaCntrType);

    fn enable(&self) {
        unsafe { self.get_page_table().enable() };
    }

    fn token(&self) -> usize {
        self.get_page_table().token()
    }

    fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.get_page_table().translate(vpn)
    }

    fn push(&mut self, mut map_area: Self::VmAreaType, data: Option<&[u8]>) {
        map_area.map(self.get_page_table_mut());
        if let Some(data) = data {
            map_area.copy_data(&self.get_page_table_mut(), data);
        }
        self.get_areas_mut().push(map_area);
    }

    fn remove_area_with_vpn(&mut self, vpn: VirtPageNum) {
        let (pgt, areas) = self.get_pgt_areas_mut();
        if let Some(mut area) = areas.remove_with_vpn(vpn)
        {
            area.unmap(pgt);
        }
    }

    fn shrink_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        let (pgt, areas) = self.get_pgt_areas_mut();
        if let Some(area) = areas.find_mut_with_vpn(start.floor())
        {   
            area.shrink_to( pgt, new_end.ceil());
            true
        } else {
            false
        }
    }

    fn append_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        let (pgt, areas) = self.get_pgt_areas_mut();
        if let Some(area) = areas.find_mut_with_vpn(start.floor())
        {
            area.append_to(pgt, new_end.ceil());
            true
        } else {
            false
        }
    }

    fn recycle_data_pages(&mut self) {
        self.get_areas_mut().clear();
    }
}

#[allow(missing_docs)]
pub trait VmSpacePageFaultExt: VmSpace<VmAreaType: VmAreaPageFaultExt> {
    fn handle_page_fault(&mut self, 
        va: VirtAddr,
        access_type: PageFaultAccessType) -> Option<()> {
        let (pgt, areas) = self.get_pgt_areas_mut();
        let area = areas.find_mut_with_va(va)?;
        area.handle_page_fault(pgt, va.floor(), access_type)
    }
}

impl<T: VmSpace<VmAreaType: VmAreaPageFaultExt>> VmSpacePageFaultExt for T {}

#[allow(missing_docs, unused)]
pub trait VmSpaceHeapExt: VmSpace {
    fn get_heap_area(&self) -> Option<&Self::VmAreaType>;

    fn get_heap_area_mut(&mut self) -> Option<&mut Self::VmAreaType>;

    fn reset_heap_break(&mut self, new_brk: VirtAddr) -> VirtAddr {
        let heap = self.get_heap_area_mut().unwrap();
        let range = heap.range_va();
        log::debug!("[MemorySpace::reset_heap_break] heap range: {range:?}, new_brk: {new_brk:?}");
        if new_brk >= range.end {
            *heap.range_va_mut() = range.start..new_brk;
            new_brk
        } else if new_brk > range.start {
            let mut area = self.get_areas_mut().remove_with_va(new_brk).unwrap();
            let mut right = area.split_off(new_brk.floor());
            right.unmap(self.get_page_table_mut());
            self.get_areas_mut().push(area);
            new_brk
        } else {
            range.end
        }
    }
}

pub struct KernelVmSpace {
    pub page_table: PageTable,
    areas: Vec<KernelVmArea>
}

impl<V: VmArea> VmAreaContainer<V> for Vec<V> {

    fn remove_with_vpn(&mut self, vpn: VirtPageNum) -> Option<V> {
        let (idx, _) = self
            .iter()
            .enumerate()
            .find(|(_, area)| { 
                area.range_vpn().contains(&vpn) 
            })?;
        Some(self.swap_remove(idx))
    }

    fn find_mut_with_vpn(&mut self, vpn: VirtPageNum) -> Option<&mut V> {
        self.iter_mut().find(|area| { area.range_vpn().contains(&vpn) })
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn push(&mut self, area: V) {
        self.push(area);
    }
    
    fn find_with_vpn(&mut self, vpn: VirtPageNum) -> Option<&V> {
        self.iter().find(|area| { area.range_vpn().contains(&vpn) })
    }
    
    fn remove_with_va(&mut self, va: VirtAddr) -> Option<V> {
        let (idx, _) = self
            .iter()
            .enumerate()
            .find(|(_, area)| { 
                area.range_va().contains(&va) 
            })?;
        Some(self.swap_remove(idx))
    }
    
    fn find_with_va(&mut self, va: VirtAddr) -> Option<&V> {
        self.iter().find(|area| { area.range_va().contains(&va) })
    }
    
    fn find_mut_with_va(&mut self, va: VirtAddr) -> Option<&mut V> {
        self.iter_mut().find(|area| { area.range_va().contains(&va) })
    }
}

impl VmSpace for KernelVmSpace {
    type VmAreaType = KernelVmArea;

    type VmAreaCntrType = Vec<KernelVmArea>;

    fn get_page_table(&self) -> &PageTable {
        &self.page_table
    }

    fn get_page_table_mut(&mut self) -> &mut PageTable {
        &mut self.page_table
    }

    fn get_areas(&self) -> &Self::VmAreaCntrType {
        &self.areas
    }

    fn get_areas_mut(&mut self) -> &mut Self::VmAreaCntrType {
        &mut self.areas
    }

    fn get_pgt_areas_mut(&mut self) -> (&mut PageTable, &mut Self::VmAreaCntrType) {
        (&mut self.page_table, &mut self.areas)
    }

}

impl KernelVmSpace {

    pub fn new() -> Self {
        extern "C" {
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
            page_table: PageTable::new(),
            areas: Vec::new()
        };
        ret.push(
            KernelVmArea::new(
                (stext as usize).into()..(etext as usize).into(), 
                MapPerm::X | MapPerm::R, 
                KernelVmAreaType::Text
            ),
            None
        );

        ret.push(
            KernelVmArea::new(
                (srodata as usize).into()..(erodata as usize).into(), 
                MapPerm::R, 
                KernelVmAreaType::Rodata
            ),
            None
        );


        ret.push(
            KernelVmArea::new(
                (sdata as usize).into()..(edata as usize).into(), 
                MapPerm::R | MapPerm::W, 
                KernelVmAreaType::Data
            ),
            None
        );

        ret.push(
            KernelVmArea::new(
                (sbss_with_stack as usize).into()..(ebss as usize).into(), 
                MapPerm::R | MapPerm::W, 
                KernelVmAreaType::Bss
            ),
            None
        );

        ret.push(
            KernelVmArea::new(
                (ekernel as usize).into()..(MEMORY_END + KERNEL_ADDR_OFFSET).into(), 
                MapPerm::R | MapPerm::W, 
                KernelVmAreaType::PhysMem
            ),
            None
        );

        for pair in MMIO {
            ret.push(
                KernelVmArea::new(
                    ((*pair).0 + KERNEL_ADDR_OFFSET).into()..((*pair).0 + KERNEL_ADDR_OFFSET + (*pair).1).into(),
                    MapPerm::R | MapPerm::W,
                    KernelVmAreaType::MemMappedReg
                ),
                None,
            );
        }

        ret
    }

}

#[allow(missing_docs)]
pub struct UserVmSpace {
    pub page_table: PageTable,
    areas: Vec<UserVmArea>,
    heap: usize
}

impl VmSpace for UserVmSpace {
    type VmAreaType = UserVmArea;

    type VmAreaCntrType = Vec<Self::VmAreaType>;

    fn get_page_table(&self) -> &PageTable {
        &self.page_table
    }

    fn get_page_table_mut(&mut self) -> &mut PageTable {
        &mut self.page_table
    }

    fn get_areas(&self) -> &Self::VmAreaCntrType {
        &self.areas
    }

    fn get_areas_mut(&mut self) -> &mut Self::VmAreaCntrType {
        &mut self.areas
    }

    fn get_pgt_areas_mut(&mut self) -> (&mut PageTable, &mut Self::VmAreaCntrType) {
        (&mut self.page_table, &mut self.areas)
    }
}

impl VmSpaceHeapExt for UserVmSpace {
    fn get_heap_area(&self) -> Option<&Self::VmAreaType> {
        if self.areas[self.heap].vma_type == UserVmAreaType::Heap {
            Some(&self.areas[self.heap])
        } else {
            self.areas.iter().find(|area| { 
                area.vma_type == UserVmAreaType::Heap    
            })
        }
    }

    fn get_heap_area_mut(&mut self) -> Option<&mut Self::VmAreaType> {
        if self.areas[self.heap].vma_type == UserVmAreaType::Heap {
            Some(&mut self.areas[self.heap])
        } else {
            let (idx, area) = self.areas.iter_mut().enumerate().find(|(_, area)| { 
                area.vma_type == UserVmAreaType::Heap
            })?;
            self.heap = idx;
            Some(area)
        }
    }
}

#[allow(missing_docs)]
impl UserVmSpace {
    pub fn from_kernel(kernel_vm_space: &KernelVmSpace) -> Self {
        let page_table = PageTable::new();
        page_table.root_ppn.to_kern().get_pte_array().copy_from_slice(&kernel_vm_space.page_table.root_ppn.to_kern().get_pte_array()[..]);
        Self {
            page_table,
            areas: Vec::new(),
            heap: 0
        }
    }

    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut ret = Self::from_kernel(&KERNEL_SPACE.exclusive_access());
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                println!("start_va: {:#x}, end_va: {:#x}", start_va.0, end_va.0);

                let mut map_perm = MapPerm::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPerm::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPerm::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPerm::X;
                }
                let map_area = UserVmArea::new(start_va..end_va, map_perm, UserVmAreaType::Elf);
                max_end_vpn = map_area.end_vpn();
                ret.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        };
        
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let user_heap_bottom: usize = max_end_va.into();
        // used in brk
        println!("user_heap_bottom: {:#x}", user_heap_bottom);
        ret.heap = ret.areas.len();
        ret.push(
            UserVmArea::new(
                user_heap_bottom.into()..user_heap_bottom.into(),
                MapPerm::R | MapPerm::W | MapPerm::U,
                UserVmAreaType::Heap,
            ),
            None,
        );
        let user_stack_bottom = USER_STACK_TOP - USER_STACK_SIZE;
        let user_stack_top = USER_STACK_TOP;
        println!("user_stack_bottom: {:#x}, user_stack_top: {:#x}", user_stack_bottom, user_stack_top);
        ret.push(
            UserVmArea::new(
                user_stack_bottom.into()..USER_STACK_TOP.into(),
                MapPerm::R | MapPerm::W | MapPerm::U,
                UserVmAreaType::Stack
            ),
            None,
        );
        
        println!("trap_context: {:#x}", TRAP_CONTEXT);
        // map TrapContext
        ret.push(
            UserVmArea::new(
                TRAP_CONTEXT.into()..(USER_MEMORY_SPACE.1).into(),
                MapPerm::R | MapPerm::W,
                UserVmAreaType::TrapContext
            ),
            None,
        );
        (
            ret,
            user_stack_top - 8, // reserve for argc
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub fn from_existed(user_space: &mut UserVmSpace) -> Self {
        let mut ret = Self::from_kernel(&KERNEL_SPACE.exclusive_access());
        for area in user_space.areas.iter_mut() {
            match area.clone_cow(&mut user_space.page_table) {
                Ok(new_area) => {
                    ret.push(new_area, None);
                },
                Err(new_area) => {
                    ret.push(new_area, None);
                    for vpn in area.range_vpn() {
                        let src_ppn = user_space.page_table.translate(vpn).unwrap().ppn();
                        let dst_ppn = ret.translate(vpn).unwrap().ppn();
                        dst_ppn
                            .to_kern()
                            .get_bytes_array()
                            .copy_from_slice(src_ppn.to_kern().get_bytes_array());
                    }
                }
            }
            
        }
        ret
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

#[allow(missing_docs)]
impl PageFaultAccessType {

    pub fn from_exception(e: scause::Exception) -> Self {
        match e {
            scause::Exception::InstructionPageFault => Self::EXECUTE,
            scause::Exception::LoadPageFault => Self::READ,
            scause::Exception::StorePageFault => Self::WRITE,
            _ => panic!("unexcepted exception type for PageFaultAccessType"),
        }
    }

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

#[allow(missing_docs, unused)]
pub fn remap_test() {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
    }

    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = (stext as usize + ((etext as usize - stext as usize) >> 1)).into();
    let mid_rodata: VirtAddr = (srodata as usize + ((erodata as usize - srodata as usize) >> 1)).into();
    let mid_data: VirtAddr = (sdata as usize + ((edata as usize - sdata as usize) >> 1)).into();
    assert!(!kernel_space
        .page_table
        .translate(mid_text.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_rodata.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_data.floor())
        .unwrap()
        .executable(),);
    println!("remap_test passed!");
}

