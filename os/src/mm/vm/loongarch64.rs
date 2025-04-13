use core::ops::{Deref, Range};

use alloc::{collections::btree_map::BTreeMap, format, string::{String, ToString}, sync::Arc, vec::Vec};
use hal::{addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, RangePPNHal, VirtAddr, VirtAddrHal, VirtPageNum, VirtPageNumHal}, allocator::FrameAllocatorHal, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::{MapPerm, PTEFlags, PageLevel, PageTableEntry, PageTableEntryHal, PageTableHal, VpnPageRangeIter}, println, util::smart_point::StrongArc};
use range_map::RangeMap;
use xmas_elf::reader::Reader;

use crate::{config::PAGE_SIZE, fs::{page, utils::FileReader, vfs::{dentry::global_find_dentry, file::open_file, DentryState, File}, OpenFlags}, mm::{allocator::{FrameAllocator, SlabAllocator}, FrameTracker, PageTable}, syscall::{mm::MmapFlags, SysError, SysResult}, task::utils::{generate_early_auxv, AuxHeader, AT_BASE, AT_CLKTCK, AT_EGID, AT_ENTRY, AT_EUID, AT_FLAGS, AT_GID, AT_HWCAP, AT_NOTELF, AT_PAGESZ, AT_PHDR, AT_PHENT, AT_PHNUM, AT_PLATFORM, AT_RANDOM, AT_SECURE, AT_UID}, utils::round_down_to_page};

use super::{KernVmArea, KernVmAreaType, KernVmSpaceHal, MaxEndVpn, PageFaultAccessType, StartPoint, UserVmArea, UserVmAreaType, UserVmSpaceHal};

/// Kernel's VmSpace
pub struct KernVmSpace {
    page_table: PageTable,
    areas: RangeMap<VirtPageNum, KernVmArea>,
}

/// User's VmSpace
pub struct UserVmSpace {
    page_table: PageTable,
    areas: RangeMap<VirtPageNum, UserVmArea>,
    heap_bottom_va: VirtAddr
}

impl KernVmSpaceHal for KernVmSpace {

    fn enable(&self) {
        unsafe { 
            self.page_table.enable_high();
            Instruction::tlb_flush_all();
        }
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
    
    fn push_area(&mut self, area: KernVmArea, _data: Option<&[u8]>) {
        area.map(&mut self.page_table);
        let _ = self.areas.try_insert(area.range_vpn(), area);
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
                self.page_table.map(vpn, page.ppn(), MapPerm::R, PageLevel::Small);
                area.frames.insert(vpn, page.frame());
                unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
                Ok(())
            },
            _ => Err(())
        }
    }

}

#[allow(missing_docs, unused)]
impl UserVmSpace {
    fn find_heap(&mut self) -> Option<&mut UserVmArea> {
        while let Some(area) = self.areas.get_mut(self.heap_bottom_va.floor()) {
            if area.vma_type != UserVmAreaType::Heap {
                self.heap_bottom_va = area.range_vpn().end.start_addr();
            } else {
                break;
            }
        }
        self.areas.get_mut(self.heap_bottom_va.floor())
    }
}

impl UserVmSpaceHal for UserVmSpace {

    fn new() -> Self {
        Self {
            page_table: PageTable::new_in(0, FrameAllocator),
            areas: RangeMap::new(),
            heap_bottom_va: VirtAddr(0),
        }
    }

    fn get_page_table(&self) -> &PageTable {
        &self.page_table
    }

    fn from_kernel() -> Self {
        Self::new()
    }

    fn map_elf<T: Reader + ?Sized>(&mut self, elf: &xmas_elf::ElfFile<'_, T>, elf_file: Option<Arc<dyn File>>, offset: VirtAddr) -> 
        (MaxEndVpn, StartPoint) {
        let elf_header = elf.header;
        let ph_count = elf_header.pt2.ph_count();

        let mut max_end_vpn = offset.floor();
        let mut header_va = 0;
        let mut has_found_header_va = false;
        // map the elf data to user space
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize + offset.0).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize + offset.0).into();
                log::debug!("i: {}, start_va: {:#x}, end_va: {:#x}", i, start_va.0, end_va.0);
                if !has_found_header_va {
                    header_va = start_va.0;
                    has_found_header_va = true;
                }

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
               
                log::debug!("{:?}", &elf.input.read(ph.offset() as usize, 4));                
                let elf_offset_start = PhysAddr::from(ph.offset() as usize).floor().start_addr().0;
                let elf_offset_end = (ph.offset() + ph.file_size()) as usize;
                log::debug!("{:x} aligned to {:x}, now pushing ({:x}, {:x})", ph.offset() as usize, elf_offset_start, elf_offset_start, elf_offset_end);
                
