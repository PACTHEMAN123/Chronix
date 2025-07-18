use core::ops::{Deref, DerefMut, Range};

use alloc::{collections::btree_map::BTreeMap, format, string::{String, ToString}, sync::Arc, vec::Vec};
use hal::{addr::{PhysAddr, PhysAddrHal, PhysPageNum, PhysPageNumHal, RangePPNHal, VirtAddr, VirtAddrHal, VirtPageNum, VirtPageNumHal}, allocator::{FrameAllocatorHal, FrameAllocatorTrackerExt}, constant::{Constant, ConstantsHal}, instruction::{Instruction, InstructionHal}, pagetable::{MapPerm, PageLevel, PageTableEntry, PageTableEntryHal, PageTableHal, VpnPageRangeIter}, println, util::smart_point::StrongArc};
use log::info;
use range_map::RangeMap;
use xmas_elf::reader::Reader;

use crate::{config::PAGE_SIZE, fs::{page, utils::FileReader, vfs::{dentry::global_find_dentry, inode::InodeMode, DentryState, File}, OpenFlags}, ipc::sysv::{self, ShmObj}, mm::{allocator::{frames_alloc, FrameAllocator, SlabAllocator}, vm, FrameTracker, PageTable, KVMSPACE}, sync::mutex::{spin_rw_mutex::SpinRwMutex, MutexSupport, SpinNoIrqLock}, syscall::{mm::MmapFlags, SysError, SysResult}, task::utils::{generate_early_auxv, AuxHeader, AT_BASE, AT_CLKTCK, AT_EGID, AT_ENTRY, AT_EUID, AT_FLAGS, AT_GID, AT_HWCAP, AT_NOTELF, AT_PAGESZ, AT_PHDR, AT_PHENT, AT_PHNUM, AT_PLATFORM, AT_RANDOM, AT_SECURE, AT_UID}, utils::{round_down_to_page, timer::TimerGuard}};

use super::{KernVmArea, KernVmAreaType, KernVmSpaceHal, MapFlags, MaxEndVpn, PageFaultAccessType, StartPoint, UserVmArea, UserVmAreaType, UserVmAreaView, UserVmFile, UserVmSpaceHal};

/// User's VmSpace
pub struct UserVmSpace {
    page_table: PageTable,
    areas: RangeMap<VirtPageNum, UserVmArea>,
    brk: Range<VirtAddr>
}

impl UserVmSpace {

    pub fn new() -> Self {
        Self {
            page_table: PageTable::new_in(0, FrameAllocator),
            areas: RangeMap::new(),
            brk: VirtAddr(0)..VirtAddr(0),
        }
    }

    pub fn enable(&self) {
        unsafe {
            self.get_page_table().enable_low();
            Instruction::tlb_flush_all();
        }
    }

    pub fn get_page_table(&self) -> &PageTable {
        &self.page_table
    }

