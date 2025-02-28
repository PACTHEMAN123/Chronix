use core::ops::{Deref, DerefMut};

use crate::{board::{MEMORY_END, MMIO}, config::{KERNEL_ADDR_OFFSET, PAGE_SIZE, TRAP_CONTEXT, USER_MEMORY_SPACE, USER_STACK_SIZE}, mm::{vm_area::{KernelVmAreaType, MapPerm, UserVmArea, UserVmAreaType}, VirtAddr, VirtPageNum}, sync::UPSafeCell};

use super::{page_table::PageTable, vm_area::{KernelVmArea, VmArea}, PageTableEntry};

use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::info;

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: UPSafeCell<KernelVmSpace> = 
        unsafe { UPSafeCell::new(KernelVmSpace::new()) };
}

#[allow(missing_docs)]
pub trait VmSpace {
    type VmAreaType: VmArea;
    fn get_page_table(&self) -> &PageTable;

    fn get_page_table_mut(&mut self) -> &mut PageTable;

    fn enable(&self) {
        unsafe { self.get_page_table().enable() };
    }

    fn token(&self) -> usize {
        self.get_page_table().token()
    }

    fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.get_page_table().translate(vpn)
    }

    fn push(&mut self, map_area: Self::VmAreaType, data: Option<&[u8]>);

    fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum);
}

pub struct BaseVmSpace<V: VmArea> {
    pub page_table: PageTable,
    areas: Vec<V>
}

impl<V: VmArea> BaseVmSpace<V> {
    pub fn shrink_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        if let Some(area) = self.areas
            .iter_mut()
            .find(|area| area.start_vpn() == start.floor())
        {   
            area.shrink_to( &mut self.page_table, new_end.ceil());
            true
        } else {
            false
        }
    }

    pub fn append_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
        if let Some(area) = self.areas
            .iter_mut()
            .find(|area| area.start_vpn() == start.floor())
        {
            area.append_to(&mut self.page_table, new_end.ceil());
            true
        } else {
            false
        }
    }

    pub fn recycle_data_pages(&mut self) {
        self.areas.clear();
    }
}

impl<V: VmArea> VmSpace for BaseVmSpace<V> {
    type VmAreaType = V;

    fn get_page_table(&self) -> &PageTable {
        &self.page_table
    }

    fn get_page_table_mut(&mut self) -> &mut PageTable {
        &mut self.page_table
    }
    
    fn push(&mut self, mut map_area: Self::VmAreaType, data: Option<&[u8]>) {
        map_area.map(self.get_page_table_mut());
        if let Some(data) = data {
            map_area.copy_data(&self.get_page_table_mut(), data);
        }
        self.areas.push(map_area);
    }
    
    fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, area)) = self
            .areas
            .iter_mut()
            .enumerate()
            .find(|(_, area)| area.start_vpn() == start_vpn)
        {
            area.unmap(&mut self.page_table);
            self.areas.swap_remove(idx);
        }
    }

}

pub struct KernelVmSpace {
    base: BaseVmSpace<KernelVmArea>
}

impl Deref for KernelVmSpace {
    type Target = BaseVmSpace<KernelVmArea>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for KernelVmSpace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
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
            base: BaseVmSpace {
                page_table: PageTable::new(),
                areas: Vec::new()
            }
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
    base: BaseVmSpace<UserVmArea>
}

impl Deref for UserVmSpace {
    type Target = BaseVmSpace<UserVmArea>;

    fn deref(&self) -> &Self::Target {
        & self.base
    }
}

impl DerefMut for UserVmSpace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[allow(missing_docs)]
impl UserVmSpace {
    pub fn from_kernel(kernel_vm_space: &KernelVmSpace) -> Self {
        let page_table = PageTable::new();
        page_table.root_ppn.get_pte_array().copy_from_slice(&kernel_vm_space.page_table.root_ppn.get_pte_array()[..]);
        Self {
            base: BaseVmSpace {
                page_table,
                areas: Vec::new()
            }
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
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        let user_stack_top_extend = user_stack_top;
        println!("user_stack_bottom: {:#x}, user_stack_top: {:#x}", user_stack_bottom, user_stack_top);
        ret.push(
            UserVmArea::new(
                user_stack_bottom.into()..user_stack_top_extend.into(),
                MapPerm::R | MapPerm::W | MapPerm::U,
                UserVmAreaType::Elf
            ),
            None,
        );
        // used in sbrk
        ret.push(
            UserVmArea::new(
                user_stack_top.into()..user_stack_top.into(),
                MapPerm::R | MapPerm::W | MapPerm::U,
                UserVmAreaType::Elf,
            ),
            None,
        );
        println!("trap_context: {:#x}", TRAP_CONTEXT);
        // map TrapContext
        ret.push(
            UserVmArea::new(
                TRAP_CONTEXT.into()..(USER_MEMORY_SPACE.1).into(),
                MapPerm::R | MapPerm::W,
                UserVmAreaType::Elf
            ),
            None,
        );
        (
            ret,
            user_stack_top - 8, // reserve for argc
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub fn from_existed(user_space: &UserVmSpace) -> Self {
        let mut ret = Self::from_kernel(&KERNEL_SPACE.exclusive_access());
        for area in user_space.areas.iter() {
            let new_area = area.clone();
            ret.push(new_area, None);
            for vpn in area.range_vpn() {
                let src_ppn = user_space.translate(vpn).unwrap().ppn();
                let dst_ppn = ret.translate(vpn).unwrap().ppn();
                dst_ppn
                    .get_bytes_array()
                    .copy_from_slice(src_ppn.get_bytes_array());
            }
        }
        ret
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