                let mut map_area = UserVmArea::new(
                    start_va.floor().start_addr()..end_va.ceil().start_addr(), 
                    UserVmAreaType::Data,
                    map_perm,
                );
                map_area.file = elf_file.clone();
                map_area.offset = elf_offset_start;
                map_area.len = elf_offset_end - elf_offset_start;

                max_end_vpn = map_area.range_vpn().end;
                let data = if map_area.file.is_none() {
                    Some(elf.input.read(map_area.offset, map_area.len))
                } else {
                    None
                };

                self.push_area(
                    map_area,
                    data
                );
            }
        };

        (
            max_end_vpn,
            header_va.into()
        )
    }
    
    fn from_elf<T: Reader + ?Sized>(elf: &xmas_elf::ElfFile<'_, T>, elf_file: Option<Arc<dyn File>>) -> 
        Result<(Self, super::StackTop, super::EntryPoint, Vec<AuxHeader>), SysError> {
        let mut ret = Self::from_kernel();

        let elf_header = elf.header;
        let mut entry = elf_header.pt2.entry_point() as usize;
        let ph_count = elf_header.pt2.ph_count();
        let ph_entry_size = elf.header.pt2.ph_entry_size() as usize;
        // extract the aux
        // let mut auxv = generate_early_auxv(ph_entry_size, ph_count as usize, entry);
        let mut auxv = Vec::with_capacity(64);
        auxv.push(AuxHeader::new( 
            AT_PHENT, ph_entry_size)); // ELF64 header 64bytes
        auxv.push(AuxHeader::new(AT_PHNUM, ph_count as usize));
        auxv.push(AuxHeader::new(AT_PAGESZ, Constant::PAGE_SIZE));
        auxv.push(AuxHeader::new(AT_ENTRY, entry as usize));

        if let Some((offset, interp_entry_point)) = ret.load_dl_interp_if_needed(&elf)? {
            auxv.push(AuxHeader::new(AT_BASE, offset));
            entry = interp_entry_point;
        } else {
            auxv.push(AuxHeader::new(AT_BASE, 0));
        }
        
        auxv.push(AuxHeader::new(AT_FLAGS, 0 as usize));
        auxv.push(AuxHeader::new(AT_UID, 0 as usize));
        auxv.push(AuxHeader::new(AT_EUID, 0 as usize));
        auxv.push(AuxHeader::new(AT_GID, 0 as usize));
        auxv.push(AuxHeader::new(AT_EGID, 0 as usize));
        auxv.push(AuxHeader::new(AT_PLATFORM, 0 as usize));
        auxv.push(AuxHeader::new(AT_HWCAP, 0 as usize));
        auxv.push(AuxHeader::new(AT_CLKTCK, 100 as usize));
        auxv.push(AuxHeader::new(AT_SECURE, 0 as usize));
        auxv.push(AuxHeader::new(AT_NOTELF, 0x112d as usize));

        // map the elf data to user space
        let (max_end_vpn, header_va) = ret.map_elf(&elf, elf_file, 0.into());

        let ph_head_addr = header_va.0 + elf.header.pt2.ph_offset() as usize;
        auxv.push(AuxHeader::new(AT_RANDOM, ph_head_addr));
        auxv.push(AuxHeader::new(AT_PHDR, ph_head_addr));

        ret.heap_bottom_va = max_end_vpn.start_addr();

        // map user stack with U flags
        let user_stack_bottom = Constant::USER_STACK_BOTTOM;
        let user_stack_top = Constant::USER_STACK_TOP;
        log::debug!("user_stack_bottom: {:#x}, user_stack_top: {:#x}", user_stack_bottom, user_stack_top);
        ret.push_area(
            UserVmArea::new(
                user_stack_bottom.into()..user_stack_top.into(),
                UserVmAreaType::Stack,
                MapPerm::R | MapPerm::W | MapPerm::U,
            ),
            None,
        );
        
        log::debug!("trap_context: {:#x}", Constant::USER_TRAP_CONTEXT_BOTTOM);
        
        let mut trap_cx_area = UserVmArea::new(
            Constant::USER_TRAP_CONTEXT_BOTTOM.into()..(Constant::USER_TRAP_CONTEXT_TOP).into(),
            UserVmAreaType::TrapContext,
            MapPerm::R | MapPerm::W,
        );
        trap_cx_area.alloc_frames();
        // map TrapContext
        ret.push_area(
            trap_cx_area,
            None,
        );
        Ok((
            ret,
            user_stack_top,
            entry,
            auxv,
        ))
    }

    fn push_area(&mut self, area: UserVmArea, data: Option<&[u8]>) -> &mut UserVmArea{
        match self.areas.try_insert(area.range_vpn(), area) {
            Ok(area) => {
                if let Some(data) = data{
                    area.copy_data(&mut self.page_table, data);
                } 
                area.map(&mut self.page_table);
                area
            },
            Err(_) => panic!("[push_area] fail")
        }
    }

    fn reset_heap_break(&mut self, new_brk: VirtAddr) -> VirtAddr {
        let heap = match self.find_heap() {
            Some(heap) => heap,
            None => {
                if new_brk > self.heap_bottom_va {
                    self.push_area(
                        UserVmArea::new(
                            self.heap_bottom_va..new_brk,
                            UserVmAreaType::Heap,
                            MapPerm::R | MapPerm::W | MapPerm::U,
                        ), 
                        None
                    );
                    return new_brk;
                } else {
                    return self.heap_bottom_va;
                }
            }
        };
        let range = heap.range_va.clone();
        if new_brk.ceil() > range.end.ceil() {
            match self.areas.extend_back(range.start.floor()..new_brk.ceil()) {
                Ok(_) => {}
                Err(_) => return range.end
            }
        } else if new_brk.ceil() > range.start.floor() && new_brk.ceil() < range.end.ceil() {
            match self.areas.reduce_back(range.start.floor()..new_brk.ceil()) {
                Ok(_) => {}
                Err(_) => return range.end
            }
        }

        let heap = self.find_heap().unwrap();
        if new_brk >= range.end {
            heap.range_va = range.start..new_brk;
            new_brk
        } else if new_brk > range.start {
            let right = heap.split_off(new_brk.ceil());
            right.unmap(&mut self.page_table);
            new_brk
        } else {
            range.end
        }
    }

    fn handle_page_fault(&mut self, va: VirtAddr, access_type: super::PageFaultAccessType) -> Result<(), ()> {
        let area = self.areas.get_mut(va.floor()).ok_or(())?;
        area.handle_page_fault(&mut self.page_table, va.floor(), access_type)
    }
    
    fn from_existed(uvm_space: &mut Self) -> Self {
        let mut ret = Self::from_kernel();
        ret.heap_bottom_va = uvm_space.heap_bottom_va;
        for (_, area) in uvm_space.areas.iter_mut() {
            if let Ok(new_area) =  area.clone_cow(&mut uvm_space.page_table) {
                ret.push_area(new_area, None);
            } else {
                ret.push_area(area.clone(), None);
            }
        }
        ret
    }
    
    fn alloc_mmap_area(&mut self, va: VirtAddr, len: usize, perm: MapPerm, flags: MmapFlags, file: Arc<dyn File>, offset: usize) -> SysResult {
        if len == 0 {
            return Err(SysError::EINVAL);
        }
        let len = (va.page_offset() + len - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE - 1);
        let range = if flags.contains(MmapFlags::MAP_FIXED) {
            let range = va.floor()..(va+len).ceil();
            self.areas.is_range_free(range.clone()).map_err(|_| SysError::ENOMEM)?;
            println!("{:#x} {:#x}", range.start.0, range.end.0);
            range
        } else {
            self.areas
            .find_free_range(
                VirtAddr::from(Constant::USER_FILE_BEG).floor()..VirtAddr::from(Constant::USER_FILE_END).floor(), 
                len / Constant::PAGE_SIZE
            )
            .ok_or(SysError::ENOMEM)?
        };
        // println!("va {:#x} len {:#x}", va.0, len);
        let range_va = range.start.start_addr()..range.end.start_addr();
        let start = range_va.start;
        let page_table = &mut self.page_table;
        let inode = file.inode().unwrap();
        let mut vma = UserVmArea::new_mmap(range_va, perm, flags, Some(file.clone()), offset, len);
        let mut range_vpn = vma.range_vpn();
        let length = core::cmp::min(len, Constant::USER_FILE_PER_PAGES * PAGE_SIZE);
        // the offset is already page aligned
        for page_offset in (offset..offset + length).step_by(PAGE_SIZE) {
            // get the cached page
            if let Some(page) = inode.clone().read_page_at(page_offset) {
                // page already in cache
                let vpn = range_vpn.next().unwrap();
                if flags.contains(MmapFlags::MAP_PRIVATE) {
                    // private mode: map in COW
                    let mut new_perm = perm;
                    new_perm.remove(MapPerm::W);
                    new_perm.insert(MapPerm::C);
                    // map a single page
                    page_table.map(vpn, page.ppn(), new_perm, PageLevel::Small);
                    vma.frames.insert(vpn, StrongArc::clone(&page.frame()));
                    vma.map_perm.insert(MapPerm::C);
                    // update tlb                     
                    unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
                } else {
                    // share mode
                    log::info!("[alloc_mmap_area]: mapping vpn:{:x} to ppn:{:x}", vpn.0, page.ppn().0);
                    page_table.map(vpn, page.ppn(), perm, PageLevel::Small);
                    vma.frames.insert(vpn, StrongArc::clone(&page.frame()));
                    unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
                }
            } else {
                // reach EOF
                break;
            }
        }
        self.push_area(vma, None);
        Ok(start.0 as isize)
    }

    fn alloc_anon_area(&mut self, va: VirtAddr, len: usize, perm: MapPerm, flags: MmapFlags, is_share: bool) -> SysResult {
        if len == 0 {
            return Err(SysError::EINVAL);
        }
        let len = (va.page_offset() + len - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE - 1);
        let va= va.floor().start_addr();
        let range = if flags.contains(MmapFlags::MAP_FIXED) {
            let range = va.floor()..(va+len).ceil();
            self.areas.is_range_free(range.clone()).map_err(|_| SysError::ENOMEM)?;
            range
        } else {
            self.areas
            .find_free_range(
                VirtAddr::from(Constant::USER_SHARE_BEG).floor()..VirtAddr::from(Constant::USER_SHARE_END).floor(), 
                len / Constant::PAGE_SIZE
            )
            .ok_or(SysError::ENOMEM)?
        };
        let range_va = range.start.start_addr()..range.end.start_addr();
        let start = range_va.start;
        if is_share {
            let vma = UserVmArea::new(range_va, UserVmAreaType::Shm, perm);
            self.push_area(vma, None);
        } else {
            let vma = UserVmArea::new_mmap(range_va, perm, flags, None, 0, len);
            self.push_area(vma, None);
        }
        Ok(start.0 as isize)
    }

    
    fn unmap(&mut self, va: VirtAddr, len: usize) -> Result<UserVmArea, SysError> {
        let mut left: UserVmArea;
        let right: UserVmArea;
        let mut mid: UserVmArea;
        if let Some((range_vpn, _)) = self.areas.get_key_value_mut(va.floor()) {
            left = self.areas.force_remove_one(range_vpn);
            mid = left.split_off(va.floor());
            mid.unmap(&mut self.page_table);
            right = mid.split_off((va + len).ceil());
        } else {
            return Err(SysError::ENOMEM);
        }
        if !left.range_va.is_empty() {
            self.areas.try_insert(left.range_vpn(), left).map_err(|_| SysError::EFAULT)?;
        }
        if !right.range_va.is_empty() {
            self.areas.try_insert(right.range_vpn(), right).map_err(|_| SysError::EFAULT)?;
        }
        Ok(mid)
    }
    
    fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        self.page_table.translate_vpn(vpn)
    }
    
    fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.page_table.translate_va(va)
    }    
}