    pub fn map_elf<T: Reader + ?Sized>(&mut self, elf: &xmas_elf::ElfFile<'_, T>, elf_file: Option<Arc<dyn File>>, offset: VirtAddr) -> 
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
                map_area.file = elf_file.clone().into();
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
    
    pub fn from_elf<T: Reader + ?Sized>(elf: &xmas_elf::ElfFile<'_, T>, elf_file: Option<Arc<dyn File>>) -> 
        Result<(Self, super::StackTop, super::EntryPoint, Vec<AuxHeader>), SysError> {
        let mut ret = KVMSPACE.lock().to_user();

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

        ret.brk = max_end_vpn.start_addr()..max_end_vpn.start_addr();

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
        
        Ok((
            ret,
            user_stack_top,
            entry,
            auxv,
        ))
    }

    pub fn push_area(&mut self, area: UserVmArea, data: Option<&[u8]>) -> &mut UserVmArea {
        match self.areas.try_insert(area.range_vpn(), area) {
            Ok(area) => {
                // println!("[push_area] {:?}", area);
                if let Some(data) = data{
                    area.copy_data(&mut self.page_table, data, 0);
                }
                area.map(&mut self.page_table);
                area
            },
            Err(_) => panic!("[push_area] fail")
        }
    }

    pub fn reset_heap_break(&mut self, new_brk: VirtAddr) -> VirtAddr {
        let range = match self.find_heap() {
            Some(heap) => heap.range_vpn(),
            None => {
                if new_brk > self.brk.end {
                    self.push_area(
                        UserVmArea::new(
                            self.brk.start..new_brk,
                            UserVmAreaType::Heap,
                            MapPerm::R | MapPerm::W | MapPerm::U,
                        ), 
                        None
                    );
                    self.brk.end = new_brk;
                    return new_brk;
                } else {
                    return self.brk.end;
                }
            }
        };
        if new_brk > self.brk.end {
            let new_range = range.start..new_brk.ceil();
            if range == new_range {
                self.brk.end = new_brk;
                return new_brk;
            }
            match self.areas.extend_back(new_range) {
                Ok(_) => {
                    let heap = self.areas.get_mut(range.start).unwrap();
                    heap.range_va.end = new_brk;
                    self.brk.end = new_brk;
                    return new_brk
                }
                Err(_) => return self.brk.end
            }
        } else if new_brk >= self.brk.start {
            while let Some(range) = self.find_heap().map(|vma| vma.range_vpn()) {
                let new_range = range.start..new_brk.ceil();
                if range == new_range {
                    self.brk.end = new_brk;
                    return new_brk;
                }
                if new_range.start >= new_range.end {
                    let heap = self.areas.force_remove_one(range);
                    heap.unmap(&mut self.page_table);
                    self.brk.end = new_brk.max(self.brk.start);
                } else {
                    match self.areas.reduce_back(new_range) {
                        Ok(_) => {
                            let heap = self.areas.get_mut(range.start).unwrap();
                            let right = heap.split_off(new_brk.ceil());
                            right.unmap(&mut self.page_table);
                            self.brk.end = new_brk;
                            return new_brk;
                        }
                        Err(_) => return self.brk.end
                    }
                }
            }
            return self.brk.end;
        } else {
            return self.brk.end;
        }
    }
    
    pub fn from_existed(uvm_space: &mut Self) -> Self {
        let mut ret = KVMSPACE.lock().to_user();
        ret.brk = uvm_space.brk.clone();
        for (_, area) in uvm_space.areas.iter_mut() {
            if let Ok(new_area) =  area.clone_cow(&mut uvm_space.page_table) {
                ret.push_area(new_area, None);
            } else {
                ret.push_area(area.clone(), None);
            }
        }
        ret
    }
    
    pub fn alloc_mmap_area(&mut self, va: VirtAddr, len: usize, perm: MapPerm, flags: MmapFlags, file: Arc<dyn File>, offset: usize) -> Result<VirtAddr, SysError> {
        if len == 0 {
            return Err(SysError::EINVAL);
        }
        let len = (va.page_offset() + len - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE - 1);
        let range = if flags.contains(MmapFlags::MAP_FIXED) {
            let range = va.floor()..(va+len).ceil();
            self.areas.is_range_free(range.clone()).map_err(|_| SysError::ENOMEM)?;
            range
        } else {
            self.areas
            .find_free_range(
                VirtAddr::from(Constant::USER_FILE_BEG).floor()..VirtAddr::from(Constant::USER_FILE_END).floor(), 
                len / Constant::PAGE_SIZE
            )
            .ok_or(SysError::ENOMEM)?
        };
        let range_va = range.start.start_addr()..range.end.start_addr();
        let start = range_va.start;
        let vma = UserVmArea::new_mmap(range_va, perm, flags, UserVmFile::File(file.clone()), offset, len);
        self.push_area(vma, None);
        Ok(start)
    }

    pub fn alloc_anon_area(&mut self, va: VirtAddr, len: usize, perm: MapPerm, flags: MmapFlags, shm: Option<Arc<ShmObj>>) -> Result<VirtAddr, SysError> {
        if len == 0 {
            return Err(SysError::EINVAL);
        }
        if flags.contains(MmapFlags::MAP_SHARED) && shm.is_none() {
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
        if let Some(shm) = shm {
            let vma = UserVmArea::new_mmap(range_va.clone(), perm, flags, UserVmFile::Shm(shm), 0, len);
            self.push_area(vma, None);
        } else {
            let vma = UserVmArea::new_mmap(range_va.clone(), perm, flags, UserVmFile::None, range_va.start.0, len);
            self.push_area(vma, None);
        }
        Ok(start)
    }

    /// try union the VMAs in a given vpn range, if all sucess, return Ok 
    fn try_union(&mut self, vpn: VirtPageNum, pg_len: usize) -> Result<(), ()> {
        let mut start = vpn;
        let end = vpn + pg_len;
        while start < end {
            let vma1 = self.areas.get(start).ok_or(())?;
            let new_start = vma1.range_vpn().end;
            if new_start >= end {
                return Ok(());
            }
            let vma2: &UserVmArea = self.areas.get(new_start).ok_or(())?;
            if vma1.check_back_contiguous(vma2) {
                let vma2 = self.areas.force_remove_one(vma2.range_vpn());
                let vma1 = self.areas.get_mut(start).unwrap();
                vma1.push_back_unchecked(vma2);
                let new_range = vma1.range_vpn().clone();
                let _ = self.areas.extend_back(new_range);
                start = new_start;
            } else {
                return Err(());
            }
        }
        Ok(())
    }
    
    /// unmap one vma in the vpn range `va.floor()..(va+len).ceil()`
    /// return Err when no matched vma
    pub fn unmap(&mut self, va: VirtAddr, len: usize) -> Result<UserVmArea, SysError> {
        let vpn = va.floor();
        let pg_len = (va + len).ceil().0 - vpn.0;
        let _ = self.try_union(vpn, pg_len);
        
        let mut mid: UserVmArea;
        let old_range;
        let new_range;
        if let Some((range_vpn, front)) = self.areas.get_key_value_mut(vpn) {
            mid = front.split_off(va.floor());
            new_range = front.range_vpn();
            old_range = range_vpn;
        } else {
            if let Some((range_vpn, front)) = self.areas.range_mut(vpn..vpn+pg_len).next() {
                mid = front.split_off(va.floor());
                new_range = front.range_vpn();
                old_range = range_vpn;
            } else {
                log::warn!("[unmap] no matched area");
                return Err(SysError::EINVAL);
            }
        }

        if new_range.is_empty() {
            // front area is empty, remove it
            self.areas.force_remove_one(old_range);
        } else {
            // front area is not empty, update rangemap
            let _ = self.areas.reduce_back(new_range);
        }

        if vpn + pg_len < mid.range_vpn().end {
            let back = mid.split_off(vpn + pg_len);
            if !back.range_va.is_empty() {
                self.areas.try_insert(back.range_vpn(), back).map_err(|_| { 
                        log::warn!("[unmap] try insert error");
                        SysError::EFAULT 
                    }
                )?;
            }
        }
        
        mid.unmap(&mut self.page_table);

        Ok(mid)
    }
    
    pub fn check_free(&self, va: VirtAddr, len: usize) -> Result<(), ()> {
        let range = va.floor()..(va+len).ceil();
        self.areas.is_range_free(range)
    }
    
    pub fn get_area_view(&self, va: VirtAddr) -> Option<UserVmAreaView> {
        let area = self.areas.get(va.floor())?;
        Some(area.into())
    }

    pub fn get_area_mut(&mut self, va: VirtAddr) -> Option<&mut UserVmArea> {
        self.areas.get_mut(va.floor())
    }

    pub fn get_area_ref(&self, va: VirtAddr) -> Option<&UserVmArea> {
        self.areas.get(va.floor())
    }

    pub fn handle_page_fault(&mut self, va: VirtAddr, access_type: super::PageFaultAccessType) -> Result<(), ()> {
        let vpn = va.floor();
        if let Some(area) = self.areas.get_mut(va.floor()) {
            area.handle_page_fault(&mut self.page_table, vpn, access_type)
        } else {
            // log::error!("[handle_page_fault] va: {va:?}, no matched vma");
            return Err(());
        }
    }
    
    pub fn access_no_fault(&mut self, va: VirtAddr, len: usize, access_type: super::PageFaultAccessType) -> bool {
        let mut vpn = va.floor();
        let end = (va+len).floor();
        while vpn < end {
            if let Some(area) = self.areas.get_mut(vpn) {
                for vpn in vpn..end.min(area.range_vpn().end) {
                    if !area.access_no_fault(vpn, access_type) {
                        return false;
                    }
                }
                vpn = area.range_vpn().end;
            } else {
                return false;
            }
        }
        return true;
    }
    
    pub fn ensure_access(&mut self, va: VirtAddr, len: usize, access_type: PageFaultAccessType) -> Result<(), ()> {
        if va.0 >= Constant::USER_ADDR_SPACE.end {
            return Err(());
        }
        let mut vpn = va.floor();
        let end = (va+len).ceil();
        while vpn < end {
            if access_type.contains(PageFaultAccessType::WRITE) {
                let ret = unsafe { 
                    hal::trap::try_write_user(vpn.start_addr().0 as *mut u8)
                };
                if ret.is_ok() {
                    vpn += 1;
                    continue;
                }
            } else if access_type.contains(PageFaultAccessType::READ) {
                let ret = unsafe { 
                    hal::trap::try_read_user(vpn.start_addr().0 as *mut u8)
                };
                if ret.is_ok() {
                    vpn += 1;
                    continue;
                }
            }
            if let Some(area) = self.areas.get_mut(vpn) {
                for vpn in vpn..end.min(area.range_vpn().end) {
                    if !area.access_no_fault(vpn, access_type) {
                        area.handle_page_fault(&mut self.page_table, vpn, access_type)?;
                    }
                }
                vpn = area.range_vpn().end;
            } else {
                return Err(())
            }
        }
        return Ok(());
    }

    pub fn ensure_access_in_lock(mutex: &SpinRwMutex<Self, impl MutexSupport>, va: VirtAddr, len: usize, access_type: PageFaultAccessType) -> Result<(), ()> {
        if va.0 >= Constant::USER_ADDR_SPACE.end {
            return Err(());
        }
        let mut vpn = va.floor();
        let end = (va+len).ceil();
        while vpn < end {
            if access_type.contains(PageFaultAccessType::WRITE) {
                let ret = unsafe { 
                    hal::trap::try_write_user(vpn.start_addr().0 as *mut u8)
                };
                if ret.is_ok() {
                    vpn += 1;
                    continue;
                }
            } else if access_type.contains(PageFaultAccessType::READ) {
                let ret = unsafe { 
                    hal::trap::try_read_user(vpn.start_addr().0 as *mut u8)
                };
                if ret.is_ok() {
                    vpn += 1;
                    continue;
                }
            }
            let rself = mutex.rlock();
            if let Some(area) = rself.areas.get(vpn) {
                let mut fault = false;
                for vpn in vpn..end.min(area.range_vpn().end) {
                    if !area.access_no_fault(vpn, access_type) {
                        fault = true;
                        break;
                    }
                }
                if !fault {
                    vpn = area.range_vpn().end;
                    continue;
                }
            } else {
                return Err(())
            }
            let mut wself = match rself.upgrade() {
                Some(v) => v,
                None => mutex.wlock()
            };
            let vm = &mut wself.deref_mut();
            if let Some(area) = vm.areas.get_mut(vpn) {
                for vpn in vpn..end.min(area.range_vpn().end) {
                    if !area.access_no_fault(vpn, access_type) {
                        area.handle_page_fault(&mut vm.page_table, vpn, access_type)?;
                    }
                }
            } else {
                return Err(())
            }
        }
        return Ok(());
    }

    pub fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        self.get_page_table().translate_vpn(vpn)
    }

    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.get_page_table().translate_va(va)
    }

    pub fn clear(&mut self) {
        self.areas.iter_mut().for_each(|(_, vma)| {
            vma.frames.clear();
        });
    }
}

impl UserVmSpace {
    fn find_heap(&mut self) -> Option<&mut UserVmArea> {
        self.areas.get_mut(self.brk.end.ceil() - 1).and_then(|vma| {
            if vma.vma_type != UserVmAreaType::Heap {
                None
            } else {
                Some(vma)
            }
        })
    }

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
        is_dl |= elf_header.pt2.type_().as_type() == xmas_elf::header::Type::SharedObject;
        if !is_dl {
            return Ok(None);
        }

        let mut interp: String;
        if let Some(section) = elf.find_section_by_name(".interp") {
            interp = String::from_utf8(section.raw_data(&elf).to_vec()).unwrap();
            interp = interp.strip_suffix("\0").unwrap_or(&interp).to_string();   
        } else {
            interp = "/lib/libc.so".to_string();
        }
        log::info!("[load_dl] interp {}", interp);

        let interp_file;
        let dentry = global_find_dentry(&interp).expect("cannot find interp dentry");
        if dentry.is_negative() {
            log::warn!("[load_dl] missing dl {}", interp);
            return Err(SysError::ENOENT);
        }
        // log::info!("find symlink: {}, mode: {:?}", dentry.path(), dentry.inode().unwrap().inode_inner().mode);
        // assert link depth <= 1
        let dentry = if dentry.inode().unwrap().inode_type() == InodeMode::LINK {
            let inode = dentry.inode().unwrap();
            let follow_path = inode.readlink()?;
            global_find_dentry(&follow_path)?
        } else {
            dentry
        };
        
        // log::info!("follow symlink to {}", dentry.path());
        interp_file = dentry.open(OpenFlags::O_RDWR).unwrap();

        let reader = FileReader::new(interp_file.clone()).map_err(|_| SysError::ENOEXEC)?;
        let interp_elf = xmas_elf::ElfFile::new(&reader).map_err(|_| SysError::ENOEXEC)?;
        self.map_elf(&interp_elf, Some(interp_file), Constant::DL_INTERP_OFFSET.into());