impl UserVmSpace {
    fn load_dl_interp_if_needed<T: Reader + ?Sized>(&mut self, elf: &xmas_elf::ElfFile<'_, T>) -> Result<Option<(usize, usize)>, SysError> {
        let elf_header = elf.header;
        let ph_count = elf_header.pt2.ph_count();
        let mut is_dl = false;
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Interp {
                is_dl = true;
                break;
            }
        };
        if !is_dl {
            return Ok(None);
        }

        let section = elf.find_section_by_name(".interp").unwrap();
        let mut interp = String::from_utf8(section.raw_data(&elf).to_vec()).unwrap();
        interp = interp.strip_suffix("\0").unwrap_or(&interp).to_string();
        log::info!("[load_dl] interp {}", interp);

        let interp_file;
        let dentry = global_find_dentry(&interp);
        if dentry.state() == DentryState::NEGATIVE {
            return Err(SysError::ENOENT);
        }
        // log::info!("find symlink: {}, mode: {:?}", dentry.path(), dentry.inode().unwrap().inner().mode);
        let dentry = dentry.follow()?;
        // log::info!("follow symlink to {}", dentry.path());
        interp_file = dentry.open(OpenFlags::O_RDWR).unwrap();

        let reader = FileReader::new(interp_file.clone());
        let interp_elf = xmas_elf::ElfFile::new(&reader).map_err(|_| SysError::ENOEXEC)?;
        self.map_elf(&interp_elf, Some(interp_file), Constant::DL_INTERP_OFFSET.into());