        Ok(Some((Constant::DL_INTERP_OFFSET, interp_elf.header.pt2.entry_point() as usize + Constant::DL_INTERP_OFFSET)))
    }
}

impl Drop for UserVmSpace {
    fn drop(&mut self) {
        // if this page table is using, switch to KVMSPACE
        if self.page_table.enabled() {
            KVMSPACE.lock().enable();
        }
    }
}

#[allow(missing_docs, unused)]
impl UserVmArea {

    pub fn range_vpn(&self) -> Range<VirtPageNum> {
        self.range_va.start.floor()..self.range_va.end.ceil()
    }

    fn copy_data(&mut self, page_table: &PageTable, data: &[u8], pg_offset: usize) {
        let mut range = self.range_vpn();
        range.start += pg_offset;
        for (vpn, src) in range.zip(data.chunks(Constant::PAGE_SIZE)) {
            let ppn;
            if let Some(_ppn) = page_table.translate_vpn(vpn) {
                ppn = _ppn;
            } else {
                let frame = FrameAllocator.alloc_tracker(1).unwrap();
                ppn = frame.range_ppn.start;
                self.frames.insert(vpn, StrongArc::new(frame));
            }
            let dst = &mut ppn
                    .start_addr()
                    .get_mut::<[u8; Constant::PAGE_SIZE]>();
            dst[..src.len()].copy_from_slice(src);
            dst[src.len()..].fill(0);
        }
    }

    fn split_off(&mut self, p: VirtPageNum) -> Self {
        let new_offset = self.offset + (p.0 - self.range_vpn().start.0) * Constant::PAGE_SIZE;
        let new_len = if new_offset - self.offset > self.len {
            0
        } else {
            self.len - (new_offset - self.offset)
        };
        self.len -= new_len;

        let ret = Self {
            range_va: p.start_addr()..self.range_va.end,
            frames: self.frames.split_off(&p),
            map_perm: self.map_perm,
            vma_type: self.vma_type,
            file: self.file.clone(),
            offset: new_offset,
            map_flags: self.map_flags,
            len: new_len

        };
        self.range_va = self.range_va.start..p.start_addr();
        ret
    }

    fn alloc_frames(&mut self) {
        for vpn in self.range_vpn() {
            let frame = FrameAllocator.alloc_tracker(1).unwrap();
            self.frames.insert(vpn, StrongArc::new(frame));
        }
    }

    fn map(&mut self, page_table: &mut PageTable) {
        for (&vpn, frame) in self.frames.iter() {
            let pte = page_table
                .map(vpn, frame.range_ppn.start, self.map_perm, PageLevel::Small)
                .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
            if frame.get_owners() > 1 && !self.map_flags.contains(MapFlags::SHARED) {
                pte.set_writable(false);
                pte.set_dirty(false);
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
        if !self.map_flags.contains(MapFlags::SHARED) && self.map_perm.contains(MapPerm::W) {
            /// update flag bit
            for &vpn in self.frames.keys() {
                let (pte, _) = page_table.find_pte(vpn).unwrap();
                pte.set_writable(false);
                pte.set_dirty(false);
                unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
            }
        }
        Ok(Self {
            range_va: self.range_va.clone(), 
            frames: self.frames.clone(), 
            map_perm: self.map_perm.clone(), 
            vma_type: self.vma_type.clone(),
            file: self.file.clone(),
            map_flags: self.map_flags.clone(),
            offset: self.offset,
            len: self.len
        })
    }

    pub fn extend(&mut self, size: usize) {
        if size == 0 {
            return;
        }
        self.range_va.end += size;
        self.range_va.end = self.range_va.end.ceil().start_addr();
        if self.file.is_some() {
            self.len += size;
        }
    }

    pub fn shrink(&mut self, size: usize) {
        if size == 0 {
            return;
        }
        self.split_off((self.range_va.end - size).floor());
    }

    pub fn move_frames_to(&mut self, other: &mut Self) {
        let self_start =  self.range_va.start.floor();
        let other_start = other.range_va.start.floor();
        for (vpn, frame) in self.frames.iter() {
            let new_vpn = other_start + (vpn.0 - self_start.0);
            other.frames.insert(new_vpn, frame.clone());
        }
        self.frames.clear();
    }

    pub fn handle_page_fault(&mut self, 
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
                if !access_type.contains(PageFaultAccessType::WRITE) {
                    return Err(());
                }
                if pte.is_writable() {
                    return Ok(());
                }
                if self.map_flags.contains(MapFlags::SHARED) {
                    pte.set_writable(true);
                    pte.set_dirty(true);
                    unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
                    return Ok(());
                }
                let old_frame = self.frames.get_mut(&vpn).unwrap();
                if old_frame.get_owners() > 1 {
                    let new_frame = frames_alloc(1).unwrap();
                    new_frame.range_ppn.get_slice_mut::<usize>().copy_from_slice(
                        old_frame.range_ppn.get_slice()
                    );
                    pte.set_ppn(new_frame.range_ppn.start);
                    old_frame.emplace(new_frame);
                }
                pte.set_writable(true);
                pte.set_dirty(true);
                unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
                Ok(())
            }
            _ => {
                let ret = match self.vma_type {
                    UserVmAreaType::Data =>
                        UserDataHandler::handle_lazy_page_fault(self, page_table, vpn, access_type),
                    UserVmAreaType::Stack =>
                        UserStackHandler::handle_lazy_page_fault(self, page_table, vpn, access_type),
                    UserVmAreaType::Heap =>
                        UserHeapHandler::handle_lazy_page_fault(self, page_table, vpn, access_type),
                    UserVmAreaType::Mmap =>
                        UserMmapHandler::handle_lazy_page_fault(self, page_table, vpn, access_type)
                };
                ret
            }
        }
    }