        Ok(Some((Constant::DL_INTERP_OFFSET, interp_elf.header.pt2.entry_point() as usize + Constant::DL_INTERP_OFFSET)))
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
            page_table.map(vpn, ppn, self.map_perm, level);
        });
    }

    fn map(&self, page_table: &mut PageTable) {
        unsafe extern "C" {
            fn sigreturn_trampoline();
        }
        let range_vpn = self.range_va.start.floor()..self.range_va.end.ceil();
        match self.vma_type {
            KernVmAreaType::Data |
            KernVmAreaType::PhysMem |
            KernVmAreaType::MemMappedReg |
            KernVmAreaType::KernelStack => {
            },
            KernVmAreaType::SigretTrampoline => {
                let sigret_trampoline_ppn = 
                    PhysPageNum((sigreturn_trampoline as usize & !(Constant::KERNEL_ADDR_SPACE.start)) >> 12);
                for (vpn, ppn) in self.range_vpn().zip(sigret_trampoline_ppn..sigret_trampoline_ppn+1) {
                    let pte = page_table.find_pte_create(vpn, PageLevel::Small)
                        .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
                    *pte = PageTableEntry::new(ppn, self.map_perm, true);
                    
                    pte.set_flags(pte.flags() | PTEFlags::D);
                }
            }
            KernVmAreaType::VirtMemory => {
                for (&vpn, frame) in self.frames.iter() {
                    let pte = page_table.find_pte_create(vpn, PageLevel::Small)
                        .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
                    *pte = PageTableEntry::new(frame.range_ppn.start, self.map_perm, true);
                    pte.set_flags(pte.flags() | PTEFlags::D);
                }
            }
            KernVmAreaType::Mmap => {}
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

#[allow(missing_docs, unused)]
impl UserVmArea {

    fn range_vpn(&self) -> Range<VirtPageNum> {
        self.range_va.start.floor()..self.range_va.end.ceil()
    }

    fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        for (vpn, src) in self.range_vpn().zip(data.chunks(Constant::PAGE_SIZE)) {
            let ppn;
            if let Some(_ppn) = page_table.translate_vpn(vpn) {
                ppn = _ppn;
            } else {
                let frame = FrameAllocator.alloc_tracker(1).unwrap();
                ppn = frame.range_ppn.start;
                self.frames.insert(vpn, StrongArc::new_in(frame, SlabAllocator));
            }
            let dst = &mut ppn
                    .start_addr()
                    .get_mut::<[u8; Constant::PAGE_SIZE]>();
            dst[..src.len()].copy_from_slice(src);
            dst[src.len()..].fill(0);
        }
    }

    fn split_off(&mut self, p: VirtPageNum) -> Self {
        let new_offset ;
        let new_len;
        if self.file.is_some() {
            new_offset = self.offset + (p.0 - self.range_vpn().start.0) * Constant::PAGE_SIZE;
            new_len = if new_offset - self.offset > self.len {
                0
            } else {
                self.len - (new_offset - self.offset)
            };
            self.len -= new_len;
        } else {
            new_offset = 0;
            new_len = 0;
        }

        let ret = Self {
            range_va: p.start_addr()..self.range_va.end,
            frames: self.frames.split_off(&p),
            map_perm: self.map_perm,
            vma_type: self.vma_type,
            file: self.file.clone(),
            offset: new_offset,
            mmap_flags: self.mmap_flags,
            len: new_len

        };
        self.range_va = self.range_va.start..p.start_addr();
        ret
    }

    fn alloc_frames(&mut self) {
        for vpn in self.range_vpn() {
            let frame = FrameAllocator.alloc_tracker(1).unwrap();
            self.frames.insert(vpn, StrongArc::new_in(frame, SlabAllocator));
        }
    }

    fn map(&mut self, page_table: &mut PageTable) {
        if self.vma_type == UserVmAreaType::TrapContext {
            for (&vpn, frame) in self.frames.iter() {
                let pte = page_table
                    .find_pte_create(vpn, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
                *pte = PageTableEntry::new(frame.range_ppn.start, self.map_perm, true);
                pte.set_flags(pte.flags() | PTEFlags::D);
            }
        } else {
            for (&vpn, frame) in self.frames.iter() {
                let pte = page_table
                    .find_pte_create(vpn, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
                *pte = PageTableEntry::new(frame.range_ppn.start, self.map_perm, true);
            }
        }
    }

    fn unmap(&self, page_table: &mut PageTable) {
        for &vpn in self.frames.keys() {
            page_table.unmap(vpn);
            unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
        }
    }

    fn clone_cow(&mut self, page_table: &mut PageTable) -> Result<Self, ()> {
        // note: trap context cannot supprt COW
        if self.vma_type == UserVmAreaType::TrapContext {
            return Err(());
        }
        // note: don't set C flag for readonly frames
        if self.map_perm.contains(MapPerm::W) {
            self.map_perm.insert(MapPerm::C);
            self.map_perm.remove(MapPerm::W);
            /// update flag bit
            for &vpn in self.frames.keys() {
                let (pte, _) = page_table.find_pte(vpn).unwrap();
                *pte = PageTableEntry::new(pte.ppn(), self.map_perm, true);
                unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
            }
        } else if self.map_perm.contains(MapPerm::C) {
            /// update flag bit
            for &vpn in self.frames.keys() {
                let (pte, _) = page_table.find_pte(vpn).unwrap();
                *pte = PageTableEntry::new(pte.ppn(), self.map_perm, true);
                unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
            }
        }
        Ok(Self {
            range_va: self.range_va.clone(), 
            frames: self.frames.clone(), 
            map_perm: self.map_perm.clone(), 
            vma_type: self.vma_type.clone(),
            file: self.file.clone(),
            mmap_flags: self.mmap_flags.clone(),
            offset: self.offset,
            len: self.len
        })
    }

    fn handle_page_fault(&mut self, 
        page_table: &mut PageTable, 
        vpn: VirtPageNum,
        access_type: PageFaultAccessType
    ) -> Result<(), ()> {
        if !access_type.can_access(self.map_perm) {
            log::warn!(
                "[VmArea::handle_page_fault] permission not allowed, perm:{:?}",
                self.map_perm
            );
            return Err(());
        }
        match page_table.find_pte(vpn).map(|(pte, i)| (pte, PageLevel::from(i)) ) {
            Some((pte, _)) if pte.is_valid() => {
                if !access_type.contains(PageFaultAccessType::WRITE)
                    || !pte.map_perm().contains(MapPerm::C) {
                    return Err(());
                }
                PageFaultProcessor::handle_cow_page(vpn, pte, &mut self.frames)
            }
            _ => {
                match self.vma_type {
                    UserVmAreaType::TrapContext => return Err(()),
                    UserVmAreaType::Data =>
                        UserDataHandler::handle_lazy_page_fault(self, page_table, vpn, access_type),
                    UserVmAreaType::Stack =>
                        UserStackHandler::handle_lazy_page_fault(self, page_table, vpn, access_type),
                    UserVmAreaType::Heap =>
                        UserHeapHandler::handle_lazy_page_fault(self, page_table, vpn, access_type),
                    UserVmAreaType::Mmap =>
                        UserMmapHandler::handle_lazy_page_fault(self, page_table, vpn, access_type),
                    UserVmAreaType::Shm =>
                        UserShmHandler::handle_lazy_page_fault(self, page_table, vpn, access_type),
                }
            }
        }
    }

}

impl Clone for UserVmArea {
    fn clone(&self) -> Self {
        let mut frames = BTreeMap::new();
        for (&vpn, frame) in self.frames.iter() {
            let new_frame = FrameAllocator.alloc_tracker(frame.range_ppn.clone().count()).unwrap();
            new_frame.range_ppn.get_slice_mut::<usize>().copy_from_slice(frame.range_ppn.get_slice());
            frames.insert(vpn, StrongArc::new_in(new_frame, SlabAllocator));
        }
        Self { 
            range_va: self.range_va.clone(), 
            vma_type: self.vma_type.clone(), 
            map_perm: self.map_perm.clone(), 
            frames,
            file: self.file.clone(),
            mmap_flags: self.mmap_flags.clone(),
            offset: self.offset,
            len: self.len
        }
    }
}

trait UserLazyFaultHandler {
    #[allow(unused_variables)]
    fn handle_lazy_page_fault(
        vma: &mut UserVmArea,
        page_table: &mut PageTable,
        vpn: VirtPageNum,
        access_type: PageFaultAccessType,
    ) -> Result<(), ()> {
        Err(())
    }
}

#[repr(C)]
#[repr(align(4096))]
struct ZeroPage([u8; 4096]);

static ZERO_PAGE: ZeroPage = ZeroPage([0u8; 4096]);

lazy_static::lazy_static!{
    static ref ZERO_PAGE_ARC: StrongArc<FrameTracker, SlabAllocator> = 
        StrongArc::new_in(
            FrameTracker::new_in(
                PhysAddr(&ZERO_PAGE as *const _ as usize & !Constant::KERNEL_ADDR_SPACE.start).floor()..
                PhysAddr(&ZERO_PAGE as *const _ as usize & !Constant::KERNEL_ADDR_SPACE.start).floor()+1, 
                FrameAllocator
            ), 
            SlabAllocator
        );
}

/// tool structure
struct PageFaultProcessor;

#[allow(unused)]
impl PageFaultProcessor {
    /// handle cow page
    fn handle_cow_page(
        vpn: VirtPageNum,
        pte: &mut PageTableEntry,
        frames: &mut BTreeMap<VirtPageNum, StrongArc<FrameTracker, SlabAllocator>>
    ) -> Result<(), ()> {
        let frame = frames.get_mut(&vpn).ok_or(())?;
        if frame.get_owners() == 1 {
            let mut new_perm = pte.map_perm();
            new_perm.remove(MapPerm::C);
            new_perm.insert(MapPerm::W);
            pte.set_flags(PTEFlags::from(new_perm) | PTEFlags::V);
            pte.set_flags(pte.flags() | PTEFlags::D);
            unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0) };
            Ok(())
        } else {
            let new_frame = StrongArc::new_in(
                FrameAllocator.alloc_tracker(1).ok_or(())?,
                SlabAllocator
            );
            let new_range_ppn = new_frame.range_ppn.clone();

            let old_data = frame.range_ppn.get_slice::<u8>();
            new_range_ppn.get_slice_mut::<u8>().copy_from_slice(old_data);

            *frame = new_frame;
            
            let mut new_perm = pte.map_perm();
            new_perm.remove(MapPerm::C);
            new_perm.insert(MapPerm::W);
            *pte = PageTableEntry::new(new_range_ppn.start, new_perm, true);
            pte.set_flags(pte.flags() | PTEFlags::D);
            unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0) };
            Ok(())
        }
    }

    /// map zero page
    fn map_zero_page(
        page_table: &mut PageTable,
        vpn: VirtPageNum,
        access_type: PageFaultAccessType,
        perm: MapPerm,
        frames: &mut BTreeMap<VirtPageNum, StrongArc<FrameTracker, SlabAllocator>>,
    ) -> Result<(), ()> {
        if access_type.contains(PageFaultAccessType::WRITE) {
            let frame = FrameAllocator.alloc_tracker(1).ok_or(())?;
            frame.range_ppn.get_slice_mut::<u8>().fill(0);
            let pte = page_table
                    .find_pte_create(vpn, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
            *pte = PageTableEntry::new(frame.range_ppn.start, perm, true);
            pte.set_flags(pte.flags() | PTEFlags::D);
            frames.insert(vpn, StrongArc::new_in(frame, SlabAllocator));
        } else { // zero page optimize
            let pte = page_table
                    .find_pte_create(vpn, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
            let mut new_perm = perm;
            if perm.contains(MapPerm::W) {
                new_perm.remove(MapPerm::W);
                new_perm.insert(MapPerm::C);
            }
            *pte = PageTableEntry::new(ZERO_PAGE_ARC.range_ppn.start, new_perm, true);
            frames.insert(vpn, ZERO_PAGE_ARC.clone());
        }
        
        unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0) };
        Ok(())
    }

    /// map private file
    fn map_private_file(
        page_table: &mut PageTable,
        vpn: VirtPageNum,
        access_type: PageFaultAccessType,
        file: Arc<dyn File>,
        offset: usize,
        len: usize,
        perm: MapPerm,
        frames: &mut BTreeMap<VirtPageNum, StrongArc<FrameTracker, SlabAllocator>>,
    ) -> Result<(), ()> {
        let inode = file.inode().unwrap().clone();
        if len < Constant::PAGE_SIZE {
            let new_frame = FrameAllocator.alloc_tracker(1).ok_or(())?;
            let data = new_frame.range_ppn.get_slice_mut::<u8>();
            let page = inode.read_page_at(offset).ok_or(())?;
            data[len..].fill(0);
            data[..len].copy_from_slice(&page.get_slice()[..len]);
            let pte = page_table
                .find_pte_create(vpn, PageLevel::Small)
                .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
            *pte = PageTableEntry::new(new_frame.range_ppn.start, perm, true);
            if access_type.contains(PageFaultAccessType::WRITE) {
                pte.set_flags(pte.flags() | PTEFlags::D);
            }
            frames.insert(vpn, StrongArc::new_in(new_frame, SlabAllocator));
        } else {
            if access_type.contains(PageFaultAccessType::WRITE) {
                let new_frame = FrameAllocator.alloc_tracker(1).ok_or(())?;
                let page = inode.read_page_at(offset).ok_or(())?;
                let data = new_frame.range_ppn.get_slice_mut::<u8>();
                data.copy_from_slice(page.get_slice());
                let pte = page_table
                    .find_pte_create(vpn, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
                *pte = PageTableEntry::new(new_frame.range_ppn.start, perm, true);
                pte.set_flags(pte.flags() | PTEFlags::D);
                frames.insert(vpn, StrongArc::new_in(new_frame, SlabAllocator));
            } else {
                let page = inode.read_page_at(offset).ok_or(())?;
                let mut new_perm = perm;
                if perm.contains(MapPerm::W) {
                    new_perm.insert(MapPerm::C);
                    new_perm.remove(MapPerm::W);
                }
                let pte = page_table
                    .find_pte_create(vpn, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
                *pte = PageTableEntry::new(page.ppn(), perm, true);
                frames.insert(vpn, page.frame());
            }
        }
        unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
        Ok(())
    }

    /// map shared file
    fn map_shared_file(
        page_table: &mut PageTable,
        vpn: VirtPageNum,
        access_type: PageFaultAccessType,
        file: Arc<dyn File>,
        offset: usize,
        perm: MapPerm,
        frames: &mut BTreeMap<VirtPageNum, StrongArc<FrameTracker, SlabAllocator>>,
    ) -> Result<(), ()> {
        let inode = file.inode().ok_or(())?.clone();
        // share file mapping
        let page = inode.read_page_at(offset).ok_or(())?;
        // map a single page
        let pte = page_table
            .find_pte_create(vpn, PageLevel::Small)
            .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
        *pte = PageTableEntry::new(page.ppn(), perm, true);
        if access_type.contains(PageFaultAccessType::WRITE) {
            pte.set_flags(pte.flags() | PTEFlags::D);
            page.set_dirty();
        }
        frames.insert(vpn, page.frame());
        unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
        Ok(())
    }
}

/// User Data Page Fault Handler
struct UserDataHandler;
/// User Mmap Page Fault Handler
struct UserMmapHandler;
/// User Shared Memory Page Fault Handler
struct UserShmHandler;
/// User Stack Page Fault Handler
struct UserStackHandler;
/// User Heap Page Fault Handler
struct UserHeapHandler;
/// Cow Page Fault Handler

#[allow(missing_docs, unused)]
impl UserLazyFaultHandler for UserDataHandler {
    fn handle_lazy_page_fault(
            vma: &mut UserVmArea,
            page_table: &mut PageTable,
            vpn: VirtPageNum,
            access_type: PageFaultAccessType,
        ) -> Result<(), ()> {
        if let Some(file) = vma.file.clone() {
            assert_eq!(vma.offset % Constant::PAGE_SIZE, 0);
            let area_offset = (vpn.0 - vma.range_va.start.floor().0) * Constant::PAGE_SIZE;
            if area_offset < vma.len {
                let offset = vma.offset + area_offset;
                let len = Constant::PAGE_SIZE.min(vma.len - area_offset);
                PageFaultProcessor::map_private_file(
                    page_table, 
                    vpn, 
                    access_type, 
                    file.clone(), 
                    offset,
                    len,
                    vma.map_perm, 
                    &mut vma.frames
                )
            } else {
                PageFaultProcessor::map_zero_page(
                    page_table, 
                    vpn, 
                    access_type, 
                    vma.map_perm, 
                    &mut vma.frames
                )
            }
        } else {
            PageFaultProcessor::map_zero_page(
                page_table, 
                vpn, 
                access_type, 
                vma.map_perm, 
                &mut vma.frames
            )
        }
    }
}

impl UserLazyFaultHandler for UserStackHandler {
    fn handle_lazy_page_fault(
            vma: &mut UserVmArea,
            page_table: &mut PageTable,
            vpn: VirtPageNum,
            access_type: PageFaultAccessType,
        ) -> Result<(), ()> {
        PageFaultProcessor::map_zero_page(page_table, vpn, access_type, vma.map_perm, &mut vma.frames)
    }
}

impl UserLazyFaultHandler for UserHeapHandler {
    fn handle_lazy_page_fault(
            vma: &mut UserVmArea,
            page_table: &mut PageTable,
            vpn: VirtPageNum,
            access_type: PageFaultAccessType,
        ) -> Result<(), ()> {
        PageFaultProcessor::map_zero_page(page_table, vpn, access_type, vma.map_perm, &mut vma.frames)
    }
}

impl UserLazyFaultHandler for UserMmapHandler {
    fn handle_lazy_page_fault(
        vma: &mut UserVmArea,
        page_table: &mut PageTable,
        vpn: VirtPageNum,
        access_type: PageFaultAccessType,
    ) -> Result<(), ()> {
        if !vma.mmap_flags.contains(MmapFlags::MAP_ANONYMOUS) {
            // file mapping
            let file = vma.file.as_ref().unwrap();
            let offset = vma.offset + (vpn.0 - vma.range_va.start.floor().0) * Constant::PAGE_SIZE;
            assert_eq!(offset % Constant::PAGE_SIZE, 0);
            if vma.mmap_flags.contains(MmapFlags::MAP_SHARED) {
                PageFaultProcessor::map_shared_file(
                    page_table, 
                    vpn, 
                    access_type, 
                    file.clone(), 
                    offset,
                    vma.map_perm, 
                    &mut vma.frames
                )
            } else {
                // private file mapping
                PageFaultProcessor::map_private_file(
                    page_table, 
                    vpn, 
                    access_type, 
                    file.clone(), 
                    offset,
                    Constant::PAGE_SIZE,
                    vma.map_perm, 
                    &mut vma.frames
                )
            }
        } else if vma.mmap_flags.contains(MmapFlags::MAP_PRIVATE) {
            if vma.mmap_flags.contains(MmapFlags::MAP_SHARED) {
                panic!("should not reach here")
            } else {
                PageFaultProcessor::map_zero_page(
                    page_table, 
                    vpn, 
                    access_type, 
                    vma.map_perm, 
                    &mut vma.frames
                )
            }
        } else {
            Ok(())
        }
    }
}

impl UserLazyFaultHandler for UserShmHandler {
    fn handle_lazy_page_fault(
            _vma: &mut UserVmArea,
            _page_table: &mut PageTable,
            _vpn: VirtPageNum,
            _access_type: PageFaultAccessType,
        ) -> Result<(), ()> {
        panic!("todo... ")
    }
}