    pub fn check_back_contiguous(&self, back: &Self) -> bool {
        if self.range_va.end != back.range_va.start {
            return false;
        }
        if self.vma_type != back.vma_type {
            return false;
        }
        if self.map_perm != back.map_perm {
            return false;
        }
        if self.map_flags != back.map_flags {
            return false;
        }
        if self.file != back.file {
            return false;
        }
        if self.offset + self.len != back.offset {
            return false;
        }
        return true;
    }

    pub fn push_back_unchecked(&mut self, mut back: Self) {
        self.range_va.end = back.range_va.end;
        self.len += back.len;
        self.frames.append(&mut back.frames);
    }

    pub fn push_back(&mut self, back: Self) -> Result<(), Self> {
        if !self.check_back_contiguous(&back) {
            return Err(back);
        }
        self.push_back_unchecked(back);
        Ok(())
    }

    fn access_no_fault(&self, vpn: VirtPageNum, access_type: PageFaultAccessType) -> bool {
        if let Some(frame) = self.frames.get(&vpn) {
            if access_type.contains(PageFaultAccessType::WRITE) && !self.map_flags.contains(MapFlags::SHARED){
                false
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl Clone for UserVmArea {
    fn clone(&self) -> Self {
        let frames;
        if !self.map_flags.contains(MapFlags::SHARED) {
            let mut new_frames = BTreeMap::new();
            for (&vpn, frame) in self.frames.iter() {
                let new_frame = FrameAllocator.alloc_tracker(frame.range_ppn.clone().count()).unwrap();
                new_frame.range_ppn.get_slice_mut::<usize>().copy_from_slice(frame.range_ppn.get_slice());
                new_frames.insert(vpn, StrongArc::new(new_frame));
            }
            frames = new_frames;
        } else {
            frames = self.frames.clone();
        }
        Self { 
            range_va: self.range_va.clone(), 
            vma_type: self.vma_type.clone(), 
            map_perm: self.map_perm.clone(), 
            frames,
            file: self.file.clone(),
            map_flags: self.map_flags.clone(),
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

#[repr(C, align(4096))]
struct ZeroPage([u8; 4096]);

const ZERO_PAGE: ZeroPage = ZeroPage([0u8; 4096]);

lazy_static::lazy_static!{
    static ref ZERO_PAGE_ARC: StrongArc<FrameTracker> = {
        let ppn = PhysAddr(&ZERO_PAGE as *const _ as usize & !Constant::KERNEL_ADDR_SPACE.start).floor();
        StrongArc::new(
            FrameTracker::new_in(ppn..ppn+1, FrameAllocator)
        )
    };
}

/// tool structure
struct PageFaultProcessor;

#[allow(unused)]
impl PageFaultProcessor {
    /// map zero page
    fn map_zero_page(
        page_table: &mut PageTable,
        vpn: VirtPageNum,
        access_type: PageFaultAccessType,
        perm: MapPerm,
        frames: &mut BTreeMap<VirtPageNum, StrongArc<FrameTracker>>,
    ) -> Result<(), ()> {
        if access_type.contains(PageFaultAccessType::WRITE) {
            let frame = FrameAllocator.alloc_tracker(1).ok_or(())?;
            frame.range_ppn.get_slice_mut::<usize>().fill(0);
            let pte = page_table
                    .map(vpn, frame.range_ppn.start, perm, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
            pte.set_dirty(true);
            frames.insert(vpn, StrongArc::new(frame));
        } else { // zero page optimize
            let mut new_perm = perm;
            new_perm.remove(MapPerm::W);
            let pte = page_table
                    .map(vpn, ZERO_PAGE_ARC.range_ppn.start, new_perm, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
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
        frames: &mut BTreeMap<VirtPageNum, StrongArc<FrameTracker>>,
    ) -> Result<(), ()> {
        let inode = file.inode().unwrap().clone();
        if len < Constant::PAGE_SIZE {
            let new_frame = FrameAllocator.alloc_tracker(1).ok_or(())?;
            let data = new_frame.range_ppn.get_slice_mut::<u8>();
            let page = match inode.read_page_at(offset) {
                Some(page) => page,
                None => { 
                    log::error!("[map_private_file] no page");
                    return Err(());
                }
            };
            data[len..].fill(0);
            data[..len].copy_from_slice(&page.get_slice()[..len]);
            let pte = page_table
                .map(vpn, new_frame.range_ppn.start, perm, PageLevel::Small)
                .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
            if access_type.contains(PageFaultAccessType::WRITE) {
                pte.set_dirty(true);
            }
            frames.insert(vpn, StrongArc::new(new_frame));
        } else {
            if access_type.contains(PageFaultAccessType::WRITE) {
                let new_frame = FrameAllocator.alloc_tracker(1).ok_or(())?;
                let page = match inode.read_page_at(offset) {
                    Some(page) => page,
                    None => { 
                        log::error!("[map_private_file] no page");
                        return Err(());
                    }
                };
                let data = new_frame.range_ppn.get_slice_mut::<u8>();
                data.copy_from_slice(page.get_slice());
                let pte = page_table
                    .map(vpn, new_frame.range_ppn.start, perm, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
                pte.set_dirty(true);
                frames.insert(vpn, StrongArc::new(new_frame));
            } else {
                let page = match inode.read_page_at(offset) {
                    Some(page) => page,
                    None => { 
                        log::error!("[map_private_file] no page");
                        return Err(());
                    }
                };
                let mut new_perm = perm;
                new_perm.remove(MapPerm::W);
                let pte = page_table
                    .map(vpn, page.ppn(), new_perm, PageLevel::Small)
                    .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
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
        frames: &mut BTreeMap<VirtPageNum, StrongArc<FrameTracker>>,
    ) -> Result<(), ()> {
        let inode = file.inode().ok_or(())?.clone();
        // share file mapping
        let page = match inode.read_page_at(offset) {
            Some(page) => page,
            None => { 
                log::error!("[map_shared_file] no page");
                return Err(());
            }
        };
        // map a single page
        let pte = page_table
            .map(vpn, page.ppn(), perm, PageLevel::Small)
            .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
        if access_type.contains(PageFaultAccessType::WRITE) {
            pte.set_dirty(true);
            page.set_dirty();
        }
        frames.insert(vpn, page.frame());
        unsafe { Instruction::tlb_flush_addr(vpn.start_addr().0); }
        Ok(())
    }

    fn map_shared_memory(
        page_table: &mut PageTable,
        vpn: VirtPageNum,
        access_type: PageFaultAccessType,
        shm: Arc<sysv::ShmObj>,
        offset: usize,
        perm: MapPerm,
        frames: &mut BTreeMap<VirtPageNum, StrongArc<FrameTracker>>
    ) -> Result<(), ()> {
        // share file mapping
        let page = match shm.read_page_at(offset) {
            Some(page) => page,
            None => { 
                log::error!("[map_shared_memory] no page");
                return Err(());
            }
        };
        // map a single page
        let pte = page_table
            .map(vpn, page.ppn(), perm, PageLevel::Small)
            .expect(format!("vpn: {:#x} is mapped", vpn.0).as_str());
        if access_type.contains(PageFaultAccessType::WRITE) {
            pte.set_dirty(true);
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
        if let UserVmFile::File(file) = vma.file.clone() {
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
        let vma_file = vma.file.clone();
        if let UserVmFile::File(file) = vma_file {
            // file mapping
            let offset = vma.offset + (vpn.0 - vma.range_va.start.floor().0) * Constant::PAGE_SIZE;
            assert_eq!(offset % Constant::PAGE_SIZE, 0);
            if vma.map_flags.contains(MapFlags::SHARED) {
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
        } else if let UserVmFile::Shm(shm) = vma_file {
            // shm mapping
            let offset = vma.offset + (vpn.0 - vma.range_va.start.floor().0) * Constant::PAGE_SIZE;
            assert_eq!(offset % Constant::PAGE_SIZE, 0);
            PageFaultProcessor::map_shared_memory(
                page_table, 
                vpn, 
                access_type, 
                shm.clone(), 
                offset,
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
    }
}


/// lock pages avoid swapping out
pub struct UserVmPagesLocker {
    // todo...
}

impl Clone for UserVmPagesLocker {
    fn clone(&self) -> Self {
        Self {  }
    }
}

impl Drop for UserVmPagesLocker {
    fn drop(&mut self) {
        // to unlock pages
    }
}

unsafe impl Send for UserVmPagesLocker {}
